use axum::{response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tera::Tera;

pub fn router() -> Router {
    Router::new()
        .route("/14/unsafe", post(unsafe_render))
        .route("/14/safe", post(safe_render))
}

#[derive(Serialize, Deserialize, Debug)]
struct UnsafePayload {
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SafePayload {
    content: String,
}

async fn unsafe_render(Json(payload): Json<UnsafePayload>) -> impl IntoResponse {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "unsafe_render",
        r#"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {{ content }}
  </body>
</html>"#,
    )
    .unwrap();
    let context = tera::Context::from_serialize(payload).unwrap();
    tera.render("unsafe_render", &context)
        .unwrap()
        .into_response()
}

async fn safe_render(Json(payload): Json<SafePayload>) -> impl IntoResponse {
    let mut tera = Tera::default();
    tera.add_raw_template(
        "safe_render",
        r#"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {{ content }}
  </body>
</html>"#,
    )
    .unwrap();
    let mut context = tera::Context::new();
    context.insert(
        "content",
        &payload
            .content
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;"),
    );
    // dbg!(tera.render("safe_render", &context).unwrap());
    tera.render("safe_render", &context)
        .unwrap()
        .into_response()
}
