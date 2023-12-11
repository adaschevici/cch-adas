use axum::{
    extract::Json,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};

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

async fn elfcount(text: String) -> Response {
    let mut elves = 0usize;
    let mut shelves = 0usize;
    let mut elves_on_shelves = 0usize;

    text.split("elf").for_each(|s| {
        elves += 1;
        if s.ends_with("sh") {
            shelves += 1;
        }
        if s.trim() == "on a sh" {
            elves_on_shelves += 1;
        }
    });

    Json(Day6Response {
        elf: elves - 1, // Counts one extra.
        elf_on_a_shelf: elves_on_shelves,
        shelf: shelves - elves_on_shelves,
    })
    .into_response()
}
