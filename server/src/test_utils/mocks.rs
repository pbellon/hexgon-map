use std::{collections::HashMap, sync::Arc};

use async_trait;
use redis::aio::MultiplexedConnection;

use crate::{coords::AxialCoords, game::InnerTileData, store::RedisHandler};
use tokio::sync::RwLock;

pub struct MockRedisHandler {
    pub mock_data: Arc<RwLock<HashMap<AxialCoords, InnerTileData>>>,
}

impl MockRedisHandler {
    pub fn new() -> Self {
        Self {
            mock_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl RedisHandler for MockRedisHandler {
    async fn open_connection(&self) -> redis::RedisResult<Option<MultiplexedConnection>> {
        Ok(None)
    }

    async fn flushdb(&self) -> redis::RedisResult<()> {
        let mut write = self.mock_data.write().await;

        write.clear();

        Ok(())
    }

    async fn count_tiles_by_user(&self, user_id: &str) -> redis::RedisResult<usize> {
        let read = self.mock_data.read().await;

        let mut count = 0;

        for (_, tile) in read.iter() {
            if tile.user_id == user_id {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn batch_get_tiles(
        &self,
        coords: Vec<AxialCoords>,
        _reuse_con: Option<MultiplexedConnection>,
    ) -> redis::RedisResult<Vec<(AxialCoords, InnerTileData)>> {
        let read = self.mock_data.read().await;
        let mut results = Vec::new();
        for c in coords.iter() {
            match read.get(c) {
                Some(t) => {
                    results.push((c.clone(), t.clone()));
                }
                None => {
                    // do nothing
                }
            }
        }

        Ok(results)
    }

    async fn get_tile(
        &self,
        coords: &AxialCoords,
        _reuse_con: Option<MultiplexedConnection>,
    ) -> redis::RedisResult<Option<InnerTileData>> {
        let read = self.mock_data.read().await;
        Ok(read.get(&coords).cloned())
    }

    async fn set_tile(
        &self,
        coords: &AxialCoords,
        tile: InnerTileData,
        _reuse_con: Option<MultiplexedConnection>,
    ) -> redis::RedisResult<()> {
        let mut write = self.mock_data.write().await;
        write.insert(coords.clone(), tile);
        Ok(())
    }
}
