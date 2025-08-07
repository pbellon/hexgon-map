use std::collections::HashMap;

use deadpool_redis::{Config, Runtime};

use crate::{
    config::GameConfig,
    coords::AxialCoords,
    game::InnerTileData,
    user::{PublicUser, User},
};

/// Redis prefixes and keys
const USER_IDS_KEY: &str = "user_ids";

const USER_PREFIX: &str = "user";

const TILE_PREFIX: &str = "tile";

const TOKEN_PREFIX: &str = "token";

const TILE_INDEX: &str = "idx:tile";

fn get_tile_key(coords: &AxialCoords) -> String {
    format!("{}:{}", TILE_PREFIX, coords.as_redis_key())
}

fn get_user_key(user: &User) -> String {
    get_user_key_from_str(&user.id)
}

fn get_user_key_from_str(user_id: &str) -> String {
    format!("{}:{}", USER_PREFIX, user_id)
}

fn get_token_key(user_id: &str) -> String {
    format!("{}:{}", TOKEN_PREFIX, user_id)
}

fn parse_tile_hashmap(map: &HashMap<String, String>) -> redis::RedisResult<Option<InnerTileData>> {
    if map.is_empty() {
        return Ok(None);
    }

    // Extract fields and construct InnerTileData
    let user_id = map
        .get("user_id")
        .ok_or_else(|| redis::RedisError::from((redis::ErrorKind::TypeError, "Missing user_id")))?;

    let damage = map
        .get("damage")
        .ok_or_else(|| redis::RedisError::from((redis::ErrorKind::TypeError, "Missing damage")))?
        .parse::<u8>()
        .map_err(|_| {
            redis::RedisError::from((redis::ErrorKind::TypeError, "Invalid damage value"))
        })?;

    return Ok(Some(InnerTileData {
        user_id: user_id.clone(),
        damage,
    }));
}

#[async_trait::async_trait]
pub trait RedisHandler {
    async fn flushdb(&self) -> redis::RedisResult<bool>;

    async fn count_tiles_by_user(&self, user_id: &str) -> redis::RedisResult<usize>;

    async fn get_tile<C>(
        &self,
        con: &mut C,
        coords: &AxialCoords,
    ) -> redis::RedisResult<Option<InnerTileData>>
    where
        C: redis::aio::ConnectionLike + Send;

    async fn set_tile<C>(
        &self,
        con: &mut C,
        coords: &AxialCoords,
        data: InnerTileData,
    ) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send;

    async fn batch_get_tiles<C>(
        &self,
        con: &mut C,
        coords: Vec<AxialCoords>,
    ) -> redis::RedisResult<Vec<(AxialCoords, InnerTileData)>>
    where
        C: redis::aio::ConnectionLike + Send;

    async fn add_user<C>(&self, con: &mut C, user: User) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send;

    async fn get_public_users<C>(&self, con: &mut C) -> redis::RedisResult<Vec<PublicUser>>
    where
        C: redis::aio::ConnectionLike + Send;

    async fn is_valid_token_for_user<C>(
        &self,
        con: &mut C,
        token: &str,
        user_id: &str,
    ) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send;
}

#[async_trait::async_trait]
impl RedisHandler for redis::Client {
    async fn flushdb(&self) -> redis::RedisResult<bool> {
        let mut con = self
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to create multiplexed async connection");

        let _: () = redis::cmd("FLUSHDB")
            .query_async(&mut con)
            .await
            .expect("Could not flush DB");
        Ok(true)
    }

    async fn batch_get_tiles<C>(
        &self,
        con: &mut C,
        coords: Vec<AxialCoords>,
    ) -> redis::RedisResult<Vec<(AxialCoords, InnerTileData)>>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let mut pipe = redis::pipe();

        let mut keys = Vec::new();

        for c in coords.iter() {
            keys.push(c);
            pipe.hgetall(get_tile_key(c));
        }

        let query_res: Vec<HashMap<String, String>> =
            pipe.query_async(con).await.unwrap_or(Vec::new());

        let mut res: Vec<(AxialCoords, InnerTileData)> = Vec::new();
        let mut i = 0;

        for hash in query_res.iter() {
            let coord = *keys.get(i).unwrap();
            match parse_tile_hashmap(hash) {
                Ok(Some(tile)) => {
                    res.push((coord.clone(), tile));
                }
                Ok(None) => {
                    // do nothing
                }
                Err(_) => {
                    // do nothing
                }
            }
            i += 1;
        }

        Ok(res)
    }

    async fn count_tiles_by_user(&self, user_id: &str) -> Result<usize, redis::RedisError> {
        // log::warn!("Not implemented count_tiles_by_user({user_id})");

        Ok(0)
    }

    async fn get_tile<C>(
        &self,
        con: &mut C,
        coords: &AxialCoords,
    ) -> Result<Option<InnerTileData>, redis::RedisError>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let tile_k = get_tile_key(coords);

        let res = match redis::Cmd::hgetall(tile_k).query_async(con).await {
            Ok(Some(map)) => parse_tile_hashmap(&map).unwrap_or(None),
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };

        Ok(res)
    }

    async fn set_tile<C>(
        &self,
        con: &mut C,
        coords: &AxialCoords,
        tile: InnerTileData,
    ) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let key = get_tile_key(coords);

        let () = redis::pipe()
            .hset(key.clone(), "user_id", tile.user_id)
            .hset(key.clone(), "damage", tile.damage)
            .query_async(con)
            .await?;

        Ok(true)
    }

    async fn add_user<C>(&self, con: &mut C, user: User) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        // println!("[RedisHandler.add_user] will add user into Redis DB");
        let key = get_user_key(&user);
        let mut pipe = redis::pipe();

        let id = user.id.clone();
        let username = user.username.clone();
        let color = user.color.clone();
        let token = user.token.clone();

        let token_key = get_token_key(&id);
        // add token
        pipe.set(token_key.clone(), &token);

        // println!(
        //     "[redis::Client.add_user] Will set token ({}) at {})",
        //     token,
        //     &token_key.clone()
        // );

        pipe.cmd("hset").arg(&key).arg("id").arg(&id);

        pipe.cmd("hset").arg(&key).arg("username").arg(&username);

        pipe.cmd("hset").arg(&key).arg("color").arg(&color);

        pipe.cmd("hset").arg(&key).arg("token").arg(&token);

        let () = pipe
            .query_async(con)
            .await
            .expect("Should be able to add user");

        Ok(true)
    }

    async fn get_public_users<C>(&self, con: &mut C) -> redis::RedisResult<Vec<PublicUser>>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        // first we all user id stored under `USER_IDS_KEY`
        let ids: Vec<String> = redis::Cmd::lrange(USER_IDS_KEY, 0, -1)
            .query_async(con)
            .await?;

        let mut pipe = redis::pipe();

        for id in ids {
            pipe.hgetall(get_user_key_from_str(&id));
            pipe.cmd("FT.SEARCH")
                .arg(TILE_INDEX)
                .arg(format!("\"@user_id:{id}\""))
                .arg("LIMIT")
                .arg(0)
                .arg(0);
        }

        let mut results = Vec::new();
        let pipe_res: Vec<redis::Value> = pipe.query_async(con).await?;

        let mut iter = pipe_res.iter();

        while let (Some(hash_map_value), Some(nb_tiles_value)) = (iter.next(), iter.next()) {
            let user: User = redis::from_redis_value(&hash_map_value)?;

            let score: u32 = redis::from_redis_value(nb_tiles_value)?;

            results.push(PublicUser {
                id: user.id,
                color: user.color,
                username: user.username,
                score,
            });
        }

        Ok(results)
    }

    async fn is_valid_token_for_user<C>(
        &self,
        con: &mut C,
        token: &str,
        user_id: &str,
    ) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let token_key = get_token_key(user_id);

        let r_token: Option<String> = redis::Cmd::get(token_key).query_async(con).await?;

        if let Some(t) = r_token {
            return Ok(t == token.to_string());
        } else {
            return Ok(false);
        }
    }
}

pub async fn has_index<C>(conn: &mut C, index_name: &str) -> redis::RedisResult<bool>
where
    C: redis::aio::ConnectionLike + Send,
{
    let indices: Vec<String> = redis::cmd("FT._LIST")
        .query_async(conn)
        .await
        .expect("Unable to retrieve indices list");

    Ok(indices.contains(&index_name.to_string()))
}

pub async fn init_redis_client(
    app_config: &GameConfig,
) -> redis::RedisResult<(redis::Client, deadpool_redis::Pool)> {
    let client =
        redis::Client::open(app_config.redis_url.clone()).expect("Failed to create redis client");

    let cfg = Config::from_url(app_config.redis_url.clone());
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

    Ok((client, pool))
}

pub async fn init_redis_indices<C>(conn: &mut C) -> redis::RedisResult<bool>
where
    C: redis::aio::ConnectionLike + Send,
{
    let has_tile_index = has_index(conn, "idx:tile").await.unwrap();

    if has_tile_index {
        let () = redis::cmd("FT.DROPINDEX")
            .arg("idx:tile")
            .arg("DD")
            .query_async(conn)
            .await
            .unwrap();
    }

    // create indices
    let () = redis::cmd("FT.CREATE")
        .arg("idx:tile")
        .arg("ON")
        .arg("HASH")
        .arg("PREFIX")
        .arg(1)
        .arg("tile:")
        .arg("SCHEMA")
        .arg("user_id")
        .arg("TAG")
        .arg("damage")
        .arg("NUMERIC")
        .query_async(conn)
        .await?;

    Ok(true)
}

// const DATA_FILE: &str = "game_data.json";

// fn load_data_from_file(radius: i32) -> GameData {
//     if let Ok(mut file) = File::open(DATA_FILE) {
//         let mut contents = String::new();
//         if file.read_to_string(&mut contents).is_ok() {
//             if let Ok(data) = serde_json::from_str::<GameData>(&contents) {
//                 return data;
//             }
//         }
//     }

//     GameData::new(radius)
// }

// async fn periodic_save(data: Data<RwLock<GameData>>) {
//     loop {
//         tokio::time::sleep(Duration::from_secs(30)).await; // Save every 30 seconds
//         if let Ok(store) = data.write() {
//             if let Ok(serialized) = serde_json::to_string(&*store) {
//                 if let Ok(mut file) = File::create(DATA_FILE) {
//                     let _ = file.write_all(serialized.as_bytes());
//                 }
//             }
//         }
//     }
// }

// fn parse_cord_part(cord: String) -> Result<i32, ParseIntError> {
//     let mut to_parse = cord.clone();

//     let has_minor_sign = to_parse.starts_with("m");
//     let mut sign = 1;

//     if has_minor_sign {
//         sign = -1;
//         to_parse = to_parse.get(1..).unwrap_or("").to_string();
//     }

//     let parsed: i32 = to_parse.parse().unwrap();

//     Ok(parsed * sign)
// }

// fn parse_tile_key(key: String) -> Option<AxialCoords> {
//     let vec: Vec<&str> = key.split(":").collect();
//     match vec.get(1) {
//         Some(v) => {
//             let parts: Vec<i32> = (*v)
//                 .to_string()
//                 .split('_')
//                 .map(|p| parse_cord_part(p.to_string()).expect(&format!("Cannot parse {p}")))
//                 .collect();

//             let q = *parts.get(0).unwrap();
//             let r = *parts.get(1).unwrap();

//             return Some(AxialCoords { q, r });
//         }
//         None => None,
//     }
// }
