use axum::{extract::Path, routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/1/*tail", get(cch_1))
}

async fn cch_1(Path(tail): Path<String>) -> String {
    let params: Vec<i64> = tail
        .split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect();
    let response = params
        .iter()
        .fold(0i64, |mut xored, val| {
            xored ^= val;
            xored
        })
        .pow(3);
    response.to_string()
}
