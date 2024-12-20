use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::{get, http, post, App, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::basic::BasicAuth;
use pixelstratwar::config::GameConfig;
use pixelstratwar::coords::AxialCoords;
use pixelstratwar::game::GameData;
use pixelstratwar::store::{init_redis_client, RedisHandler};
use pixelstratwar::user::GameUsers;
use pixelstratwar::websocket::{
    init_clients, notify_new_user, notify_score_change, tile_change_message, ws_handler,
    ClientList, MyBinaryMessage,
};
use serde::Deserialize;

#[post("/tile/{q}/{r}")]
async fn post_tile(
    path: web::Path<AxialCoords>,
    game_data: web::Data<GameData>,
    users: web::Data<GameUsers>,
    clients: web::Data<ClientList>,
    redis_client: web::Data<redis::Client>,
    user_id: String,
    credentials: BasicAuth,
) -> impl Responder {
    let user_id_auth = credentials.user_id();
    let token: &str = credentials.password().unwrap_or("");

    // log::info!("user_id({user_id}) - token({token})");

    if users.is_valid_token_for_user(user_id_auth, token).await {
        // log::info!("Yep it's valid");
        let coords = path.into_inner();

        let updated_tiles = match game_data
            .handle_click(&**redis_client, &coords, &user_id)
            .await
        {
            Ok(value) => value,
            Err(e) => {
                return HttpResponse::InternalServerError().body(format!(
                    "An error occured while handling click on {coords:?}.\nError: {e}"
                ));
            }
        };

        // log::info!("Updated tiles => {updated_tiles:?}");

        for client in clients.lock().unwrap().iter() {
            updated_tiles.iter().for_each(|(coords, tile)| {
                client.do_send(MyBinaryMessage(tile_change_message(&coords, &tile)));
            });
        }

        let new_score = redis_client.count_tiles_by_user(&user_id).await.unwrap();

        notify_score_change(&clients, &user_id, new_score as u32);

        return HttpResponse::Ok().body("Tile updated");
    } else {
        // log::info!("Nope, it's not valid returning unauthorized");
        return HttpResponse::Unauthorized().body("Invalid token");
    }
}

#[get("/settings")]
async fn get_game_settings(game_data: web::Data<GameData>) -> impl Responder {
    log::info!("OK?");

    HttpResponse::Ok()
        .content_type("application/json")
        .json(game_data.settings)
}

#[derive(Deserialize)]
struct BatchTilesQuery {
    batch: usize,
}

#[get("/tiles")]
async fn get_batch_tiles(
    redis_client: web::Data<redis::Client>,
    game_data: web::Data<GameData>,
    query: web::Query<BatchTilesQuery>,
) -> impl Responder {
    match game_data.compute_batch(&**redis_client, query.batch).await {
        Ok(computed_batch) => HttpResponse::Ok()
            .content_type("application/json")
            .json(computed_batch),
        Err(e) => HttpResponse::InternalServerError()
            .content_type("text/plain")
            .body(format!("Failed to compute batch: {}", e)),
    }
}

#[get("/batches")]
async fn get_batch_list(game_data: web::Data<GameData>) -> impl Responder {
    let list = game_data.get_batch_list();

    HttpResponse::Ok()
        .content_type("application/json")
        .json(list)
}

#[get("/users")]
async fn get_users(users: web::Data<GameUsers>) -> impl Responder {
    let users_public = users.as_public().await;

    HttpResponse::Ok()
        .content_type("application/json")
        .json(users_public)
}

#[derive(Deserialize)]
struct RegisterUserParams {
    username: String,
}

#[post("/login")]
async fn register_user(
    users: web::Data<GameUsers>,
    clients: web::Data<ClientList>,
    post_params: web::Json<RegisterUserParams>,
) -> impl Responder {
    let username = post_params.into_inner().username;
    let user = users.register_user(&username).await;

    notify_new_user(&clients, &user.id, &user.username, &user.color);

    HttpResponse::Ok().json(user)
}

fn cors_middleware(app_config: &GameConfig) -> Cors {
    Cors::default()
        .allowed_methods(vec!["GET", "POST"])
        .allowed_origin(&app_config.front_end_url)
        .allowed_headers(vec![
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
        ])
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log::info!("Initializing game");

    let users = GameUsers::new();
    let app_config = GameConfig::read_config_from_env();

    let redis_client = init_redis_client(&app_config).await.unwrap();
    log::info!("Redis Client initialized");

    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let game_data = GameData::init_from_config(&redis_client, &app_config, &users).await;

    // log::info!("Game data initialized: {game_data:?}");

    let clients = init_clients();

    // let data_clone = data.clone();
    // task::spawn(async move {
    //     periodic_save(data_clone).await;
    // });

    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
            .app_data(web::Data::new(game_data.clone()))
            .app_data(web::Data::new(clients.clone()))
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(users.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            // protected
            .service(post_tile)
            .service(get_batch_list)
            .service(get_batch_tiles)
            .service(get_game_settings)
            .service(get_users)
            .service(register_user)
            .service(web::resource("/ws").to(ws_handler))
            // .wrap(Compress::default())
            .wrap(logger)
            .wrap(cors_middleware(&app_config))
    })
    .workers(350)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
