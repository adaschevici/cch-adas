use axum::{
    body::Body,
    extract::Multipart,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use tower_http::services::ServeFile;

pub fn router() -> Router {
    Router::new()
        .route("/11/red_pixels", post(analyze_pixels))
        .nest_service(
            "/11/assets/decoration.png",
            ServeFile::new("assets/decoration.png"),
        )
}

async fn analyze_pixels(mut files: Multipart) -> impl IntoResponse {
    match files.next_field().await.unwrap() {
        Some(field) => {
            let _name = field.name().unwrap().to_string();
            let data = field.bytes().await.unwrap();
            let image = image::io::Reader::new(std::io::Cursor::new(data.clone()))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap();
            let total_magic_red =
                image
                    .into_rgba8()
                    .enumerate_pixels()
                    .fold(0, |acc, (_x, _y, pixel)| {
                        let [r, g, b, _] = pixel.0;
                        if r as usize > b as usize + g as usize {
                            acc + 1
                        } else {
                            acc
                        }
                    });
            return total_magic_red.to_string().into_response();
        }
        None => {
            println!("No file uploaded");
            return "422".into_response();
        }
    }
}
