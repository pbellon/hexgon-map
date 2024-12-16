use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::utils::{color_to_hex, string_to_color};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub color: String,
    pub id: String,
    pub username: String,
    token: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct PublicUser {
    id: String,
    username: String,
    color: String,
}

impl User {
    pub fn new(username: &str) -> (String, Self) {
        let token = Uuid::new_v4().to_string();
        let uuid = Uuid::new_v4();
        let encoded_id = base62::encode(uuid.as_u128());

        (
            token.clone(),
            Self {
                id: encoded_id,
                username: username.to_string(),
                token,
                color: color_to_hex(string_to_color(username)),
            },
        )
    }

    pub fn as_public(&self) -> PublicUser {
        PublicUser {
            id: self.id.clone(),
            color: self.color.clone(),
            username: self.username.clone(),
        }
    }
}

#[derive(Clone)]
pub struct GameUsers {
    tokens: Arc<RwLock<HashMap<String, String>>>,
    users: Arc<RwLock<Vec<User>>>,
}

impl GameUsers {
    // `&self` instead of `&mut self`
    pub async fn register_user(&self, username: &str) -> User {
        let (token, user) = User::new(username);
        let mut tokens = self.tokens.write().await;
        let mut users = self.users.write().await;

        tokens.insert(user.id.clone(), token);
        users.push(user.clone());

        user
    }

    // `&self` instead of consuming `self`
    pub async fn is_valid_token_for_user(&self, user_id: &str, token: &str) -> bool {
        let tokens = self.tokens.read().await;

        if let Some(user_token) = tokens.get(user_id) {
            return user_token == token;
        }

        false
    }

    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn as_public(&self) -> Vec<PublicUser> {
        let users = self.users.read().await;
        users.iter().map(|u| u.as_public()).collect()
    }
}
