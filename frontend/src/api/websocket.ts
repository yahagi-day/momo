import type { PipelineEvent } from './types';

export function connectWebSocket(onEvent: (event: PipelineEvent) => void): () => void {
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

  return () => {
    closed = true;
    ws?.close();
  };
}
