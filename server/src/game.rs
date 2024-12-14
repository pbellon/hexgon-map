use std::{cmp::max, collections::HashMap};

use serde::Serialize;

use crate::{
    coords::{cube_ring, AxialCoords},
    grid::{generate_tilemap, GridSettings, InnerTileData, TileData, TileMap},
    user::{PublicUser, User},
};

#[derive(Serialize, Debug, Clone)]
pub struct PublicGameData {
    settings: GridSettings,
    tiles: Vec<(i32, i32, u8, Option<String>)>,
    users: Vec<PublicUser>,
}

#[derive(Serialize, Debug, Clone)]
pub struct GameData {
    pub tiles: TileMap,
    pub settings: GridSettings,
    pub users: Vec<User>,
}

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

impl GameData {
    pub fn score_of_user(&self, user_id: &str) -> u32 {
        let nb_tiles = self
            .tiles
            .iter()
            .filter(|(_, tile)| tile.user_id == Some(user_id.to_string()))
            .count();
        nb_tiles as u32
    }

    pub fn computed_tile(&self, coords: &AxialCoords, tile: &InnerTileData) -> TileData {
        if let Some(user_id) = &tile.user_id {
            let nb_neighboors = self
                .contiguous_neighboors_of_tile(coords, &user_id, 2)
                .len() as u8;

            let strength = 1 + nb_neighboors - tile.damage;

            return TileData {
                strength,
                user_id: Some(user_id.clone()),
            };
        }

        TileData {
            strength: 0,
            user_id: None,
        }
    }

    pub fn as_public(&self) -> PublicGameData {
        PublicGameData {
            tiles: self
                .tiles
                .iter()
                .map(|(coords, tile)| {
                    let computed = self.computed_tile(coords, tile);
                    (coords.q, coords.r, computed.strength, computed.user_id)
                })
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

    fn get(&self, coords: &AxialCoords) -> Option<&InnerTileData> {
        self.tiles.get(coords)
    }

    pub fn insert(&mut self, coords: AxialCoords, tile: InnerTileData) -> Option<InnerTileData> {
        self.tiles.insert(coords, tile)
    }

    /// Returns all tiles that are contiguous to the given `coords`, i.e., all "connected" tiles next to `coords`
    /// that are owned by the specified `user_id`. The tile at `coords` *MUST* already be owned by `user_id`.
    /// otherwise will return empty Vec.
    fn contiguous_neighboors_of_tile(
        &self,
        tile_coords: &AxialCoords,
        user_id: &str,
        radius: u8,
    ) -> Vec<(AxialCoords, InnerTileData)> {
        let mut processed_map: HashMap<AxialCoords, bool> = HashMap::new();
        let mut results = Vec::new();
        let user_id_str = user_id.to_string();
        let mut to_check = vec![tile_coords.clone()];

        for _ in 0..radius {
            let mut next_to_check = Vec::new();

            // Temporarily take the value of `to_check` to avoid the borrowing issue
            for coords_to_check in std::mem::take(&mut to_check) {
                let ring = cube_ring(&coords_to_check.as_cube(), 1);

                for rc in ring {
                    // exclude given `tile_coords` out of results
                    if rc.as_axial() != *tile_coords {
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
            }
            to_check = next_to_check;
        }
        results
    }

    // TODO: change algorithm to avoid resetting strength when tile disappear
    pub fn handle_click(
        &mut self,
        coords: &AxialCoords,
        click_user_id: &str,
    ) -> Vec<(AxialCoords, TileData)> {
        let mut updated_tiles: Vec<(AxialCoords, InnerTileData)> = Vec::new();

        // If the tile exists
        if let Some(current_tile) = self.get(&coords).cloned() {
            let mut nb_neighboors = 0;
            let mut damage = current_tile.damage as i8;

            if let Some(current_tile_owner) = current_tile.user_id.clone() {
                nb_neighboors = self
                    .contiguous_neighboors_of_tile(coords, &current_tile_owner, 2)
                    .len() as i8;
            }

            // If the tile is not owned by the clicking user
            if current_tile.user_id.clone() != Some(click_user_id.to_string()) {
                // raise damage only if on a tile owned by another user,
                // do that to avoid issue with remaining_strength calculus below
                let remaining_strength: i8;

                // when clicking on an owned tile => raise damage
                if let Some(_) = current_tile.user_id.clone() {
                    damage += 1;
                    remaining_strength = max(0, 1 + nb_neighboors - damage);
                } else {
                    // if no owner set, no strength so ownership can be took directly
                    remaining_strength = 0;
                }

                // Handle the tile change in ownership
                if remaining_strength == 0 {
                    let new_tile = InnerTileData {
                        user_id: Some(click_user_id.to_string()),
                        damage: 0,
                    };

                    // 0 => Directly insert tile with new user_id to ease strength computing below
                    self.insert(coords.clone(), new_tile.clone());
                    updated_tiles.push((*coords, new_tile));

                    // 1 => Append former owner's contiguous tiles for client notification, strength will be recomputed at the end
                    if let Some(former_owner_id) = current_tile.user_id.clone() {
                        updated_tiles.append(&mut self.contiguous_neighboors_of_tile(
                            &coords,
                            &former_owner_id,
                            2,
                        ));
                    }

                    // 2 => append new owner's tiles to `update_tiles` vec, will compute final strength at the end
                    updated_tiles.append(&mut self.contiguous_neighboors_of_tile(
                        &coords,
                        click_user_id,
                        2,
                    ));
                } else {
                    let new_damage = current_tile.damage + 1;
                    // Update current tile without changing ownership, not yet "destroyed"
                    // but with augmented damage
                    let new_tile = InnerTileData {
                        damage: new_damage,
                        user_id: current_tile.user_id.clone(),
                    };
                    self.insert(*coords, new_tile.clone());

                    updated_tiles.push((coords.clone(), new_tile));
                }
            } else {
                // => checking if diminish damage, in other word we "heal" the tile
                if current_tile.damage > 0 {
                    let new_tile = InnerTileData {
                        damage: current_tile.damage - 1,
                        user_id: current_tile.user_id.clone(),
                    };
                    self.insert(coords.clone(), new_tile.clone());
                    updated_tiles.push((*coords, new_tile));
                }
            }
        }

        updated_tiles
            .iter()
            .map(|(coords, tile)| (coords.clone(), self.computed_tile(coords, tile)))
            .collect()
    }

    pub fn create_user(&mut self, username: String) -> User {
        let user = User::new(username);
        self.users.push(user.clone());
        user
    }
}
