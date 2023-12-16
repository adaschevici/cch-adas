use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool, Row};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct AppState {
    pool: PgPool,
}

pub fn router(pool: PgPool) -> Router {
    let shared_state = Arc::new(AppState { pool });
    Router::new()
        .route("/13/sql", get(get_const_from_db))
        .route("/13/reset", post(reset_db))
        .route("/13/orders", post(create_orders))
        .route("/13/orders/total", get(get_order_totals))
        .route("/13/orders/popular", get(get_popular_gifts))
        .with_state(shared_state)
}

async fn get_const_from_db(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let row = sqlx::query("SELECT 20231213 number")
        .fetch_one(&state.pool)
        .await
        .unwrap()
        .get::<i32, _>("number");

    row.to_string().into_response()
}

async fn reset_db(State(state): State<Arc<AppState>>) -> StatusCode {
    sqlx::query("DROP TABLE IF EXISTS orders")
        .execute(&state.pool)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE orders (
          id INT PRIMARY KEY,
          region_id INT,
          gift_name VARCHAR(50),
          quantity INT
        );",
    )
    .execute(&state.pool)
    .await
    .unwrap();

    StatusCode::OK
}

#[derive(Serialize, Deserialize)]
struct Order {
    id: i32,
    region_id: i32,
    gift_name: String,
    quantity: i32,
}

async fn create_orders(
    State(state): State<Arc<AppState>>,
    Json(orders): Json<Vec<Order>>,
) -> StatusCode {
    for order in orders {
        sqlx::query(
            "INSERT INTO orders (id, region_id, gift_name, quantity)
            VALUES ($1, $2, $3, $4)",
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(order.gift_name)
        .bind(order.quantity)
        .execute(&state.pool)
        .await
        .unwrap();
    }

    StatusCode::OK
}

#[derive(Serialize, Deserialize, Debug)]
struct Total {
    total: i64,
}

async fn get_order_totals(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let total = sqlx::query(
        "SELECT SUM(quantity) AS total
        FROM orders",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap()
    .get::<i64, _>("total");

    Json(Total { total }).into_response()
}

async fn get_popular_gifts(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    #[derive(FromRow, Serialize, Deserialize, Debug)]
    struct Gift {
        #[serde(rename = "popular")]
        gift_name: String,
    }

    let gifts = sqlx::query_as::<_, Gift>(
        "SELECT gift_name, SUM(quantity) AS quantity
        FROM orders
        GROUP BY gift_name
        ORDER BY quantity DESC
        LIMIT 1",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();

    if gifts.is_empty() {
        return Json(json!({ "popular": null })).into_response();
    }
    Json(&gifts[0]).into_response()
}
