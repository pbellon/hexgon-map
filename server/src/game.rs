use std::collections::HashMap;

use serde::Serialize;

use crate::{
    coords::{cube_ring, cube_spiral_without_center, AxialCoords},
    grid::{generate_tilemap, GridSettings, TileData, TileMap},
    user::{PublicUser, User},
};

#[derive(Serialize, Debug, Clone)]
pub struct PublicGameData {
    settings: GridSettings,
    tiles: Vec<(AxialCoords, TileData)>,
    users: Vec<PublicUser>,
}

#[derive(Serialize, Debug, Clone)]
pub struct GameData {
    tiles: TileMap,
    settings: GridSettings,
    users: Vec<User>,
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

    /// Returns all tiles that are contiguous to the given `coords`, i.e., all "connected" tiles next to `coords`
    /// that are owned by the specified `user_id`. The tile at `coords` *MUST* already be owned by `user_id`.
    /// otherwise will return empty Vec.
    fn contiguous_neighboors_of_tile(
        &self,
        coords: &AxialCoords,
        user_id: &str,
        radius: u8,
    ) -> Vec<(AxialCoords, TileData)> {
        let mut processed_map: HashMap<AxialCoords, bool> = HashMap::new();
        let mut results = Vec::new();
        let user_id_str = user_id.to_string();
        if let Some(t) = self.get(coords) {
            if t.user_id == Some(user_id.to_string()) {
                processed_map.insert(coords.clone(), true);
                results.push((coords.clone(), t.clone()));
                let mut to_check = vec![coords.clone()];

                for _ in 0..radius {
                    let mut next_to_check = Vec::new();

                    for coord in &to_check {
                        let ring = cube_ring(&coord.as_cube(), 1);

                        for rc in ring {
                            if let Some(tile) = self.tiles.get(&rc.as_axial()) {
                                if tile.user_id == Some(user_id_str.clone()) {
                                    let axial_coords = rc.as_axial();
                                    if processed_map.get(&axial_coords).is_none() {
                                        next_to_check.push(axial_coords);
                                        results.push((axial_coords, tile.clone()));
                                        processed_map.insert(axial_coords, true);
                                    }
                                }
                            }
                        }
                    }

                    to_check = next_to_check;
                }
            }
        }

        results
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
            let mut remaining_strength = current_tile.strength;

            // If the tile is not owned by the clicking user
            if current_owner != Some(click_user_id.to_string()) {
                // Adjust current tile's strength
                if remaining_strength > 0 {
                    remaining_strength -= 1;
                }

                // Handle the tile change in ownership
                if remaining_strength == 0 {
                    // 0 => Directly insert tile with new user_id to ease strength computing below
                    self.insert(
                        coords,
                        TileData {
                            user_id: Some(click_user_id.to_string()),
                            strength: 1,
                        },
                    );

                    // **Step 1**: Process tiles of the former owner (if any)
                    if let Some(former_owner_id) = current_owner.clone() {
                        for (adjacent_coords, adjacent_tile) in
                            self.contiguous_neighboors_of_tile(&coords, &former_owner_id, 2)
                        {
                            let strength = self
                                .contiguous_neighboors_of_tile(
                                    &adjacent_coords,
                                    &former_owner_id,
                                    2,
                                )
                                .len() as u8;

                            let new_tile = TileData {
                                strength,
                                user_id: adjacent_tile.user_id.clone(),
                            };
                            self.insert(adjacent_coords, new_tile.clone());
                            updated_tiles.push((adjacent_coords, new_tile));
                        }
                    }

                    // **Step 2**: Process tiles of the new owner (this will include clicked tile)
                    for (click_user_tile_coords, click_user_tile) in
                        self.contiguous_neighboors_of_tile(&coords, click_user_id, 2)
                    {
                        // recompute strength at this point
                        let strength = self
                            .contiguous_neighboors_of_tile(
                                &click_user_tile_coords,
                                &click_user_id,
                                2,
                            )
                            .len() as u8;
                        let new_tile = TileData {
                            strength,
                            user_id: click_user_tile.user_id.clone(),
                        };
                        self.insert(click_user_tile_coords, new_tile.clone());
                        updated_tiles.push((click_user_tile_coords, new_tile));
                    }
                } else {
                    // Update current tile without changing ownership, not yet "destroyed"
                    // but with lowered strength
                    let new_tile = TileData {
                        strength: remaining_strength,
                        user_id: current_owner,
                    };
                    self.insert(coords, new_tile.clone());
                    updated_tiles.push((coords, new_tile));
                }
            } else {
                // all contiguous `click_user_id`'s tiles around `coords` within a radius of 2
                let user_contiguous_tiles_around_coords =
                    self.contiguous_neighboors_of_tile(&coords, click_user_id, 2);

                // check if clicked tile was previously attacked and raise health if so,
                // allowing user to "repair" its tiles
                if remaining_strength < user_contiguous_tiles_around_coords.len() as u8 {
                    // Update current tile without changing ownership
                    let new_tile = TileData {
                        strength: remaining_strength + 1,
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
