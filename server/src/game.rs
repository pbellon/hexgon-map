use serde::{Deserialize, Serialize};

use crate::{grid::TileStore, user::User};

#[derive(Deserialize, Serialize, Debug)]
pub struct GameData {
    grid: TileStore,
    users: Vec<User>,
}

impl GameData {
    pub fn create_user(&self, username: Option<String>) -> User {
        let user = User::new();
        self.users.push(user);
        user
    }

    pub fn grid(&self) -> &TileStore {
        &self.grid
    }
}
