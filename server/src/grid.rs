// Grid/Layout related functions

// fn generate_grid(cols: i32, rows: i32) -> Vec<CubeCoords> {

// }

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::coords::{cube_spiral, AxialCoords, CubeCoords};

pub fn generate_tilemap(radius: i32) -> TileMap {
    let mut map = TileMap::new();
    let center = CubeCoords::new(0, 0, 0);
    for coords in cube_spiral(&center, radius) {
        map.insert(coords.as_axial(), TileData::empty());
    }
    map
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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

pub type TileMap = HashMap<AxialCoords, TileData>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GridSettings {
    radius: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Grid {
    pub tiles: TileMap,
    settings: GridSettings,
}

impl Grid {
    pub fn new(radius: i32) -> Self {
        Self {
            settings: GridSettings { radius },
            tiles: generate_tilemap(radius),
        }
    }
}
