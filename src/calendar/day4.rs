use axum::{extract::Json, response::IntoResponse, routing::post, Router};
use serde::{Deserialize, Serialize};

pub fn router() -> Router {
    Router::new()
        .route("/4/strength", post(strength))
        .route("/4/contest", post(contest))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct StrongReindeer {
    name: String,
    strength: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Reindeer {
    name: String,
    strength: u64,
    speed: f64,
    height: u64,
    antler_width: u64,
    snow_magic_power: u64,
    favorite_food: String,
    #[serde(rename = "cAnD13s_3ATeN-yesT3rdAy")]
    candy_eaten_yesterday: u64,
}

async fn strength(Json(reindeers): Json<Vec<StrongReindeer>>) -> String {
    let total_strength: u64 = reindeers.iter().map(|r| r.strength).sum();
    format!("{}", total_strength)
}

fn find_max_by_attribute(reindeers: Vec<Reindeer>, attribute: &str) -> Option<Reindeer> {
    let binding = reindeers.clone();
    let mut winner: Option<Reindeer> = None;
    let mut max_attribute: u64 = 0;
    for attr in binding.iter() {
        match attribute {
            "speed" => {
                if (attr.speed * 100.0) as u64 > max_attribute {
                    max_attribute = (attr.speed * 100.0) as u64;
                    winner = Some(attr.clone());
                }
            }
            "height" => {
                if attr.height > max_attribute {
                    max_attribute = attr.height as u64;
                    winner = Some(attr.clone());
                }
            }
            "snow_magic_power" => {
                if attr.snow_magic_power > max_attribute as u64 {
                    max_attribute = attr.snow_magic_power as u64;
                    winner = Some(attr.clone());
                }
            }
            "candy_eaten_yesterday" => {
                if attr.candy_eaten_yesterday > max_attribute as u64 {
                    max_attribute = attr.candy_eaten_yesterday as u64;
                    winner = Some(attr.clone());
                }
            }
            _ => panic!("Invalid attribute"),
        }
    }
    let winner = winner.unwrap().clone();
    Some(Reindeer {
        name: winner.name,
        strength: winner.strength,
        speed: winner.speed,
        height: winner.height,
        antler_width: winner.antler_width,
        snow_magic_power: winner.snow_magic_power,
        favorite_food: winner.favorite_food,
        candy_eaten_yesterday: winner.candy_eaten_yesterday,
    })
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ContestResponse {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}

impl ContestResponse {
    fn new(
        fastest: String,
        tallest: String,
        magician: String,
        consumer: String,
    ) -> ContestResponse {
        ContestResponse {
            fastest,
            tallest,
            magician,
            consumer,
        }
    }
}

async fn contest(Json(reindeers): Json<Vec<Reindeer>>) -> impl IntoResponse {
    let reindeer_list = reindeers;
    let tallest = find_max_by_attribute(reindeer_list.clone(), "height").unwrap();
    let fastest = find_max_by_attribute(reindeer_list.clone(), "speed").unwrap();
    let magician = find_max_by_attribute(reindeer_list.clone(), "snow_magic_power").unwrap();
    let consumer = find_max_by_attribute(reindeer_list.clone(), "candy_eaten_yesterday").unwrap();
    let tallest_instr = format!(
        "{} is standing tall with his {} cm wide antlers",
        tallest.name, tallest.antler_width
    );
    let fastest_instr = format!(
        "Speeding past the finish line with a strength of {} is {}",
        fastest.strength, fastest.name
    );
    let magician_instr = format!(
        "{} could blast you away with a snow magic power of {}",
        magician.name, magician.snow_magic_power
    );
    let consumer_instr = format!(
        "{} ate lots of candies, but also some {}",
        consumer.name, consumer.favorite_food
    );
    let response =
        ContestResponse::new(fastest_instr, tallest_instr, magician_instr, consumer_instr);
    Json(response).into_response()
}
