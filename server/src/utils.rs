use std::hash::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::user::User;
use crate::{game::GameData, grid::InnerTileData};

pub async fn create_benchmark_game_data(benchmark_user: &User, radius: i32) -> GameData {
    let data = GameData::new(radius);

    let keys: Vec<_> = {
        let tiles_r = data.tiles.read().await;
        tiles_r.keys().cloned().collect()
    };
    {
        let mut tiles_w = data.tiles.write().await;
        for coords in keys {
            tiles_w.insert(
                coords,
                InnerTileData {
                    user_id: benchmark_user.id.clone(),
                    damage: 0,
                },
            );
        }
    }
    data
}

pub fn string_to_color(input: &str) -> (u8, u8, u8) {
    // Hash the input string
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    // Extract RGB values from the hash
    let r = (hash & 0xFF) as u8; // First 8 bits
    let g = ((hash >> 8) & 0xFF) as u8; // Next 8 bits
    let b = ((hash >> 16) & 0xFF) as u8; // Next 8 bits

    (r, g, b)
}

pub fn color_to_hex(color: (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", color.0, color.1, color.2)
}
