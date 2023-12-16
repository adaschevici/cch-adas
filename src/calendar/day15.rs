use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fmt::Write;

#[derive(Deserialize, Debug, Serialize)]
struct Password {
    input: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct PasswordValidation {
    result: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/15/nice", post(validate_password))
        .route("/15/game", post(password_validation_game))
}

async fn contains_double_letter(input: &str) -> bool {
    let mut chars = input.chars();
    let mut last_char = chars.next().unwrap();
    for c in chars {
        if c == last_char && c.is_alphabetic() {
            return true;
        }
        last_char = c;
    }
    false
}

async fn contains_three_vowels(input: &str) -> bool {
    input.chars().filter(|c| "aeiou".contains(*c)).count() >= 3
}

async fn does_not_contain_forbidden_strings(input: &str) -> bool {
    !input.contains("ab") && !input.contains("cd") && !input.contains("pq") && !input.contains("xy")
}

struct AppError(JsonRejection);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": format!("Something went wrong: {}", self.0)})),
        )
            .into_response()
    }
}

async fn validate_password(
    payload: Result<Json<Password>, JsonRejection>,
) -> Result<(StatusCode, String), AppError> {
    match payload {
        Ok(payload) => {
            let mut errors = Vec::new();
            if !contains_double_letter(&payload.input).await {
                errors.push("Password does not contain a double letter".to_string());
            }
            if !contains_three_vowels(&payload.input).await {
                errors.push("Password does not contain three vowels".to_string());
            }
            if !does_not_contain_forbidden_strings(&payload.input).await {
                errors.push("Password contains forbidden strings".to_string());
            }
            if errors.is_empty() {
                Ok((
                    StatusCode::OK,
                    json!({"result": "nice".to_string()}).to_string(),
                ))
            } else {
                Ok((
                    StatusCode::BAD_REQUEST,
                    json!({"result": "naughty"}).to_string(),
                ))
            }
        }
        Err(e) => Err(AppError(e)),
    }
}

async fn is_at_least_eight_chars_long(input: &str) -> bool {
    input.len() >= 8
}

async fn contains_all_types_alphanumeric(input: &str) -> bool {
    input.chars().any(|c| c.is_numeric())
        && input.chars().any(|c| c.is_lowercase())
        && input.chars().any(|c| c.is_uppercase())
}

async fn contains_at_least_five_digits(input: &str) -> bool {
    input.chars().filter(|c| c.is_numeric()).count() >= 5
}

async fn sequence_arithmetic(input: &str) -> bool {
    Regex::new(r"\d+")
        .unwrap()
        .find_iter(input)
        .map(|m| m.as_str().parse::<i32>().unwrap())
        .sum::<i32>()
        == 2023
}

async fn is_joyful(input: &str) -> bool {
    let mut extracted = String::new();
    let joy = "joy".chars();
    let mut joy_iter = joy.into_iter();

    let mut current_char = joy_iter.next();

    for c in input.chars() {
        if extracted.contains(c) {
            return false;
        }
        if Some(c) == current_char {
            extracted.push(c);
            current_char = joy_iter.next();
        }
        if current_char.is_none() {
            joy_iter = "joy".chars().into_iter();
            current_char = joy_iter.next();
        }
    }

    extracted == "joy"
}

async fn has_sandwich(input: &str) -> bool {
    let re = Regex::new(r"\p{L}{3}").unwrap();
    let matches = re.find_iter(input);
    for m in matches {
        let chars = m.as_str().chars().collect::<Vec<_>>();
        if chars[0] == chars[2] {
            return true;
        }
    }
    false
}

async fn string_contains_char_in_range(input: &str) -> bool {
    let re = Regex::new(r"[\u{2980}-\u{2BFF}]").unwrap();
    re.is_match(input)
}

async fn string_contains_at_least_one_emoji(input: &str) -> bool {
    let emoji_pattern = r"\p{Extended_Pictographic}";
    let re = Regex::new(emoji_pattern).unwrap();
    re.is_match(input)
}

async fn sha256_ends_with_a(input: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    let mut hex_str = String::new();
    for byte in result {
        write!(&mut hex_str, "{:02x}", byte).expect("Unable to write to string");
    }
    hex_str.chars().last().unwrap() == 'a'
}

async fn password_validation_game(
    payload: Result<Json<Password>, JsonRejection>,
) -> Result<(StatusCode, String), AppError> {
    match payload {
        Ok(payload) => {
            let mut errors = Vec::new();
            if !is_at_least_eight_chars_long(&payload.input).await {
                errors.push((
                    StatusCode::BAD_REQUEST,
                    json!({"result": "naughty", "reason": "8 chars"}).to_string(),
                ));
            }
            if !contains_all_types_alphanumeric(&payload.input).await {
                errors.push((
                    StatusCode::BAD_REQUEST,
                    json!({"result": "naughty", "reason": "more types of chars"}).to_string(),
                ));
            }
            if !contains_at_least_five_digits(&payload.input).await {
                errors.push((
                    StatusCode::BAD_REQUEST,
                    json!({"result": "naughty", "reason": "55555"}).to_string(),
                ));
            }
            if !sequence_arithmetic(&payload.input).await {
                errors.push((
                    StatusCode::BAD_REQUEST,
                    json!({"result": "naughty", "reason": "math is hard"}).to_string(),
                ));
            }
            if !is_joyful(&payload.input).await {
                errors.push((
                    StatusCode::NOT_ACCEPTABLE,
                    json!({"result": "naughty", "reason": "not joyful enough"}).to_string(),
                ));
            }
            if !has_sandwich(&payload.input).await {
                errors.push((
                    StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
                    json!({"result": "naughty", "reason": "illegal: no sandwich"}).to_string(),
                ));
            }
            if !string_contains_char_in_range(&payload.input).await {
                errors.push((
                    StatusCode::RANGE_NOT_SATISFIABLE,
                    json!({"result": "naughty", "reason": "outranged"}).to_string(),
                ));
            }
            if !string_contains_at_least_one_emoji(&payload.input).await {
                errors.push((
                    StatusCode::UPGRADE_REQUIRED,
                    json!({"result": "naughty", "reason": "😳"}).to_string(),
                ));
            }
            if !sha256_ends_with_a(&payload.input).await {
                errors.push((
                    StatusCode::IM_A_TEAPOT,
                    json!({"result": "naughty", "reason": "not a coffee brewer"}).to_string(),
                ));
            }
            if errors.is_empty() {
                return Ok((
                    StatusCode::OK,
                    json!({"result": "nice".to_string(), "reason": "that's a nice password"})
                        .to_string(),
                ));
            } else {
                let result = errors.first().unwrap().clone();
                return Ok(result);
            }
        }
        Err(e) => Err(AppError(e)),
    }
}
