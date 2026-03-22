use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Deserialize)]
struct WsCommand {
    command: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = {
        let pipeline = state.pipeline.read().await;
        pipeline.subscribe()
    };

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let json = match serde_json::to_string(&event) {
                            Ok(j) => j,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<WsCommand>(&text) {
                            handle_command(&cmd.command, &state).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

async fn handle_command(command: &str, state: &AppState) {
    match command {
        "stop" => {
            let mut pipeline = state.pipeline.write().await;
            if let Err(e) = pipeline.stop() {
                tracing::warn!("WebSocket stop command failed: {e}");
            }
        }
        "start" => {
            let mut pipeline = state.pipeline.write().await;
            if let Err(e) = pipeline.start() {
                tracing::warn!("WebSocket start command failed: {e}");
            }
        }
        _ => {
            tracing::debug!("Unknown WebSocket command: {command}");
        }
    }
}
