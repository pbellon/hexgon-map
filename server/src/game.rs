use serde::Serialize;

use crate::{
    coords::{cube_spiral_without_center, AxialCoords},
    grid::{generate_tilemap, GridSettings, TileData, TileMap},
    user::{PublicUser, User},
};

#[derive(Serialize, Debug, Clone)]
pub struct GameData {
    tiles: TileMap,
    settings: GridSettings,
    users: Vec<User>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PublicGameData {
    settings: GridSettings,
    tiles: Vec<(AxialCoords, TileData)>,
    users: Vec<PublicUser>,
}

impl GameData {
    pub fn score_of_user(&self, user_id: &str) -> u32 {
        let nb_tiles = self
            .tiles
            .iter()
            .filter(|(_, tile)| tile.user_id == Some(user_id.to_string()))
            .count();
        nb_tiles as u32
    }

    pub fn as_public(&self) -> PublicGameData {
        PublicGameData {
            tiles: self
                .tiles
                .iter()
                .map(|(coords, tile)| (*coords, (*tile).clone()))
                .collect(),
            users: self.users.iter().map(|u| u.as_public()).collect(),
            settings: self.settings.clone(),
        }
    }

    pub fn new(radius: i32) -> Self {
        Self {
            settings: GridSettings { radius },
            tiles: generate_tilemap(radius),
            users: Vec::new(),
        }
    }

    fn get(&self, coords: &AxialCoords) -> Option<&TileData> {
        self.tiles.get(coords)
    }

    pub fn insert(&mut self, coords: AxialCoords, tile: TileData) -> Option<TileData> {
        self.tiles.insert(coords, tile)
    }

    /// Updates a neighbor tile's strength and ownership, if needed, based on the current context.
    fn update_neighbor(
        &mut self,
        coords: AxialCoords,
        neighbor: TileData,
        click_user_id: &str,
        current_user_id: &Option<String>,
        current_tile_strength: &mut u8,
        updated_tiles: &mut Vec<(AxialCoords, TileData)>,
    ) {
        // Case 1: Lower neighbor strength if it's owned by the same owner as the current tile
        if neighbor.user_id == *current_user_id && neighbor.strength > 1 {
            let new_neighbor = TileData {
                strength: neighbor.strength - 1,
                user_id: neighbor.user_id.clone(),
            };
            self.insert(coords, new_neighbor.clone());
            updated_tiles.push((coords, new_neighbor));
        }
        // Case 2: Increase neighbor strength if it's owned by the clicking user
        else if neighbor.user_id == Some(click_user_id.to_string()) {
            *current_tile_strength += 1;
            let new_neighbor = TileData {
                strength: neighbor.strength + 1,
                user_id: neighbor.user_id.clone(),
            };
            self.insert(coords, new_neighbor.clone());
            updated_tiles.push((coords, new_neighbor));
        }
    }
    pub fn handle_click(
        &mut self,
        coords: AxialCoords,
        click_user_id: &str,
    ) -> Vec<(AxialCoords, TileData)> {
        let mut updated_tiles = Vec::new();

        // If the tile exists
        if let Some(current_tile) = self.get(&coords).cloned() {
            let current_owner = current_tile.user_id.clone();

            // If the tile is not owned by the clicking user
            if current_owner != Some(click_user_id.to_string()) {
                let mut remaining_strength = current_tile.strength;

                // Adjust current tile's strength
                if remaining_strength > 0 {
                    remaining_strength -= 1;
                }

                if remaining_strength == 0 {
                    // Prepare to take ownership
                    remaining_strength = 1;

                    // Process adjacent tiles
                    let adjacent_tiles: Vec<(AxialCoords, TileData)> =
                        cube_spiral_without_center(&coords.as_cube(), 2)
                            .into_iter()
                            .filter_map(|cube_coords| {
                                let adjacent_coords = cube_coords.as_axial();
                                self.get(&adjacent_coords)
                                    .cloned()
                                    .map(|tile| (adjacent_coords, tile))
                            })
                            .collect();

                    for (adjacent_coords, adjacent_tile) in adjacent_tiles {
                        self.update_neighbor(
                            adjacent_coords,
                            adjacent_tile,
                            &click_user_id,
                            &current_owner,
                            &mut remaining_strength,
                            &mut updated_tiles,
                        );
                    }

                    // Transfer ownership of the current tile
                    let new_tile = TileData {
                        user_id: Some(click_user_id.to_string()),
                        strength: remaining_strength,
                    };
                    self.insert(coords, new_tile.clone());
                    updated_tiles.push((coords, new_tile));
                } else {
                    // Update current tile without changing ownership
                    let new_tile = TileData {
                        strength: remaining_strength,
                        user_id: current_owner,
                    };
                    self.insert(coords, new_tile.clone());
                    updated_tiles.push((coords, new_tile));
                }
            }
        }

        updated_tiles
    }

    pub fn create_user(&mut self, username: String) -> User {
        let user = User::new(username);
        self.users.push(user.clone());
        user
    }
}
