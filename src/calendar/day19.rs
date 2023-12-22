use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::broadcast;

#[derive(Clone)]
struct GameState {
    started: bool,
}

#[derive(Clone, Debug)]
struct TwitterState {
    tweet_count: u64,
    rooms: HashMap<i64, broadcast::Sender<String>>,
}

impl TwitterState {
    fn new() -> Self {
        Self {
            tweet_count: 0,
            rooms: HashMap::new(),
        }
    }

    fn reset(&mut self) {
        self.tweet_count = 0;
        self.rooms.clear();
    }

    fn increment(&mut self) {
        self.tweet_count += 1;
    }
}
#[derive(Clone)]
struct FeedState {
    tweets: Arc<RwLock<TwitterState>>,
    conn: TweetParams,
}

#[derive(Deserialize, Clone)]
struct TweetParams {
    room: i64,
    name: String,
}

pub fn router() -> Router {
    let ping_pong_game = Router::new()
        .route("/ws/ping", get(play_ping_pong))
        .with_state(GameState { started: false });

    let twitter = Router::new()
        .route("/reset", post(reset_tweet_metrics))
        .route("/views", get(get(views_count)))
        .route("/ws/room/:id/user/:name", get(run_twitter))
        .with_state(Arc::new(RwLock::new(TwitterState::new())));
    Router::new()
        .nest("/19", ping_pong_game)
        .nest("/19", twitter)
}

async fn play_ping_pong(ws: WebSocketUpgrade, State(state): State<GameState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ping_socket(socket, state))
}

async fn handle_ping_socket(mut socket: WebSocket, mut state: GameState) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("an error occured while receiving a message: {}", e);
                break;
            }
        };

        if !state.started {
            if msg != Message::Text("serve".to_string()) {
                eprintln!("ignoring message, got: {:?}", msg);
                continue;
            }

            state.started = true;
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
                eprintln!("expected 'ping' got: {:?}", msg);
                continue;
            }
        };

        if let Err(e) = socket.send(Message::Text(msg)).await {
            eprintln!("an error occured while sending a message: {}", e);
            break;
        }
    }
}

async fn reset_tweet_metrics(State(twitter): State<Arc<RwLock<TwitterState>>>) -> StatusCode {
    twitter
        .write()
        .expect("Could not acquire write lock")
        .reset();
    StatusCode::OK
}

async fn views_count(State(twitter): State<Arc<RwLock<TwitterState>>>) -> String {
    let twitter = twitter.read().unwrap();
    dbg!(&twitter);
    format!("{}", twitter.tweet_count)
}

async fn run_twitter(
    ws: WebSocketUpgrade,
    Path((id, name)): Path<(i64, String)>,
    State(twitter): State<Arc<RwLock<TwitterState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        handle_twitter_socket(
            socket,
            FeedState {
                tweets: twitter,
                conn: TweetParams { room: id, name },
            },
        )
    })
}

async fn handle_twitter_socket(socket: WebSocket, state: FeedState) {
    #[derive(Deserialize)]
    struct Msg {
        message: String,
    }

    let tweets = state.tweets;

    let (mut send, mut recv) = socket.split();

    let out = tweets
        .write()
        .expect("could not establish write lock on data")
        .rooms
        .entry(state.conn.room)
        .or_insert(broadcast::channel(100).0)
        .clone();

    let mut sub = out.subscribe();

    let mut send_tweet = tokio::spawn(async move {
        while let Ok(msg) = sub.recv().await {
            tweets
                .write()
                .expect("could not establish write lock on data")
                .increment();
            if send.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let mut recv_tweet = {
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(msg))) = recv.next().await {
                let msg: Msg = serde_json::from_str(msg.as_str())
                    .expect("could not deserialize input message");
                if msg.message.chars().count() > 128 {
                    continue;
                }
                let msg = json!({"user":state.conn.name.clone(),"message":msg.message}).to_string();
                let _ = out.send(msg);
            }
        })
    };

    tokio::select! {
        _ = (&mut send_tweet) => recv_tweet.abort(),
        _ = (&mut recv_tweet) => send_tweet.abort(),
    };
}
