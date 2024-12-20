use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};

use rand::seq::SliceRandom; // you may need to adjust version depending on your Rust version

use ::futures::future;
use serde::{Deserialize, Serialize};

use crate::{
    config::GameConfig,
    coords::{self, AxialCoords, PrecomputedNeighbors},
    store::RedisHandler,
    user::{GameUsers, PublicUser},
    utils::create_benchmark_game_data,
};

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

#[derive(Copy, Serialize, Debug, Clone)]
pub struct GridSettings {
    pub radius: u32,
}

#[derive(Serialize, Debug, Clone)]
pub struct PublicGameData {
    settings: GridSettings,
    tiles: Vec<(i32, i32, u8, String)>,
    users: Vec<PublicUser>,
}

pub type TileMap = HashMap<AxialCoords, InnerTileData>;

#[derive(Debug, Clone)]
pub struct GameData {
    pub precomputed_neighbors: PrecomputedNeighbors,
    precomputed_batches: Vec<Vec<AxialCoords>>,
    pub settings: GridSettings,
}

impl GameData {
    pub fn get_batch_list(&self) -> Vec<usize> {
        let batches_len = self.precomputed_batches.len();
        let mut list = (0..batches_len).collect::<Vec<_>>();
        let mut rng = rand::thread_rng();
        list.shuffle(&mut rng);
        list
    }

    pub async fn compute_batch<R: RedisHandler>(
        &self,
        redis_client: &R,
        batch: usize,
    ) -> Result<Vec<(i32, i32, u8, String)>, String> {
        // Check if the batch exists
        if let Some(batch_coords) = self.precomputed_batches.get(batch) {
            let mut results = Vec::new();

            match redis_client.get_all_tiles(batch_coords.clone()).await {
                Ok(tiles) => {
                    for (coords, tile) in tiles {
                        match self.computed_tile(redis_client, &coords, &tile).await {
                            Ok(c) => {
                                results.push((coords.q, coords.r, c.strength, c.user_id));
                            }
                            Err(e) => {
                                log::error!("Encounted error while computing batch tile: {e}");
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error encounted while fetching all coords: {e}");
                    return Err(e.to_string());
                }
            }

            return Ok(results);

            // for coords in batch_coords.iter() {
            //     match redis_client.get_tile(coords).await {
            //         Ok(Some(tile)) => match self.computed_tile(redis_client, coords, &tile).await {
            //             Ok(computed) => {
            //                 results.push((coords.q, coords.r, computed.strength, computed.user_id));
            //             }
            //             Err(e) => {
            //                 return Err(format!("Redis error: {e}"));
            //             }
            //         },
            //         Ok(None) => {
            //             continue;
            //         }
            //         Err(e) => {
            //             return Err(format!("Redis error: {e}"));
            //         }
            //     }
            // }
        }

        Err(format!("Batch {} does not exist", batch))
    }

    pub fn all_grid_coords(&self) -> Vec<AxialCoords> {
        self.precomputed_neighbors.keys().cloned().collect()
    }

    pub async fn init_from_config<R: RedisHandler>(
        redis_client: &R,
        config: &GameConfig,
        users: &GameUsers,
    ) -> Self {
        if config.use_benchmark_data {
            let user = users.register_user("benchmark-user").await;
            return create_benchmark_game_data(
                redis_client,
                &user,
                config.grid_radius,
                config.grid_batch_div,
            )
            .await;
        }

        Self::new(config.grid_radius, config.grid_batch_div)
    }

    /// Returns all tiles that are contiguous to the given `coords`, i.e., all "connected" tiles next to `coords`
    /// that are owned by the specified `user_id`.
    pub async fn contiguous_neighbors_of_tile<R: RedisHandler>(
        &self,
        redis_client: &R,
        tile_coords: &AxialCoords,
        user_id: &str,
        radius: u8,
    ) -> Result<(Vec<(AxialCoords, InnerTileData)>, u8), redis::RedisError> {
        let mut count = 0;
        let mut processed_set: HashSet<AxialCoords> = HashSet::new();
        let mut results = Vec::new();
        let mut to_check = vec![*tile_coords];

        for _ in 0..radius {
            let mut next_to_check = Vec::new();

            for coords_to_check in to_check.drain(..) {
                if let Some(ring) = self.precomputed_neighbors.get(&coords_to_check) {
                    let mut filtered_neighbors = Vec::new();

                    let mut coords: Vec<AxialCoords> = ring
                        .iter()
                        .filter_map(|rc| {
                            rc.and_then(|drc| {
                                if &drc == tile_coords || processed_set.contains(&drc) {
                                    return None;
                                }

                                return Some(drc);
                            })
                        })
                        .collect();

                    // fetch all tiles at once
                    match redis_client.get_all_tiles(coords).await {
                        Ok(tiles) => {
                            filtered_neighbors.append(
                                &mut tiles
                                    .iter()
                                    .filter_map(|(c, t)| {
                                        if t.user_id == user_id {
                                            return Some((c.clone(), t.clone()));
                                        }

                                        return None;
                                    })
                                    .collect(),
                            );
                        }
                        Err(e) => {
                            log::error!("An error occured fetching neighboors: {e}");
                            return Err(e);
                        }
                    };

                    // Add valid neighbors to results and mark them as processed
                    for (neighbor, tile_data) in filtered_neighbors {
                        processed_set.insert(neighbor);
                        results.push((neighbor, tile_data));
                        count += 1;
                        next_to_check.push(neighbor);
                    }
                }
            }

            to_check = next_to_check;
        }

        Ok((results, count))
    }

    pub async fn computed_tile<R: RedisHandler>(
        &self,
        redis_client: &R,
        coords: &AxialCoords,
        tile: &InnerTileData,
    ) -> Result<TileData, redis::RedisError> {
        let nb_neighboors = match self
            .contiguous_neighbors_of_tile(redis_client, coords, &tile.user_id, 2)
            .await
        {
            Ok(vec) => vec.1,
            Err(e) => {
                return Err(e);
            }
        };

        let strength = 1 + nb_neighboors - tile.damage;

        Ok(TileData {
            strength,
            user_id: tile.user_id.to_string(),
        })
    }

    pub fn new(radius: u32, batch_rows_and_cols: u8) -> Self {
        let precomputed_neighbors = coords::compute_neighboors(radius);

        Self {
            precomputed_batches: coords::create_parallelogram_coords_batches(
                batch_rows_and_cols,
                batch_rows_and_cols,
                radius,
            ),
            settings: GridSettings { radius },
            precomputed_neighbors,
        }
    }

    pub async fn get_tile<R: RedisHandler>(
        &self,
        redis_client: &R,
        coords: &AxialCoords,
    ) -> Result<Option<InnerTileData>, redis::RedisError> {
        redis_client.get_tile(coords).await
    }

    pub async fn handle_click<R: RedisHandler>(
        &self,
        redis_client: &R,
        coords: &AxialCoords,
        click_user_id: &str,
    ) -> Result<Vec<(AxialCoords, TileData)>, redis::RedisError> {
        let mut updated_tiles: Vec<(AxialCoords, InnerTileData)> = Vec::new();

        // If the tile exists (aka is owned by someone)
        if let Some(current_tile) = redis_client.get_tile(coords).await.expect(&format!(
            "Redis error occured why retrieving tile at {coords:?}",
        )) {
            log::info!("Found tile at {coords:?} => {current_tile:?}");

            let mut updated_tile = current_tile.clone();

            let nb_neighboors = match self
                .contiguous_neighbors_of_tile(redis_client, coords, &current_tile.user_id, 2)
                .await
            {
                Ok(vec) => vec.1 as i8,
                Err(e) => {
                    return Err(e);
                }
            };

            let mut damage = current_tile.damage as i8;

            // If the tile is not owned by the clicking user
            if current_tile.user_id != click_user_id {
                // raise damage only if on a tile owned by another user,
                // do that to avoid issue with remaining_strength calculus below
                let remaining_strength: i8;

                log::info!("Tile clicked not owned by current user");

                // when clicking on a tile owned by someone => raise damage
                damage += 1;
                remaining_strength = max(0, 1 + nb_neighboors - damage);

                // Handle the tile change in ownership
                if remaining_strength == 0 {
                    updated_tile.user_id = click_user_id.to_string();
                    updated_tile.damage = 0;

                    // 0 => Directly insert tile with new user_id to ease strength computing below
                    redis_client
                        .set_tile(coords, updated_tile.clone())
                        .await
                        .expect(&format!(
                            "Could not update tile at {coords:?} with new user id"
                        ));

                    match self
                        .contiguous_neighbors_of_tile(
                            redis_client,
                            &coords,
                            &current_tile.user_id,
                            2,
                        )
                        .await
                    {
                        Ok((tiles, _)) => {
                            updated_tiles.append(&mut tiles.clone());
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }

                    // log::info!("request user's neighbors tiles to update: {req_user_tiles:?}");

                    // 2 => append new owner's tiles to `update_tiles` vec, will compute final strength at the end
                    let (tiles, _) = self
                        .contiguous_neighbors_of_tile(redis_client, &coords, click_user_id, 2)
                        .await
                        .unwrap();

                    updated_tiles.append(&mut tiles.clone());
                } else {
                    // Update current tile without changing ownership, not yet "destroyed"
                    // but with augmented damage
                    updated_tile.damage += 1;
                    redis_client
                        .set_tile(coords, updated_tile.clone())
                        .await
                        .expect(&format!(
                            "Could not update tile at {coords:?} to raise damage"
                        ));
                }
            } else {
                // Clicking user clicks on its tile

                // if has some damage => heals its tile
                if current_tile.damage > 0 {
                    updated_tile.damage -= 1;
                    redis_client
                        .set_tile(coords, updated_tile.clone())
                        .await
                        .expect(&format!(
                            "Could not update tile at {coords:?} to decrease damage"
                        ));
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

            match redis_client.set_tile(coords, new_tile.clone()).await {
                Ok(_) => {
                    updated_tiles.push((coords.clone(), new_tile.clone()));
                    // append its neighboors to have new strength
                    let (tiles, _) = self
                        .contiguous_neighbors_of_tile(redis_client, &coords, click_user_id, 2)
                        .await
                        .expect(&format!("Could not get contiguous neighbors at {coords:?}"));

                    updated_tiles.append(&mut tiles.clone());
                }
                Err(e) => {
                    log::error!("A redis error occured while updating tile at {coords:?}: {e}");
                    return Err(e);
                }
            }
        }

        // otherwise
        let futures: Vec<_> = updated_tiles
            .iter()
            .map(|(coords, tile)| async move {
                (
                    coords.to_owned(),
                    self.computed_tile(redis_client, coords, tile)
                        .await
                        .unwrap(),
                )
            })
            .collect();

        Ok(future::join_all(futures).await)
    }
}
