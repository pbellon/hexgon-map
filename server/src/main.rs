mod coords;
mod game;
mod grid;
mod user;

#[cfg(test)]
mod tests;

use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use coords::CubeCoords;
use game::GameData;
use grid::{generate_grid, TileData, TileStore, TileStoreRwLock};
use serde_json;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::RwLock;
use std::time::Duration;
use tokio::task;

const DATA_FILE: &str = "tile_data.json";

fn load_data_from_file() -> TileStore {
    if let Ok(mut file) = File::open(DATA_FILE) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if let Ok(data) = serde_json::from_str::<TileStore>(&contents) {
                return data;
            }
        }
    }

    generate_grid(10)
}

async fn periodic_save(data: Data<TileStoreRwLock>) {
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

#[post("/tile/{q}/{r}/{s}")]
async fn post_tile(
    path: Path<CubeCoords>,
    data: Data<TileStoreRwLock>,
    tile_data: Json<TileData>,
) -> impl Responder {
    let coords = path.into_inner();
    let mut store = data.write().unwrap();
    store.insert(coords, tile_data.into_inner());
    HttpResponse::Ok().body("Tile updated")
}

#[get("/grid")]
async fn get_grid(data: Data<RwLock<GameData>>) -> impl Responder {
    let store = data.read().unwrap();

    HttpResponse::Ok().json(store.grid())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let initial_data = load_data_from_file();
    let data = Data::new(RwLock::new(initial_data));
    let data_clone = data.clone();
    task::spawn(async move {
        periodic_save(data_clone).await;
    });

    HttpServer::new(move || App::new().app_data(data.clone()).service(post_tile))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
