use axum::Router;
// use shuttle_runtime::CustomError;
use sqlx::PgPool;

mod calendar;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    let router = Router::new().nest("/", calendar::router(pool.clone()));

    Ok(router.into())
}
