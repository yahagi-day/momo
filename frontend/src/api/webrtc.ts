/// WebRTC preview stream client.
///
/// Manages a single RTCPeerConnection and signaling WebSocket to `/ws/preview`.
/// Server sends Offer after SubscribeTrack; client responds with Answer.

type TrackCallback = (streamId: string, stream: MediaStream | null) => void;

interface SignalMessage {
  type: string;
  sdp?: string;
  candidate?: string;
  sdp_mid?: string | null;
  sdp_m_line_index?: number | null;
  stream_id?: string;
  mid?: string;
  message?: string;
}

export class PreviewStream {
  private pc: RTCPeerConnection | null = null;
  private ws: WebSocket | null = null;
  private closed = false;
  private onTrack: TrackCallback;
  private midToStreamId = new Map<string, string>();
  private subscribedStreams = new Set<string>();
  private pendingSubscriptions = new Set<string>();
  private connected = false;

  constructor(onTrack: TrackCallback) {
    this.onTrack = onTrack;
  }

  connect(): void {
    if (this.closed) return;
    this.setupWebSocket();
  }

  subscribe(streamId: string): void {
    if (this.subscribedStreams.has(streamId)) return;
    this.subscribedStreams.add(streamId);

    if (this.connected && this.ws?.readyState === WebSocket.OPEN) {
      this.sendSignal({ type: 'SubscribeTrack', stream_id: streamId });
    } else {
      this.pendingSubscriptions.add(streamId);
    }
  }

  unsubscribe(streamId: string): void {
    this.subscribedStreams.delete(streamId);
    this.pendingSubscriptions.delete(streamId);

    if (this.connected && this.ws?.readyState === WebSocket.OPEN) {
      this.sendSignal({ type: 'UnsubscribeTrack', stream_id: streamId });
    }

    this.onTrack(streamId, null);
  }

  destroy(): void {
    this.closed = true;
    this.subscribedStreams.clear();
    this.pendingSubscriptions.clear();
    this.midToStreamId.clear();
    this.pc?.close();
    this.pc = null;
    this.ws?.close();
    this.ws = null;
  }

  static isSupported(): boolean {
    return typeof RTCPeerConnection !== 'undefined';
  }

  private setupWebSocket(): void {
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${location.host}/ws/preview`;

    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      this.connected = true;
      this.setupPeerConnection();
      for (const streamId of this.pendingSubscriptions) {
        this.sendSignal({ type: 'SubscribeTrack', stream_id: streamId });
      }
      this.pendingSubscriptions.clear();
    };

    this.ws.onmessage = (e) => {
      try {
        const msg: SignalMessage = JSON.parse(e.data);
        this.handleSignalMessage(msg);
      } catch {
        // ignore
      }
    };

    this.ws.onclose = () => {
      this.connected = false;
      if (!this.closed) {
        setTimeout(() => this.setupWebSocket(), 2000);
      }
    };

    this.ws.onerror = () => {
      this.ws?.close();
    };
  }

  private setupPeerConnection(): void {
    this.pc?.close();
    this.pc = new RTCPeerConnection({ iceServers: [] });

    this.pc.ontrack = (event) => {
      const mid = event.transceiver.mid;
      if (mid) {
        const streamId = this.midToStreamId.get(mid);
        if (streamId && event.streams[0]) {
          this.onTrack(streamId, event.streams[0]);
        }
      }
    };

    this.pc.onicecandidate = (event) => {
      if (event.candidate) {
        this.sendSignal({
          type: 'IceCandidate',
          candidate: event.candidate.candidate,
          sdp_mid: event.candidate.sdpMid,
          sdp_m_line_index: event.candidate.sdpMLineIndex,
        });
      }
    };

    this.pc.oniceconnectionstatechange = () => {
      if (this.pc?.iceConnectionState === 'failed' || this.pc?.iceConnectionState === 'disconnected') {
        this.pc?.close();
        this.pc = null;
        if (!this.closed) {
          this.ws?.close();
        }
      }
    };
  }

  private async handleSignalMessage(msg: SignalMessage): Promise<void> {
    if (!this.pc) return;

    switch (msg.type) {
      // Server sends Offer after adding tracks
      case 'Offer':
        if (msg.sdp) {
          try {
            await this.pc.setRemoteDescription({ type: 'offer', sdp: msg.sdp });
            const answer = await this.pc.createAnswer();
            await this.pc.setLocalDescription(answer);
            this.sendSignal({ type: 'Answer', sdp: answer.sdp! });
          } catch (e) {
            console.warn('Failed to handle offer:', e);
          }
        }
        break;

      case 'IceCandidate':
        if (msg.candidate) {
          try {
            await this.pc.addIceCandidate({
              candidate: msg.candidate,
              sdpMid: msg.sdp_mid ?? undefined,
              sdpMLineIndex: msg.sdp_m_line_index ?? undefined,
            });
          } catch (e) {
            console.warn('Failed to add ICE candidate:', e);
          }
        }
        break;

      case 'TrackAdded':
        if (msg.stream_id && msg.mid) {
          this.midToStreamId.set(msg.mid, msg.stream_id);
        }
        break;

      case 'TrackRemoved':
        if (msg.stream_id) {
          for (const [mid, sid] of this.midToStreamId) {
            if (sid === msg.stream_id) {
              this.midToStreamId.delete(mid);
              break;
            }
          }
          this.onTrack(msg.stream_id, null);
        }
        break;

      case 'Error':
        console.warn('WebRTC signal error:', msg.message);
        break;
    }
  }

  private sendSignal(msg: SignalMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }
}
