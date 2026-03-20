import type { PipelineEvent } from './types';

export interface WsConnection {
  send: (command: string) => void;
  cleanup: () => void;
}

export function connectWebSocket(onEvent: (event: PipelineEvent) => void): WsConnection {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  const url = `${protocol}//${location.host}/ws/status`;

  let ws: WebSocket | null = null;
  let closed = false;

  function connect() {
    if (closed) return;
    ws = new WebSocket(url);

    ws.onmessage = (e) => {
      try {
        const event: PipelineEvent = JSON.parse(e.data);
        onEvent(event);
      } catch {
        // ignore parse errors
      }
    };

    ws.onclose = () => {
      if (!closed) {
        setTimeout(connect, 2000);
      }
    };

    ws.onerror = () => {
      ws?.close();
    };
  }

  connect();

  return {
    send: (command: string) => {
      if (ws?.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ command }));
      }
    },
    cleanup: () => {
      closed = true;
      ws?.close();
    },
  };
}
