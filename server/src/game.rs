use serde::{Deserialize, Serialize};

use crate::{
    coords::AxialCoords,
    grid::{Grid, TileData},
    user::User,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GameData {
    grid: Grid,
    users: Vec<User>,
}

impl GameData {
    pub fn new(radius: i32) -> Self {
        Self {
            grid: Grid::new(radius),
            users: Vec::new(),
        }
    }

    pub fn insert(&mut self, coords: AxialCoords, tile: TileData) {
        self.grid.tiles.insert(coords, tile);
    }

    pub fn create_user(&mut self, username: Option<String>) -> String {
        let user = User::new(username);
        self.users.push(user.clone());
        user.id()
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }
}
