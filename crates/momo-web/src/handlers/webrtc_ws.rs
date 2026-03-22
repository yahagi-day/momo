//! WebRTC signaling WebSocket handler (`/ws/preview`).

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;

use momo_webrtc::signal::{ClientMessage, ServerMessage};

use crate::state::AppState;

pub async fn ws_preview_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_preview_ws(socket, state))
}

async fn handle_preview_ws(mut socket: WebSocket, state: AppState) {
    let mut session = state.webrtc_manager.create_session();
    tracing::info!("WebRTC preview WebSocket connected");

    loop {
        tokio::select! {
            // Forward server messages to WebSocket
            msg = session.signal_rx.recv() => {
                let Some(msg) = msg else { break };
                if let Ok(json) = serde_json::to_string(&msg) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }

            // Forward WebSocket messages to session
            ws_msg = socket.recv() => {
                let Some(Ok(ws_msg)) = ws_msg else { break };
                let text = match ws_msg {
                    Message::Text(t) => t.to_string(),
                    Message::Close(_) => break,
                    _ => continue,
                };

                let client_msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        let err = ServerMessage::Error {
                            message: format!("invalid message: {e}"),
                        };
                        if let Ok(json) = serde_json::to_string(&err) {
                            let _ = socket.send(Message::Text(json.into())).await;
                        }
                        continue;
                    }
                };

                if session.signal_tx.send(client_msg).await.is_err() {
                    break;
                }
            }
        }
    }

    tracing::info!("WebRTC preview WebSocket disconnected");
}
