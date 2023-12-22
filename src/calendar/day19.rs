#![allow(clippy::unwrap_used, clippy::significant_drop_tightening)]

use anyhow::anyhow;
use axum::{
    extract::{
        ws::{Message::Text, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::Response,
    routing::{get, post},
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, RwLock,
    },
};
use tokio::sync::broadcast::{self, Sender};
use tracing::{info, warn};

use super::AppError;

pub fn router() -> Router {
    let ping_pong_state = PingPongAppState::new();
    let twitter_state = BirdAppState::new();
    let ping_pong_game = Router::new()
        .route("/ws/ping", get(ping))
        .with_state(ping_pong_state);
    let tweet = Router::new()
        .route("/views", get(views))
        .route("/reset", post(reset))
        .route("/ws/room/:room_id/user/:user", get(tweet))
        .with_state(twitter_state);

    Router::new().nest("/19", ping_pong_game).nest("/19", tweet)
}

#[axum::debug_handler]
async fn ping(ws: WebSocketUpgrade, State(state): State<PingPongAppState>) -> Response {
    dbg!("ping");
    ws.on_upgrade(move |socket| handle_ping(socket, state))
}

async fn handle_ping(mut socket: WebSocket, state: PingPongAppState) {
    while let Some(msg) = socket.recv().await {
        let Ok(msg) = msg else {
            return;
        };

        dbg!(&msg);
        if let Text(msg) = &msg {
            match msg.as_str() {
                "serve" => state.started.store(true, Ordering::Relaxed),
                "ping" => {
                    if state.started.load(Ordering::Relaxed) {
                        socket
                            .send(Text("pong".to_string()))
                            .await
                            .expect("Could not send message");
                    }
                }
                _ => {}
            }
        }
    }
}

async fn reset(State(state): State<BirdAppState>) -> Result<(), AppError> {
    let _ = state.views.swap(0, Ordering::Relaxed);

    Ok(())
}

async fn views(State(state): State<BirdAppState>) -> String {
    state.views.load(Ordering::Relaxed).to_string()
}

type RoomId = i32;

#[derive(Clone, Debug)]
struct PingPongAppState {
    started: Arc<AtomicBool>,
}

impl PingPongAppState {
    fn new() -> Self {
        Self {
            started: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Clone, Debug)]
struct BirdAppState {
    views: Arc<AtomicU32>,
    rooms: Arc<RwLock<HashMap<RoomId, RoomState>>>,
}

#[derive(Debug)]
struct RoomState {
    sender: Sender<Tweet>,
}

impl RoomState {
    fn new() -> Self {
        Self {
            sender: broadcast::channel(100).0,
        }
    }
}

impl BirdAppState {
    fn new() -> Self {
        Self {
            views: Arc::new(AtomicU32::new(0)),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TweetInput {
    message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Tweet {
    user: String,
    message: TweetInput,
}

impl TryFrom<&String> for TweetInput {
    type Error = AppError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let tweet_input = serde_json::from_str::<Self>(value)
            .map_err(|e| anyhow!("Error parsing TweetInput: {}", e))?;

        if tweet_input.message.len() > 128 {
            return Err(anyhow!("Message length cannot be over 128").into());
        }

        Ok(tweet_input)
    }
}

impl From<Tweet> for String {
    fn from(value: Tweet) -> Self {
        format!(
            r#"{{
      "user": "{}",
      "message": "{}"
    }}"#,
            value.user, value.message.message
        )
    }
}

async fn tweet(
    ws: WebSocketUpgrade,
    Path((room, user)): Path<(i32, String)>,
    State(state): State<BirdAppState>,
) -> Response {
    ws.on_upgrade(move |c| handle_tweet(c, room, user, Arc::new(state)))
}

async fn handle_tweet(socket: WebSocket, room: i32, user: String, state: Arc<BirdAppState>) {
    let (mut sender, mut receiver) = socket.split();
    #[allow(unused_assignments)]
    let mut room_sender = None::<Sender<Tweet>>;

    if let Some(room_state) = state.rooms.read().unwrap().get(&room) {
        room_sender = Some(room_state.sender.clone());
    }

    if room_sender.is_none() {
        let mut rooms = state.rooms.write().unwrap();
        let room_state = rooms.entry(room).or_insert_with(RoomState::new);
        room_sender = Some(room_state.sender.clone());
    }

    let room_sender = room_sender.unwrap();
    let mut room_receiver = room_sender.subscribe();

    let mut send = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            let Ok(msg) = msg else {
                return;
            };

            if let Text(text) = &msg {
                match TweetInput::try_from(text) {
                    Ok(message) => {
                        info!("Parsed {:?}", message);
                        let user = user.clone();
                        let _ = room_sender.send(Tweet { user, message }).unwrap();
                    }
                    Err(e) => warn!("Failed to parse TweetInput: {:?}", e),
                }
            }
        }
    });

    let mut receive = tokio::spawn(async move {
        while let Ok(msg) = room_receiver.recv().await {
            let msg_str: String = msg.into();
            let _ = state.views.fetch_add(1, Ordering::Relaxed);
            sender.send(Text(msg_str)).await.unwrap();
        }
    });

    tokio::select! {
        _ = (&mut send) => receive.abort(),
        _ = (&mut receive) => send.abort(),
    };
}
