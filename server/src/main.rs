mod coords;
mod game;
mod grid;
mod user;
mod websocket;

#[cfg(test)]
mod tests;

use actix_web::middleware::Logger;
use actix_web::web::{resource, Data, Json, Path};
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use coords::AxialCoords;
use env_logger::Env;
use game::GameData;
use serde::Deserialize;
use std::sync::RwLock;
use websocket::{init_clients, ws_handler, ClientList};

const DEFAULT_GRID_RADIUS: u8 = 80;

#[derive(Deserialize)]
struct PostTileData {
    pub user_id: String,
}

#[post("/tile/{q}/{r}")]
async fn post_tile(
    path: Path<AxialCoords>,
    game_data: Data<RwLock<GameData>>,
    tile_data: Json<PostTileData>,
) -> impl Responder {
    let coords = path.into_inner();
    let mut store = game_data.write().unwrap();
    let new_tiles = store.handle_click(coords, tile_data.into_inner().user_id);
    HttpResponse::Ok().json(new_tiles)
    // return HttpResponse::BadRequest().body(format!("Tile does not exists at {:?}", coords));
}

#[get("/grid")]
async fn get_grid(app_data: Data<RwLock<GameData>>) -> impl Responder {
    let store = app_data.read().unwrap();

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

    let initial_data = GameData::new(DEFAULT_GRID_RADIUS as i32);
    let data = Data::new(RwLock::new(initial_data));

    let clients: ClientList = init_clients();

    // let data_clone = data.clone();
    // task::spawn(async move {
    //     periodic_save(data_clone).await;
    // });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(Data::new(clients.clone()))
            .service(post_tile)
            .service(resource("/ws/").to(ws_handler))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
