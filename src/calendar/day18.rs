use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Serialize, Deserialize)]
struct Order {
    id: i32,
    region_id: i32,
    gift_name: String,
    quantity: i32,
}

#[derive(Serialize, Deserialize)]
struct Region {
    id: i32,
    name: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct RegionTotal {
    region: String,
    total: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
struct RegionTopN {
    region: String,
    #[serde(rename = "top_gifts")]
    region_top_n: Vec<String>,
}

pub fn router(pool: PgPool) -> Router {
    let shared_state = Arc::new(AppState { pool });
    Router::new()
        .route("/18/reset", post(reset_db))
        .route("/18/orders", post(create_orders))
        .route("/18/regions", post(create_regions))
        .route("/18/regions/total", get(get_totals_by_region))
        .route(
            "/18/regions/top_list/:number",
            get(get_top_n_list_by_region),
        )
        .with_state(shared_state)
}

async fn reset_db(State(app): State<Arc<AppState>>) -> impl IntoResponse {
    sqlx::query("DROP TABLE IF EXISTS orders;")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query("DROP TABLE IF EXISTS regions;")
        .execute(&app.pool)
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE regions (
            id INT PRIMARY KEY,
            name VARCHAR(50)
        );",
    )
    .execute(&app.pool)
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
    .execute(&app.pool)
    .await
    .unwrap();

    StatusCode::OK
}

async fn create_orders(
    State(app): State<Arc<AppState>>,
    Json(orders): Json<Vec<Order>>,
) -> StatusCode {
    let mut tx = app.pool.begin().await.unwrap();
    for order in orders {
        sqlx::query(
            r#"
            INSERT INTO orders (id, region_id, gift_name, quantity)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(order.id)
        .bind(order.region_id)
        .bind(order.gift_name)
        .bind(order.quantity)
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    tx.commit().await.unwrap();

    StatusCode::OK
}

async fn create_regions(
    State(app): State<Arc<AppState>>,
    Json(regions): Json<Vec<Region>>,
) -> StatusCode {
    let mut tx = app.pool.begin().await.unwrap();
    for region in regions {
        sqlx::query(
            r#"
            INSERT INTO regions (id, name)
            VALUES ($1, $2)
            "#,
        )
        .bind(region.id)
        .bind(region.name)
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    tx.commit().await.unwrap();

    StatusCode::OK
}

async fn get_totals_by_region(State(app): State<Arc<AppState>>) -> impl IntoResponse {
    let mut tx = app.pool.begin().await.unwrap();
    let rows = sqlx::query(
        r#"
        SELECT regions.name, SUM(orders.quantity) AS total
        FROM orders
        INNER JOIN regions ON regions.id = orders.region_id
        GROUP BY regions.name
        ORDER BY total DESC
        "#,
    )
    .fetch_all(&mut *tx)
    .await
    .unwrap();

    let mut totals = Vec::<RegionTotal>::new();
    for row in rows {
        let name: String = row.get("name");
        let total: i64 = row.get("total");
        totals.push(RegionTotal {
            region: name,
            total,
        });
    }

    Json(totals).into_response()
}

#[axum::debug_handler]
async fn get_top_n_list_by_region(
    State(app): State<Arc<AppState>>,
    Path(number): Path<i32>,
) -> impl IntoResponse {
    let top_gifts = sqlx::query_as::<_, RegionTopN>(
        r#"
            SELECT r.name AS region,
                array_remove(array_agg(o.gift_name), NULL) AS region_top_n
            FROM regions r
            LEFT JOIN LATERAL (
                SELECT o.gift_name,
                    sum(o.quantity) AS total_quantity
                FROM orders o
                WHERE o.region_id = r.id
                GROUP BY o.gift_name
                ORDER BY total_quantity DESC,
                    o.gift_name ASC
                LIMIT $1
                ) o ON TRUE
            GROUP BY r.name
            ORDER BY r.name ASC
        "#,
    )
    .bind(number)
    .fetch_all(&app.pool)
    .await
    .unwrap();

    Json(top_gifts).into_response()
}
