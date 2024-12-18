use std::{
    cmp::max,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use ::futures::future;
use serde::Serialize;
use tokio::sync::RwLock;

use crate::{
    config::GameConfig,
    coords::{self, AxialCoords, PrecomputedNeighbors},
    grid::{GridSettings, InnerTileData, TileData, TileMap},
    user::{GameUsers, PublicUser},
    utils::create_benchmark_game_data,
};

#[derive(Serialize, Debug, Clone)]
pub struct PublicGameData {
    settings: GridSettings,
    tiles: Vec<(i32, i32, u8, String)>,
    users: Vec<PublicUser>,
}

#[derive(Debug, Clone)]
pub struct GameData {
    precomputed_neighbors: PrecomputedNeighbors,
    pub tiles: Arc<RwLock<TileMap>>,
    pub settings: GridSettings,
}

impl GameData {
    pub fn all_grid_coords(&self) -> Vec<AxialCoords> {
        self.precomputed_neighbors.keys().cloned().collect()
    }

    pub async fn init_from_config(config: &GameConfig, users: &GameUsers) -> Self {
        if config.use_benchmark_data {
            let user = users.register_user("benchmark-user").await;
            return create_benchmark_game_data(&user, config.grid_radius as i32).await;
        }

        Self::new(config.grid_radius as i32)
    }

    pub async fn score_of_user(&self, user_id: &str) -> u32 {
        let tiles = self.tiles.read().await;
        let nb_tiles = tiles
            .iter()
            .filter(|(_, tile)| &tile.user_id == user_id)
            .count();
        nb_tiles as u32
    }

    /// Returns all tiles that are contiguous to the given `coords`, i.e., all "connected" tiles next to `coords`
    /// that are owned by the specified `user_id`.
    pub async fn contiguous_neighbors_of_tile(
        &self,
        tile_coords: &AxialCoords,
        user_id: &str,
        radius: u8,
    ) -> (Vec<(AxialCoords, InnerTileData)>, u8) {
        let mut count = 0;
        let mut processed_set: HashSet<AxialCoords> = HashSet::new();
        let mut results = Vec::new();
        let mut to_check = vec![*tile_coords];

        // log::info!("ready to start reading tiles");

        let tiles = self.tiles.read().await;

        for _ in 0..radius {
            let mut next_to_check = Vec::new();

            // Step 2: Iterate over `to_check` and decouple access
            for coords_to_check in to_check.drain(..) {
                // Step 3: Fetch neighbors before checking `tiles` to avoid nested access
                if let Some(ring) = self.precomputed_neighbors.get(&coords_to_check) {
                    // log::info!("Ring => {ring:?}");

                    let filtered_neighbors: Vec<AxialCoords> = ring
                        .iter()
                        .filter_map(|rc| {
                            rc.and_then(|drc| {
                                // Avoid re-processing the same tile
                                if drc == *tile_coords || processed_set.contains(&drc) {
                                    return None;
                                }

                                // Check if the tile belongs to the same user
                                if let Some(tile) = tiles.get(&drc) {
                                    if tile.user_id == user_id {
                                        return Some(drc);
                                    }
                                }
                                None
                            })
                        })
                        .collect();

                    // Step 4: Push valid neighbors to results and mark as processed
                    for neighbor in filtered_neighbors {
                        processed_set.insert(neighbor);
                        results.push((
                            neighbor,
                            tiles
                                .get(&neighbor)
                                .expect("Tile should exist after checking")
                                .clone(),
                        ));
                        count += 1;
                        next_to_check.push(neighbor);
                    }
                }
            }
            to_check = next_to_check;
        }

        // log::info!("Results => {results:?}, Count => {count:?}");

        (results, count)
    }

    pub async fn computed_tile(&self, coords: &AxialCoords, tile: &InnerTileData) -> TileData {
        let nb_neighboors = self
            .contiguous_neighbors_of_tile(coords, &tile.user_id, 2)
            .await
            .1;
        let strength = 1 + nb_neighboors - tile.damage;

        TileData {
            strength,
            user_id: tile.user_id.to_string(),
        }
    }

    pub async fn as_public(&self, users: &GameUsers) -> PublicGameData {
        let tiles = self.tiles.read().await;
        let futures: Vec<_> = tiles
            .iter()
            .map(|(coords, tile)| async move {
                let computed = self.computed_tile(coords, tile).await;
                (coords.q, coords.r, computed.strength, computed.user_id)
            })
            .collect();

        PublicGameData {
            tiles: future::join_all(futures).await,
            users: users.as_public().await,
            settings: self.settings.clone(),
        }
    }

    pub fn new(radius: i32) -> Self {
        let precomputed_neighbors = coords::compute_neighboors(radius);

        Self {
            settings: GridSettings { radius },
            tiles: Arc::new(RwLock::new(HashMap::new())),
            precomputed_neighbors,
        }
    }

    pub async fn get_tile(&self, coords: &AxialCoords) -> Option<InnerTileData> {
        let tiles = self.tiles.read().await;
        tiles.get(coords).cloned()
    }

    pub async fn handle_click(
        &self,
        coords: &AxialCoords,
        click_user_id: &str,
    ) -> Vec<(AxialCoords, TileData)> {
        let mut updated_tiles: Vec<(AxialCoords, InnerTileData)> = Vec::new();

        // If the tile exists (aka is owned by someone)
        if let Some(current_tile) = self.get_tile(coords).await {
            let mut updated_tile = current_tile.clone();

            let nb_neighboors = self
                .contiguous_neighbors_of_tile(coords, &current_tile.user_id, 2)
                .await
                .1 as i8;

            let mut damage = current_tile.damage as i8;

            // If the tile is not owned by the clicking user
            if current_tile.user_id != click_user_id {
                // raise damage only if on a tile owned by another user,
                // do that to avoid issue with remaining_strength calculus below
                let remaining_strength: i8;

                // log::info!("Tile clicked not owned by current user");

                // when clicking on a tile owned by someone => raise damage
                damage += 1;
                remaining_strength = max(0, 1 + nb_neighboors - damage);

                // Handle the tile change in ownership
                if remaining_strength == 0 {
                    updated_tile.user_id = click_user_id.to_string();
                    updated_tile.damage = 0;

                    // 0 => Directly insert tile with new user_id to ease strength computing below
                    {
                        // log::info!("Directly take ownership");
                        let mut tiles_w = self.tiles.write().await;
                        tiles_w.insert(*coords, updated_tile.clone());
                    }

                    // 1 => Append former owner's contiguous tiles for client notification, strength will be recomputed at the end
                    updated_tiles.append(
                        &mut self
                            .contiguous_neighbors_of_tile(&coords, &current_tile.user_id, 2)
                            .await
                            .0,
                    );

                    // log::info!("request user's neighbors tiles to update: {req_user_tiles:?}");

                    // 2 => append new owner's tiles to `update_tiles` vec, will compute final strength at the end
                    updated_tiles.append(
                        &mut self
                            .contiguous_neighbors_of_tile(&coords, click_user_id, 2)
                            .await
                            .0,
                    );
                } else {
                    // Update current tile without changing ownership, not yet "destroyed"
                    // but with augmented damage
                    updated_tile.damage += 1;
                    {
                        let mut tiles_w = self.tiles.write().await;
                        tiles_w.insert(*coords, updated_tile.clone());
                    }
                }
            } else {
                // Clicking user clicks on its tile

                // if has some damage => heals its tile
                if current_tile.damage > 0 {
                    updated_tile.damage -= 1;
                    {
                        let mut tiles_w = self.tiles.write().await;
                        tiles_w.insert(*coords, updated_tile.clone());
                    }
                }
            }

            // append new_tile to updated tiles if changed (either damage change or ownership change)
            if updated_tile.damage != current_tile.damage
                || updated_tile.user_id != current_tile.user_id
            {
                updated_tiles.push((*coords, updated_tile))
            }
        } else {
            // if not then we create the tile
            let new_tile = InnerTileData {
                user_id: click_user_id.to_string(),
                damage: 0,
            };
            {
                let mut tiles_w = self.tiles.write().await;
                tiles_w.insert(coords.clone(), new_tile.clone());
            }

            // then append it to updated_tiles result
            updated_tiles.push((coords.clone(), new_tile.clone()));
            updated_tiles.append(
                &mut self
                    .contiguous_neighbors_of_tile(&coords, click_user_id, 2)
                    .await
                    .0,
            );

            // then append its neighboors to have new strength
        }
        // otherwise
        let futures: Vec<_> = updated_tiles
            .iter()
            .map(|(coords, tile)| async move {
                (coords.to_owned(), self.computed_tile(coords, tile).await)
            })
            .collect();

        future::join_all(futures).await
    }
}
