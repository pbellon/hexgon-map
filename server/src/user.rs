use std::collections::HashMap;

use redis::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::{color_to_hex, string_to_color};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub color: String,
    pub id: String,
    pub username: String,
    pub token: String,
}

impl FromRedisValue for User {
    /// Assumes that if we're trying to parse some value, it is there
    /// so if we try to get an user from redis via `hgetall` it must have been
    /// properly saved to redis otherwise will return an error
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let raw_hashmap: HashMap<String, String> = redis::from_redis_value(v)?;

        if raw_hashmap.is_empty() {
            return Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Trying to convert empty hashmap to User",
            )));
        }

        let id = raw_hashmap.get("id").ok_or_else(|| {
            redis::RedisError::from((redis::ErrorKind::TypeError, "Missing id in user hashmap"))
        })?;

        let color = raw_hashmap.get("color").ok_or_else(|| {
            redis::RedisError::from((redis::ErrorKind::TypeError, "Missing color in user hashmap"))
        })?;

        let token = raw_hashmap.get("token").ok_or_else(|| {
            redis::RedisError::from((redis::ErrorKind::TypeError, "Missing token in user hashmap"))
        })?;

        let username = raw_hashmap.get("username").ok_or_else(|| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Missing username in user hashmap",
            ))
        })?;

        return Ok(User {
            id: id.to_owned(),
            color: color.to_owned(),
            token: token.to_owned(),
            username: username.to_owned(),
        });
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PublicUser {
    pub id: String,
    pub username: String,
    pub color: String,
    pub score: u32,
}

impl User {
    pub fn new(username: &str) -> Self {
        let token = Uuid::new_v4().to_string();
        let uuid = Uuid::new_v4();
        let encoded_id = base62::encode(uuid.as_u128());

        Self {
            id: encoded_id,
            username: username.to_string(),
            token,
            color: color_to_hex(string_to_color(username)),
        }
    }
}

impl ToRedisArgs for User {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(b"id");
        out.write_arg(self.username.as_bytes());

        out.write_arg(b"username");
        out.write_arg(self.username.as_bytes());
        out.write_arg(b"color");
        out.write_arg(self.color.as_bytes());
        out.write_arg(b"token");
        out.write_arg(self.token.as_bytes());
    }
}
