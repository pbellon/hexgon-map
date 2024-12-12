// Grid/Layout related functions

// fn generate_grid(cols: i32, rows: i32) -> Vec<CubeCoords> {

// }

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::coords::{cube_spiral, AxialCoords, CubeCoords};

pub fn generate_tilemap(radius: i32) -> TileMap {
    let mut map = TileMap::new();
    let center = CubeCoords::center();

    for coords in cube_spiral(&center, radius, true) {
        map.insert(coords.as_axial(), TileData::empty());
    }
    map
}

/// Data associated to an hexagon in the grid
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TileData {
    /// Owner of the tile, None => No owner yet
    pub user_id: Option<String>,
    /// Strength represents the number of clicks needed in order to take ownership
    pub strength: u8,
}

impl TileData {
    pub fn empty() -> Self {
        TileData {
            user_id: None,
            strength: 0,
        }
    }
}

pub type TileMap = HashMap<AxialCoords, TileData>;

#[derive(Serialize, Debug, Clone)]
pub struct GridSettings {
    pub radius: i32,
}
