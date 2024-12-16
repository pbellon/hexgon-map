use crate::{game::GameData, grid::InnerTileData};

pub fn create_benchmark_game_data(radius: i32) -> GameData {
    let mut data = GameData::new(radius);
    let user = data.create_user("benchmark-user".to_string());
    let keys: Vec<_> = data.tiles.keys().cloned().collect();

    for coords in keys {
        data.insert(
            coords,
            InnerTileData {
                user_id: Some(user.id.clone()),
                damage: 0,
            },
        );
    }

    data
}
