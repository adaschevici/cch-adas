use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RegionTopN {
    region: String,
    top_gifts: Vec<String>,
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
    fn sort_strings(strings: Vec<(String, i64)>) -> Vec<String> {
        let mut sorted_strings = strings.clone();
        sorted_strings.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        sorted_strings.iter().map(|(s, _)| s.clone()).collect()
    }
    let mut tx = app.pool.begin().await.unwrap();
    let rows = sqlx::query(
        r#"
        SELECT regions.name, orders.gift_name, SUM(orders.quantity) AS total
        FROM orders
        INNER JOIN regions ON regions.id = orders.region_id
        GROUP BY regions.name, orders.gift_name, orders.region_id
        ORDER BY total DESC
        "#,
    )
    .fetch_all(&mut *tx)
    .await
    .unwrap();
    let region_names = sqlx::query(
        r#"
        SELECT regions.name
        FROM regions
        "#,
    )
    .fetch_all(&app.pool)
    .await
    .unwrap();

    // let mut totals = Vec::<RegionTopN>::new();
    let mut totals = HashMap::<String, Vec<(String, i64)>>::new();
    for row in rows {
        if number == 0 {
            break;
        }
        let name: String = row.get("name");
        let top_gift: (String, i64) = (row.get("gift_name"), row.get("total"));
        if !totals.contains_key(&name) {
            totals.insert(name.clone(), vec![top_gift.clone()]);
        } else {
            let mut top_gifts = totals.get_mut(&name).unwrap().clone();
            top_gifts.push(top_gift.clone());
            if top_gifts.len() > number as usize {
                continue;
            }
            totals.insert(name.clone(), top_gifts);
        }
    }

    let mut top_n = Vec::<RegionTopN>::new();
    for (region, gifts) in totals.clone() {
        let sorted_gifts = sort_strings(gifts);
        top_n.push(RegionTopN {
            region,
            top_gifts: sorted_gifts,
        });
    }
    for region in region_names {
        if !&totals.contains_key(&region.get::<String, _>("name")) {
            top_n.push(RegionTopN {
                region: region.get("name"),
                top_gifts: vec![],
            });
        }
    }
    top_n.sort_by(|topa, topb| topa.region.cmp(&topb.region));
    dbg!(&top_n);

    Json(top_n).into_response()
}
