mod coords;
mod game;
mod grid;
mod user;
mod websocket;

#[cfg(test)]
mod tests;

use actix_web::middleware::{Compress, Logger};
use actix_web::web::{resource, Data, Json, Path};
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use coords::AxialCoords;
use env_logger::Env;
use game::{create_benchmark_game_data, GameData};
use serde::Deserialize;
use std::sync::RwLock;
use websocket::{
    init_clients, notify_new_user, notify_score_change, tile_change_message, ws_handler,
    ClientList, MyBinaryMessage,
};

const DEFAULT_GRID_RADIUS: u8 = 80;

#[post("/tile/{q}/{r}")]
async fn post_tile(
    path: Path<AxialCoords>,
    game_data: Data<RwLock<GameData>>,
    clients: Data<ClientList>,
    user_id: String,
) -> impl Responder {
    let coords = path.into_inner();
    let mut store = game_data.write().unwrap();
    let updated_tiles = store.handle_click(&coords, &user_id);

    for client in clients.lock().unwrap().iter() {
        updated_tiles.iter().for_each(|(coords, tile)| {
            client.do_send(MyBinaryMessage(tile_change_message(&coords, &tile)));
        });
    }

    let new_score = store.score_of_user(&user_id);

    notify_score_change(&clients, &user_id, new_score);

    HttpResponse::Ok().json(updated_tiles)
    // return HttpResponse::BadRequest().body(format!("Tile does not exists at {:?}", coords));
}

#[get("/data")]
async fn get_game_data(app_data: Data<RwLock<GameData>>) -> impl Responder {
    let store = app_data.read().unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .json(store.as_public())
}

#[derive(Deserialize)]
struct RegisterUserParams {
    username: String,
}

#[post("/login")]
async fn register_user(
    app_data: Data<RwLock<GameData>>,
    clients: Data<ClientList>,
    post_params: Json<RegisterUserParams>,
) -> impl Responder {
    let mut store = app_data.write().unwrap();
    let username = post_params.into_inner().username;
    let user = store.create_user(username);

    notify_new_user(&clients, &user.id, &user.username, &user.color);

    HttpResponse::Ok().json(user)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let initial_data = create_benchmark_game_data(DEFAULT_GRID_RADIUS as i32);
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
            .service(get_game_data)
            .service(register_user)
            .service(resource("/ws").to(ws_handler))
            .wrap(Compress::default())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
