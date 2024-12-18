// Grid/Layout related functions

// fn generate_grid(cols: i32, rows: i32) -> Vec<CubeCoords> {

// }

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::coords::AxialCoords;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InnerTileData {
    pub user_id: String,
    pub damage: u8,
}

/// Data associated to an hexagon in the grid
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TileData {
    /// Owner of the tile, None => No owner yet
    pub user_id: String,
    /// Strength represents the number of clicks needed in order to take ownership
    pub strength: u8,
}

pub type TileMap = HashMap<AxialCoords, InnerTileData>;

#[derive(Copy, Serialize, Debug, Clone)]
pub struct GridSettings {
    pub radius: i32,
}
