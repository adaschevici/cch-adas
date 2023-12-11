use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use reqwest;

const GRAVITY: f64 = 9.825;
const CHIMNEY_HEIGHT: u64 = 10;
const POKEAPI_URL: &str = "https://pokeapi.co/api/v2/pokemon/";

pub fn router() -> Router {
    Router::new()
        .route("/8/weight/:pokeid", get(get_pokemon_weight))
        .route("/8/drop/:pokeid", get(drop_pokemon))
}
async fn get_weight(poke_id: u64) -> f64 {
    let url = format!("{}{}", POKEAPI_URL, poke_id);
    let resp = reqwest::get(&url).await.unwrap();
    let body = resp.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    json["weight"].as_f64().unwrap() / 10.0
}

async fn get_pokemon_weight(Path(params): Path<u64>) -> impl IntoResponse {
    let pokeid = params;
    let weight = get_weight(pokeid).await;
    format!("{}", weight).into_response()
}

async fn drop_pokemon(Path(params): Path<u64>) -> impl IntoResponse {
    let pokeid = params;
    let weight = get_weight(pokeid).await;
    let velocity = (2.0 * GRAVITY * CHIMNEY_HEIGHT as f64).sqrt();
    format!("{}", weight as f64 * velocity)
}
