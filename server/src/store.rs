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

use std::collections::HashMap;

use redis::aio::MultiplexedConnection;

use crate::{config::GameConfig, coords::AxialCoords, game::InnerTileData};

#[async_trait::async_trait]
pub trait RedisHandler {
    async fn flushdb(&self) -> redis::RedisResult<()>;

    async fn count_tiles_by_user(&self, user_id: &str) -> redis::RedisResult<usize>;

    async fn get_tile(&self, coords: &AxialCoords) -> redis::RedisResult<Option<InnerTileData>>;

    async fn set_tile(&self, coords: &AxialCoords, data: InnerTileData) -> redis::RedisResult<()>;

    async fn batch_get_tiles(
        &self,
        coords: Vec<AxialCoords>,
    ) -> redis::RedisResult<Vec<(AxialCoords, InnerTileData)>>;
}

fn get_tile_key(coords: &AxialCoords) -> String {
    format!("tile:{}", coords.as_redis_key())
}

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

fn parse_hashmap(
    map: &HashMap<String, String>,
) -> Result<Option<InnerTileData>, redis::RedisError> {
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
impl RedisHandler for redis::Client {
    async fn flushdb(&self) -> redis::RedisResult<()> {
        let mut con = self
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to create multiplexed async connection");

        let _: () = redis::cmd("FLUSHDB")
            .query_async(&mut con)
            .await
            .expect("Could not flush DB");
        Ok(())
    }

    async fn batch_get_tiles(
        &self,
        coords: Vec<AxialCoords>,
    ) -> Result<Vec<(AxialCoords, InnerTileData)>, redis::RedisError> {
        let mut con = self
            .get_connection_manager()
            .await
            .expect("Could not open connection");
        let mut pipe = redis::pipe();

        let mut keys = Vec::new();

        for c in coords.iter() {
            keys.push(c);
            pipe.hgetall(get_tile_key(c));
        }

        let query_res: Vec<HashMap<String, String>> =
            pipe.query_async(&mut con).await.unwrap_or(Vec::new());

        let mut res: Vec<(AxialCoords, InnerTileData)> = Vec::new();
        let mut i = 0;

        for hash in query_res.iter() {
            let coord = *keys.get(i).unwrap();
            match parse_hashmap(hash) {
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
        log::warn!("Not implemented count_tiles_by_user({user_id})");

        Ok(0)
    }

    async fn get_tile(
        &self,
        coords: &AxialCoords,
    ) -> Result<Option<InnerTileData>, redis::RedisError> {
        let mut con = self
            .get_connection_manager()
            .await
            .expect("Failed to open connection");

        let tile_k = get_tile_key(coords);

        let res = match redis::Cmd::hgetall(tile_k)
            .query_async::<_, Option<HashMap<String, String>>>(&mut con)
            .await
        {
            Ok(Some(map)) => parse_hashmap(&map).unwrap_or(None),
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };

        Ok(res)
    }

    async fn set_tile(
        &self,
        coords: &AxialCoords,
        tile: InnerTileData,
    ) -> Result<(), redis::RedisError> {
        let mut con = self.get_connection_manager().await.unwrap();
        let key = get_tile_key(coords);

        redis::pipe()
            .hset(key.clone(), "user_id", tile.user_id)
            .hset(key.clone(), "damage", tile.damage)
            .query_async(&mut con)
            .await
    }
}

pub async fn has_index(client: &redis::Client, index_name: &str) -> redis::RedisResult<bool> {
    let mut conn = client.get_connection_manager().await.unwrap();
    let indices: Vec<String> = redis::cmd("FT._LIST")
        .query_async(&mut conn)
        .await
        .expect("Unable to retrieve indices list");

    Ok(indices.contains(&index_name.to_string()))
}

pub async fn init_redis_client(app_config: &GameConfig) -> redis::RedisResult<redis::Client> {
    let client =
        redis::Client::open(app_config.redis_url.clone()).expect("Failed to create redis client");

    Ok(client)
}

pub async fn init_redis_indices(client: &redis::Client) -> redis::RedisResult<bool> {
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to create multiplexed async connection");

    let has_tile_index = has_index(&client, "idx:tile").await.unwrap();

    if has_tile_index {
        redis::cmd("FT.DROPINDEX")
            .arg("idx:tile")
            .arg("DD")
            .query_async::<_, ()>(&mut con)
            .await
            .unwrap();
    }

    // create indices
    let _ = redis::cmd("FT.CREATE")
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
        .query_async::<MultiplexedConnection, ()>(&mut con)
        .await
        .or_else(|e| {
            log::error!("Failed to create indice: {e}");
            Ok::<(), redis::RedisError>(())
        });

    Ok(true)
}
