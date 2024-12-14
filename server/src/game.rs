use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    ops::Deref,
};

use serde::Serialize;

use crate::{
    coords::{cube_ring, cube_spiral, direct_neighbors, AxialCoords, CubeCoords},
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
    precomputed_neighbors: HashMap<AxialCoords, [Option<AxialCoords>; 6]>,
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

pub fn is_within_grid(coords: AxialCoords, radius: i32) -> bool {
    coords.q >= -radius && coords.q <= radius && coords.r >= -radius && coords.r <= radius
}
pub fn precompute_neighboors(radius: i32) -> HashMap<AxialCoords, [Option<AxialCoords>; 6]> {
    cube_spiral(&CubeCoords::center(), radius)
        .iter()
        .map(|coords| {
            let mut results = [None; 6]; // Use an array of Option<AxialCoords>
            let mut index = 0;

            for cc in direct_neighbors(&coords).iter() {
                let ac = cc.as_axial();
                if is_within_grid(ac, radius) {
                    results[index] = Some(ac);
                    index += 1;
                }
            }

            (coords.as_axial(), results)
        })
        .collect()
}

impl GameData {
    pub fn score_of_user(&self, user_id: &str) -> u32 {
        let user_id_owned = Some(user_id.to_string());
        let nb_tiles = self
            .tiles
            .iter()
            .filter(|(_, tile)| tile.user_id == user_id_owned)
            .count();
        nb_tiles as u32
    }

    pub fn computed_tile(&self, coords: &AxialCoords, tile: &InnerTileData) -> TileData {
        if let Some(user_id) = &tile.user_id {
            let nb_neighboors = self.contiguous_neighboors_of_tile(coords, &user_id, 2).1;
            let strength = 1 + nb_neighboors - tile.damage;

            return TileData {
                strength,
                user_id: Some(user_id.to_string()),
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
        let tiles = generate_tilemap(radius);
        let precomputed_neighbors = precompute_neighboors(radius);

        Self {
            settings: GridSettings { radius },
            tiles,
            users: Vec::new(),
            precomputed_neighbors,
        }
    }

    fn get(&self, coords: &AxialCoords) -> Option<&InnerTileData> {
        self.tiles.get(coords)
    }

    pub fn insert(&mut self, coords: AxialCoords, tile: InnerTileData) -> Option<InnerTileData> {
        self.tiles.insert(coords, tile)
    }

    /// Returns all tiles that are contiguous to the given `coords`, i.e., all "connected" tiles next to `coords`
    /// that are owned by the specified `user_id`.
    fn contiguous_neighboors_of_tile(
        &self,
        tile_coords: &AxialCoords,
        user_id: &str,
        radius: u8,
    ) -> (Vec<(AxialCoords, &InnerTileData)>, u8) {
        let mut count = 0;
        let mut processed_set: HashSet<AxialCoords> = HashSet::new();
        let mut results = Vec::new();
        let mut to_check = vec![*tile_coords];

        for _ in 0..radius {
            let mut next_to_check = Vec::new();

            // Temporarily take the value of `to_check` to avoid borrowing issue
            for coords_to_check in to_check.drain(..) {
                if let Some(ring) = self.precomputed_neighbors.get(&coords_to_check) {
                    next_to_check.extend(ring.iter().filter_map(|rc| {
                        if let Some(drc) = *rc {
                            if drc == *tile_coords {
                                return None;
                            }

                            if let Some(tile) = self.tiles.get(&drc) {
                                if tile.user_id.as_deref() == Some(user_id)
                                    && !processed_set.contains(&drc)
                                {
                                    processed_set.insert(drc);
                                    results.push((drc, tile));
                                    count += 1;
                                    return Some(drc);
                                }
                            }
                            return None;
                        }
                        None
                    }));
                }
            }
            to_check = next_to_check;
        }
        (results, count)
    }

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
                    .1 as i8;
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
                        let nbs = self
                            .contiguous_neighboors_of_tile(&coords, &former_owner_id, 2)
                            .0;

                        updated_tiles.append(
                            &mut nbs
                                .iter()
                                .map(|&(coords, tile_ref)| (coords, tile_ref.to_owned()))
                                .collect(),
                        );
                    }

                    // 2 => append new owner's tiles to `update_tiles` vec, will compute final strength at the end
                    updated_tiles.append(
                        &mut self
                            .contiguous_neighboors_of_tile(&coords, click_user_id, 2)
                            .0
                            .iter()
                            .map(|&(coords, tile_ref)| (coords, tile_ref.to_owned()))
                            .collect(),
                    );
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
            .map(|(coords, tile)| (coords.to_owned(), self.computed_tile(coords, tile)))
            .collect()
    }

    pub fn create_user(&mut self, username: String) -> User {
        let user = User::new(username);
        self.users.push(user.clone());
        user
    }
}
