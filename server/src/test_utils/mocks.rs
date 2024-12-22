use std::{collections::HashMap, sync::Arc};

use async_trait;

use crate::{
    config::GameConfig,
    coords::AxialCoords,
    game::InnerTileData,
    store::{self, RedisHandler},
    user::{PublicUser, User},
};
use redis;

use tokio::sync::RwLock;

pub struct MockRedisConnection;

impl MockRedisConnection {
    fn new() -> Self {
        MockRedisConnection
    }
}

impl redis::aio::ConnectionLike for MockRedisConnection {
    fn get_db(&self) -> i64 {
        0
    }
    fn req_packed_command<'a>(
        &'a mut self,
        _cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        Box::pin(async move {
            log::info!("received a command");
            Ok(redis::Value::Okay)
        })
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        _cmd: &'a redis::Pipeline,
        _offset: usize,
        _count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        Box::pin(async move {
            log::info!("received a bunch of commands");
            Ok(vec![redis::Value::Okay])
        })
    }
}

pub struct MockRedisHandler {
    pub mock_tokens: Arc<RwLock<HashMap<String, String>>>,
    pub mock_users: Arc<RwLock<HashMap<String, User>>>,
    pub mock_grid: Arc<RwLock<HashMap<AxialCoords, InnerTileData>>>,
}

impl MockRedisHandler {
    pub fn new() -> Self {
        Self {
            mock_tokens: Arc::new(RwLock::new(HashMap::new())),
            mock_users: Arc::new(RwLock::new(HashMap::new())),
            mock_grid: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl RedisHandler for MockRedisHandler {
    async fn flushdb(&self) -> redis::RedisResult<bool> {
        let mut write = self.mock_grid.write().await;

        write.clear();

        Ok(true)
    }

    async fn count_tiles_by_user(&self, user_id: &str) -> redis::RedisResult<usize> {
        let read = self.mock_grid.read().await;

        let mut count = 0;

        for (_, tile) in read.iter() {
            if tile.user_id == user_id {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn batch_get_tiles<C>(
        &self,
        _reuse_con: &mut C,
        coords: Vec<AxialCoords>,
    ) -> redis::RedisResult<Vec<(AxialCoords, InnerTileData)>>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let read = self.mock_grid.read().await;
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

    async fn get_tile<C>(
        &self,
        _c: &mut C,
        coords: &AxialCoords,
    ) -> redis::RedisResult<Option<InnerTileData>>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let read = self.mock_grid.read().await;
        Ok(read.get(&coords).cloned())
    }

    async fn set_tile<C>(
        &self,
        _c: &mut C,
        coords: &AxialCoords,
        tile: InnerTileData,
    ) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let mut write = self.mock_grid.write().await;
        write.insert(coords.clone(), tile);
        Ok(true)
    }

    async fn add_user<C>(&self, _con: &mut C, user: User) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let mut w_users = self.mock_users.write().await;
        let mut w_tokens = self.mock_tokens.write().await;

        w_tokens.insert(user.id.clone(), user.token.clone());
        w_users.insert(user.id.clone(), user.clone());

        Ok(true)
    }

    async fn get_public_users<C>(&self, _con: &mut C) -> redis::RedisResult<Vec<PublicUser>>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let read_users = self.mock_users.read().await;
        let read_tiles = self.mock_grid.read().await;

        let mut res = Vec::new();

        for (_, user) in read_users.iter() {
            let user_tiles = read_tiles
                .iter()
                .filter(|(_, t)| t.user_id == user.id)
                .count();

            res.push(PublicUser {
                color: user.color.clone(),
                id: user.id.clone(),
                username: user.username.clone(),
                score: user_tiles as u32,
            })
        }

        Ok(res)
    }

    async fn is_valid_token_for_user<C>(
        &self,
        _con: &mut C,
        token: &str,
        user_id: &str,
    ) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        let r_tokens = self.mock_tokens.read().await;

        match r_tokens.get(user_id) {
            Some(t) => Ok(t == token),
            None => Ok(false),
        }
    }
}

// Define the `RedisClient` enum
pub enum TestRedisClient {
    Real(redis::Client),
    Mock(MockRedisHandler),
}

#[async_trait::async_trait]
impl RedisHandler for TestRedisClient {
    async fn flushdb(&self) -> redis::RedisResult<bool> {
        let res = match self {
            TestRedisClient::Real(client) => client.flushdb(),
            TestRedisClient::Mock(mock) => mock.flushdb(),
        };

        res.await
    }

    async fn count_tiles_by_user(&self, user_id: &str) -> redis::RedisResult<usize> {
        match self {
            TestRedisClient::Real(client) => client.count_tiles_by_user(user_id).await,
            TestRedisClient::Mock(mock) => mock.count_tiles_by_user(user_id).await,
        }
    }
    async fn batch_get_tiles<C>(
        &self,
        con: &mut C,
        coords: Vec<AxialCoords>,
    ) -> redis::RedisResult<Vec<(AxialCoords, InnerTileData)>>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        match self {
            TestRedisClient::Real(client) => client.batch_get_tiles(con, coords).await,
            TestRedisClient::Mock(mock) => mock.batch_get_tiles(con, coords).await,
        }
    }

    async fn get_tile<C>(
        &self,
        con: &mut C,
        coords: &AxialCoords,
    ) -> redis::RedisResult<Option<InnerTileData>>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        match self {
            TestRedisClient::Real(client) => client.get_tile(con, coords).await,
            TestRedisClient::Mock(mock) => mock.get_tile(con, coords).await,
        }
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
        match self {
            TestRedisClient::Real(client) => client.set_tile(con, coords, tile).await,
            TestRedisClient::Mock(mock) => mock.set_tile(con, coords, tile).await,
        }
    }

    async fn add_user<C>(&self, con: &mut C, user: User) -> redis::RedisResult<bool>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        match self {
            TestRedisClient::Real(client) => client.add_user(con, user).await,
            TestRedisClient::Mock(mock) => mock.add_user(con, user).await,
        }
    }

    async fn get_public_users<C>(&self, con: &mut C) -> redis::RedisResult<Vec<PublicUser>>
    where
        C: redis::aio::ConnectionLike + Send,
    {
        match self {
            TestRedisClient::Real(client) => client.get_public_users(con).await,
            TestRedisClient::Mock(mock) => mock.get_public_users(con).await,
        }
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
        match self {
            TestRedisClient::Real(client) => {
                client.is_valid_token_for_user(con, token, user_id).await
            }
            TestRedisClient::Mock(mock) => mock.is_valid_token_for_user(con, token, user_id).await,
        }
    }
}

impl redis::aio::ConnectionLike for TestRedisConnection {
    fn get_db(&self) -> i64 {
        0
    }

    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        match self {
            TestRedisConnection::Real(conn) => {
                Box::pin(async move { conn.req_packed_command(cmd).await })
                    as redis::RedisFuture<'a, redis::Value>
            }
            TestRedisConnection::Mock(conn) => {
                Box::pin(async move { conn.req_packed_command(cmd).await })
                    as redis::RedisFuture<'a, redis::Value>
            }
        }
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        match self {
            TestRedisConnection::Real(conn) => conn.req_packed_commands(cmd, offset, count),
            TestRedisConnection::Mock(conn) => conn.req_packed_commands(cmd, offset, count),
        }
    }
}
pub enum TestRedisConnection {
    Real(redis::aio::MultiplexedConnection),
    Mock(MockRedisConnection),
}

pub async fn get_connection(client: &TestRedisClient) -> redis::RedisResult<TestRedisConnection> {
    match client {
        TestRedisClient::Real(c) => {
            let conn = c.get_multiplexed_async_connection().await?;
            Ok(TestRedisConnection::Real(conn))
        }
        TestRedisClient::Mock(_) => Ok(TestRedisConnection::Mock(MockRedisConnection::new())),
    }
}

pub async fn redis_client_or_mock() -> redis::RedisResult<TestRedisClient> {
    let _ = env_logger::try_init();

    let app_config = GameConfig::read_config_from_env();

    if app_config.with_redis_tests {
        match store::init_redis_client(&app_config).await {
            Ok((client, pool)) => {
                let mut conn = pool.get().await.unwrap();
                let _ = store::init_redis_indices(&mut conn).await?;

                return Ok(TestRedisClient::Real(client));
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    return Ok(TestRedisClient::Mock(MockRedisHandler::new()));
}
