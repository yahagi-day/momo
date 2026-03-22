//! WebRTC session: str0m event loop + H.264 encoding + frame delivery.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use momo_core::frame::Frame;
use str0m::change::SdpPendingOffer;
use str0m::media::{Direction, MediaKind, MediaTime, Mid};
use str0m::net::{Protocol, Receive};
use str0m::{Candidate, Event, Input, Output, Rtc};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc};

use crate::convert::uyvy_to_nv12;
use crate::encoder::H264Encoder;
use crate::manager::SubscribeFn;
use crate::signal::{ClientMessage, ServerMessage};

/// Run a WebRTC session as a tokio task.
///
/// Manages the str0m Rtc instance, UDP socket, H.264 encoding, and frame delivery.
pub async fn run_session(
    mut signal_rx: mpsc::Receiver<ClientMessage>,
    signal_tx: mpsc::Sender<ServerMessage>,
    subscribe_fn: SubscribeFn,
    preview_width: u32,
    preview_height: u32,
    preview_fps: u32,
) {
    // Bind a UDP socket for WebRTC media transport
    let udp = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to bind UDP socket: {e}");
            let _ = signal_tx
                .send(ServerMessage::Error {
                    message: format!("UDP bind failed: {e}"),
                })
                .await;
            return;
        }
    };

    let local_addr = match udp.local_addr() {
        Ok(a) => a,
        Err(_) => return,
    };
    tracing::info!(%local_addr, "WebRTC UDP socket bound");

    // Create str0m Rtc instance (server mode with ICE lite)
    let mut rtc = Rtc::builder()
        .set_ice_lite(true)
        .enable_h264(true)
        .enable_vp8(false)
        .enable_vp9(false)
        .build();

    // Add local ICE candidate
    if let Ok(candidate) = Candidate::host(local_addr, "udp") {
        rtc.add_local_candidate(candidate);
    }

    // Track state: stream_id → (Mid, frame receiver)
    let mut tracks: HashMap<String, Mid> = HashMap::new();
    let mut frame_receivers: HashMap<String, broadcast::Receiver<Arc<Frame>>> = HashMap::new();
    let mut pending_offer: Option<SdpPendingOffer> = None;

    // Initialize encoder
    let mut encoder = match H264Encoder::new(preview_width, preview_height, preview_fps) {
        Ok(enc) => enc,
        Err(e) => {
            tracing::error!("failed to create H.264 encoder: {e}");
            let _ = signal_tx
                .send(ServerMessage::Error {
                    message: format!("encoder init failed: {e}"),
                })
                .await;
            return;
        }
    };

    let mut buf = vec![0u8; 2000];
    let mut media_time_counter: u64 = 0;
    let media_time_increment = 90_000u64 / preview_fps as u64; // 90kHz clock

    loop {
        // Drive str0m: poll for output
        let timeout = loop {
            match rtc.poll_output() {
                Ok(Output::Timeout(t)) => break t,
                Ok(Output::Transmit(transmit)) => {
                    let _ = udp.send_to(&transmit.contents, transmit.destination).await;
                }
                Ok(Output::Event(event)) => {
                    handle_rtc_event(&event, &signal_tx).await;
                }
                Err(e) => {
                    tracing::warn!("str0m error: {e}");
                    return;
                }
            }
        };

        let duration = timeout.saturating_duration_since(Instant::now());

        tokio::select! {
            // 1. Signaling message from WebSocket
            msg = signal_rx.recv() => {
                let Some(msg) = msg else { break };
                handle_signal_message(
                    msg, &mut rtc, &mut tracks, &mut frame_receivers,
                    &mut pending_offer, &signal_tx, &subscribe_fn,
                ).await;
            }

            // 2. UDP packet from remote peer
            result = udp.recv_from(&mut buf) => {
                if let Ok((n, source)) = result {
                    let data = &buf[..n];
                    if let Ok(contents) = data.try_into() {
                        let receive = Receive {
                            proto: Protocol::Udp,
                            source,
                            destination: local_addr,
                            contents,
                        };
                        if rtc.handle_input(Input::Receive(Instant::now(), receive)).is_err() {
                            break;
                        }
                    }
                }
            }

            // 3. Timeout
            _ = tokio::time::sleep(duration) => {
                if rtc.handle_input(Input::Timeout(Instant::now())).is_err() {
                    break;
                }
            }
        }

        // Drain frame receivers and encode for all subscribed tracks
        let track_ids: Vec<String> = tracks.keys().cloned().collect();
        for stream_id in &track_ids {
            let mid = tracks[stream_id];
            if let Some(rx) = frame_receivers.get_mut(stream_id) {
                // Take the latest frame (drain older ones)
                let mut latest_frame: Option<Arc<Frame>> = None;
                loop {
                    match rx.try_recv() {
                        Ok(frame) => latest_frame = Some(frame),
                        Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }

                if let Some(frame) = latest_frame {
                    let nv12 = uyvy_to_nv12(
                        &frame.data,
                        frame.resolution.width,
                        frame.resolution.height,
                    );

                    match encoder.encode(&nv12) {
                        Ok(Some(packet)) => {
                            if let Some(writer) = rtc.writer(mid) {
                                let params: Vec<_> = writer.payload_params().collect();
                                if let Some(param) = params.first() {
                                    let pt = param.pt();
                                    let now = Instant::now();
                                    let mt = MediaTime::from_90khz(media_time_counter);
                                    let _ = writer.write(pt, now, mt, packet.data);
                                }
                            }
                            media_time_counter += media_time_increment;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            tracing::warn!("encode error: {e}");
                        }
                    }
                }
            }
        }
    }

    tracing::info!("WebRTC session ended");
}

async fn handle_signal_message(
    msg: ClientMessage,
    rtc: &mut Rtc,
    tracks: &mut HashMap<String, Mid>,
    frame_receivers: &mut HashMap<String, broadcast::Receiver<Arc<Frame>>>,
    pending_offer: &mut Option<SdpPendingOffer>,
    signal_tx: &mpsc::Sender<ServerMessage>,
    subscribe_fn: &SubscribeFn,
) {
    match msg {
        ClientMessage::Answer { sdp } => {
            if let Some(pending) = pending_offer.take() {
                match str0m::change::SdpAnswer::from_sdp_string(&sdp) {
                    Ok(answer) => {
                        if let Err(e) = rtc.sdp_api().accept_answer(pending, answer) {
                            tracing::warn!("failed to accept answer: {e}");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("failed to parse SDP answer: {e}");
                    }
                }
            }
        }

        ClientMessage::IceCandidate { candidate, .. } => {
            if let Ok(cand) = Candidate::from_sdp_string(&candidate) {
                rtc.add_remote_candidate(cand);
            }
        }

        ClientMessage::SubscribeTrack { stream_id } => {
            // Add a send-only video track
            let mut change = rtc.sdp_api();
            let mid = change.add_media(
                MediaKind::Video,
                Direction::SendOnly,
                Some(stream_id.clone()),
                None,
                None,
            );

            // Apply changes → generate SDP offer
            if let Some((offer, pending)) = change.apply() {
                *pending_offer = Some(pending);
                let _ = signal_tx
                    .send(ServerMessage::Offer {
                        sdp: offer.to_string(),
                    })
                    .await;
            }

            tracks.insert(stream_id.clone(), mid);

            // Subscribe to pipeline frames
            if let Some(rx) = subscribe_fn(&stream_id) {
                frame_receivers.insert(stream_id.clone(), rx);
            }

            let _ = signal_tx
                .send(ServerMessage::TrackAdded {
                    stream_id,
                    mid: mid.to_string(),
                })
                .await;
        }

        ClientMessage::UnsubscribeTrack { stream_id } => {
            tracks.remove(&stream_id);
            frame_receivers.remove(&stream_id);
            let _ = signal_tx
                .send(ServerMessage::TrackRemoved { stream_id })
                .await;
        }
    }
}

async fn handle_rtc_event(event: &Event, _signal_tx: &mpsc::Sender<ServerMessage>) {
    match event {
        Event::Connected => {
            tracing::info!("WebRTC peer connected");
        }
        Event::IceConnectionStateChange(state) => {
            tracing::info!(?state, "ICE connection state changed");
        }
        Event::MediaAdded(media) => {
            tracing::info!(mid = %media.mid, "media track added in str0m");
        }
        _ => {}
    }
}
