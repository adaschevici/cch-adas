use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;

pub fn router() -> Router {
    Router::new().route("/19/ws/ping", get(play_ping_pong))
}

async fn play_ping_pong(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket))
}

async fn handle_socket(mut socket: WebSocket) {
    let mut game_started = false;

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("an error occured while receiving a message: {}", e);
                break;
            }
        };

        if !game_started {
            if msg != Message::Text("serve".to_string()) {
                eprintln!("ignoring message, got: {:?}", msg);
                continue;
            }

            game_started = true;
            continue;
        }

        let msg = match msg {
            Message::Text(msg) => msg,
            _ => {
                eprintln!("expected text message, got: {:?}", msg);
                break;
            }
        };

        let msg = match msg.as_str() {
            "ping" => "pong".to_string(),
            _ => {
                eprintln!("expected 'ping' or 'pong', got: {:?}", msg);
                continue;
            }
        };

        if let Err(e) = socket.send(Message::Text(msg)).await {
            eprintln!("an error occured while sending a message: {}", e);
            break;
        }
    }
    // let (mut tx, mut rx) = socket.split();
    //
    // while let Some(Ok(Message::Text(text))) = rx.next().await {
    //     dbg!(addr, &text);
    //     tx.send(Message::Text(text)).await.unwrap();
    // }
}
