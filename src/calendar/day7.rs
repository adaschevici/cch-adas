use std::collections::HashMap;

use axum::{
    async_trait,
    extract::{FromRequestParts, Json},
    http::{
        header::{HeaderValue, COOKIE},
        request::Parts,
        StatusCode,
    },
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::from_str;

pub fn router() -> Router {
    Router::new()
        .route("/7/decode", get(decode_recipe))
        .route("/7/bake", get(secret_recipe))
}

type Recipe = HashMap<String, u64>;
type Pantry = HashMap<String, u64>;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RecipeDecoder {
    recipe: Recipe,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Bakery {
    recipe: Recipe,
    pantry: Pantry,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Baked {
    cookies: u64,
    pantry: Pantry,
}

struct ExtractCookie(HeaderValue);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractCookie
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if let Some(cookie) = parts.headers.get(COOKIE) {
            Ok(ExtractCookie(cookie.clone()))
        } else {
            Err((StatusCode::BAD_REQUEST, "Missing cookie"))
        }
    }
}

async fn decode_recipe(ExtractCookie(cookie): ExtractCookie) -> Response {
    let recipe_base64_text = cookie.clone();
    let recipe_details = general_purpose::STANDARD
        .decode(
            recipe_base64_text
                .to_str()
                .unwrap()
                .split("recipe=")
                .nth(1)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
    let response: RecipeDecoder =
        from_str(String::from_utf8(recipe_details).unwrap().as_str()).unwrap();
    Json(response).into_response()
}

async fn calculate_max_cookies(recipe: &Recipe, pantry: &Pantry) -> u64 {
    let mut max_cookies = 0;
    for (ingredient, amount) in recipe.iter() {
        if amount == &0 {
            continue;
        }
        let mut max_ingredient = 0;
        if let Some(pantry_amount) = pantry.get(ingredient) {
            max_ingredient = pantry_amount / amount;
        }
        if max_cookies == 0 || max_ingredient < max_cookies {
            max_cookies = max_ingredient;
        }
    }
    max_cookies
}

async fn leftover_in_pantry(recipe: &Recipe, pantry: &Pantry, max_cookies: u64) -> Pantry {
    let mut leftover_pantry = pantry.clone();
    for (ingredient, amount) in recipe.iter() {
        if let Some(pantry_amount) = pantry.get(ingredient) {
            leftover_pantry.insert(ingredient.clone(), pantry_amount - (amount * max_cookies));
        }
    }
    leftover_pantry
}

async fn secret_recipe(ExtractCookie(cookie): ExtractCookie) -> Response {
    let recipe_base64_text = cookie.clone();
    let recipe_details = general_purpose::STANDARD
        .decode(
            recipe_base64_text
                .to_str()
                .unwrap()
                .split("recipe=")
                .nth(1)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
    let decoded_payload: Bakery =
        from_str(String::from_utf8(recipe_details).unwrap().as_str()).unwrap();
    let max_cookies = calculate_max_cookies(&decoded_payload.recipe, &decoded_payload.pantry).await;
    let leftover_pantry = leftover_in_pantry(
        &decoded_payload.recipe,
        &decoded_payload.pantry,
        max_cookies,
    )
    .await;
    let response = Baked {
        cookies: max_cookies,
        pantry: leftover_pantry,
    };
    Json(response).into_response()
}
