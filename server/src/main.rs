mod coords;
mod game;
mod grid;
mod user;

#[cfg(test)]
mod tests;

use actix_web::middleware::Logger;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use coords::AxialCoords;
use env_logger::Env;
use game::GameData;
use grid::TileData;
use serde::Deserialize;
use serde_json;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::RwLock;
use std::time::Duration;
use tokio::task;

const DATA_FILE: &str = "game_data.json";

fn load_data_from_file(radius: i32) -> GameData {
    if let Ok(mut file) = File::open(DATA_FILE) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(data) = serde_json::from_str::<GameData>(&contents) {
                return data;
            }
        }
    }

    GameData::new(radius)
}

async fn periodic_save(data: Data<RwLock<GameData>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await; // Save every 30 seconds
        if let Ok(store) = data.write() {
            if let Ok(serialized) = serde_json::to_string(&*store) {
                if let Ok(mut file) = File::create(DATA_FILE) {
                    let _ = file.write_all(serialized.as_bytes());
                }
            }
        }
    }
}

#[post("/tile/{q}/{r}")]
async fn post_tile(
    path: Path<AxialCoords>,
    data: Data<RwLock<GameData>>,
    tile_data: Json<TileData>,
) -> impl Responder {
    let coords = path.into_inner();
    let mut store = data.write().unwrap();
    store.insert(coords, tile_data.into_inner());
    HttpResponse::Ok().body("Tile updated")
}

#[get("/grid")]
async fn get_grid(app_data: Data<RwLock<GameData>>) -> impl Responder {
    let store: std::sync::RwLockReadGuard<'_, GameData> = app_data.read().unwrap();

    HttpResponse::Ok().json(store.grid())
}

#[derive(Deserialize)]
struct RegisterUserParams {
    username: Option<String>,
}

#[post("/user")]
async fn register_user(
    app_data: Data<RwLock<GameData>>,
    post_params: Json<RegisterUserParams>,
) -> impl Responder {
    let mut store = app_data.write().unwrap();
    let username = post_params.into_inner().username;
    let user_id = store.create_user(username);

    HttpResponse::Ok().body(user_id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let initial_data = load_data_from_file(40);
    let data = Data::new(RwLock::new(initial_data));
    let data_clone = data.clone();
    task::spawn(async move {
        periodic_save(data_clone).await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(post_tile)
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
