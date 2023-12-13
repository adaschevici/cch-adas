use axum::Router;
use sqlx::PgPool;

mod day0;
mod day1;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day17;
mod day18;
mod day19;
mod day2;
mod day20;
mod day21;
mod day22;
mod day3;
mod day4;
mod day5;
mod day6;
mod day7;
mod day8;
mod day9;

pub(crate) fn router(pool: PgPool) -> Router {
    Router::new()
        .nest("/", day0::router())
        .nest("/", day1::router())
        .nest("/", day2::router())
        .nest("/", day3::router())
        .nest("/", day4::router())
        .nest("/", day5::router())
        .nest("/", day6::router())
        .nest("/", day7::router())
        .nest("/", day8::router())
        .nest("/", day9::router())
        .nest("/", day10::router())
        .nest("/", day11::router())
        .nest("/", day12::router())
        .nest("/", day13::router(pool))
        .nest("/", day14::router())
        .nest("/", day15::router())
        .nest("/", day16::router())
        .nest("/", day17::router())
        .nest("/", day18::router())
        .nest("/", day19::router())
        .nest("/", day20::router())
        .nest("/", day21::router())
        .nest("/", day22::router())
}
