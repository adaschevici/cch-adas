use axum::{
    extract::Json,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};

use regex::Regex;
use serde::{Deserialize, Serialize};

pub fn router() -> Router {
    Router::new().route("/6", post(elfcount))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Day6Response {
    #[serde(rename = "elf")]
    elf: usize,
    #[serde(rename = "elf on a shelf")]
    elf_on_a_shelf: usize,
    #[serde(rename = "shelf with no elf on it")]
    shelf: usize,
}

async fn count_not_on_a_shelf(s: &str, substr_regex: Regex) -> usize {
    substr_regex
        .captures_iter(s)
        .filter(|caps| caps.get(1).is_none())
        .count()
}

async fn elfcount(text: String) -> Response {
    let elf_regex = Regex::new(r"elf").unwrap();
    let elf_on_a_shelf_regex = Regex::new(r"elf on a shelf").unwrap();
    let shelf_with_no_elf_regex = Regex::new(r#"(elf on a )?shelf"#).unwrap();
    let response = Day6Response {
        elf: count_not_on_a_shelf(&text, elf_regex).await,
        elf_on_a_shelf: count_not_on_a_shelf(&text, elf_on_a_shelf_regex).await,
        shelf: count_not_on_a_shelf(&text, shelf_with_no_elf_regex).await,
    };
    Json(response).into_response()
}
