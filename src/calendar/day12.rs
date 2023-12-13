use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ulid::Ulid;
use uuid::Uuid;

type AppData = HashMap<String, u64>;

#[derive(Debug, Serialize, Deserialize)]
struct AppState {
    ctx: Mutex<AppData>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct TimeClassifier {
    #[serde(rename = "christmas eve")]
    christmas_eve: usize,
    weekday: usize,
    #[serde(rename = "in the future")]
    in_the_future: usize,
    #[serde(rename = "LSB is 1")]
    lsb_one: usize,
}

pub fn router() -> Router {
    let shared_state = Arc::new(AppState {
        ctx: Mutex::new(HashMap::new()),
    });
    Router::new()
        .route("/12/save/:pkg_id", post(set_time))
        .route("/12/load/:pkg_id", get(get_time))
        .route("/12/ulids", post(convert_ulids_to_uuids))
        .route("/12/ulids/:weekday", post(organize_ulids_by_weekday))
        .with_state(shared_state)
}

async fn set_time(
    Path(pkg_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let current_time = Utc::now();
    state
        .ctx
        .lock()
        .unwrap()
        .insert(pkg_id, current_time.timestamp() as u64);
    let response = state.ctx.lock().unwrap().clone();
    Json(response).into_response()
}

async fn get_time(
    Path(pkg_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let current_time = Utc::now();
    let time_diff =
        current_time.timestamp() as u64 - state.ctx.lock().unwrap().get(&pkg_id).unwrap().clone(); // - current_time.timestamp() as u64;

    time_diff.to_string().into_response()
}

async fn convert_ulids_to_uuids(Json(ulids): Json<Vec<String>>) -> impl IntoResponse {
    let ulids_converted = ulids
        .iter()
        .map(|x| Ulid::from_string(x))
        .map(|r| r.unwrap())
        .map(|u| u.to_bytes())
        .map(|b| Uuid::from_bytes(b))
        .map(|u| format!("{}", u.hyphenated()))
        .rev()
        .collect::<Vec<_>>();
    Json(ulids_converted).into_response()
}

async fn organize_ulids_by_weekday(
    Path(weekday): Path<String>,
    Json(ulids): Json<Vec<String>>,
) -> impl IntoResponse {
    let days = ulids
        .iter()
        .map(|x| Ulid::from_string(x))
        .map(|r| r.unwrap())
        .fold(TimeClassifier::default(), |mut acc, u| {
            let millis = u.datetime();
            let dt: DateTime<Utc> = millis.into();
            if (dt.weekday().number_from_monday() - 1) as usize == weekday.parse::<usize>().unwrap()
            {
                acc.weekday += 1;
            }
            if dt > Utc::now() {
                acc.in_the_future += 1;
            }
            if dt.day() == 24 && dt.month() == 12 {
                acc.christmas_eve += 1;
            }
            if u.0 & 0b1u128 == 1 {
                acc.lsb_one += 1;
            }
            acc
        });
    Json(days).into_response()
}
