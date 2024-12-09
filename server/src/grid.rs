// Grid/Layout related functions

// fn generate_grid(cols: i32, rows: i32) -> Vec<CubeCoords> {

// }

use std::{collections::HashMap, sync::RwLock};

use serde::{Deserialize, Serialize};

use crate::coords::{cube_spiral, CubeCoords};

pub fn generate_grid(radius: i32) -> TileStore {
    let mut map = TileStore::new();
    let center = CubeCoords::new(0, 0, 0);
    for coords in cube_spiral(&center, radius) {
        map.insert(coords, TileData::empty());
    }

    map
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TileData {
    user_id: Option<String>,
    strength: u8,
}

impl TileData {
    pub fn empty() -> Self {
        TileData {
            user_id: None,
            strength: 0,
        }
    }
}

pub type TileStore = HashMap<CubeCoords, TileData>;

pub type TileStoreRwLock = RwLock<TileStore>;
