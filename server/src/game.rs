use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};

use rand::seq::SliceRandom; // you may need to adjust version depending on your Rust version

use serde::{Deserialize, Serialize};

use crate::{
    config::GameConfig,
    coords::{self, cube_spiral, is_within_grid, AxialCoords, PrecomputedNeighbors},
    store::RedisHandler,
    user::{PublicUser, User},
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

    pub async fn compute_batch<R, C>(
        &self,
        redis_client: &R,
        con: &mut C,
        batch: usize,
    ) -> Result<Vec<(i32, i32, u8, String)>, String>
    where
        C: redis::aio::ConnectionLike + Send,
        R: RedisHandler,
    {
        // Check if the batch exists
        if let Some(batch_coords) = self.precomputed_batches.get(batch) {
            let mut results = Vec::new();

            match redis_client
                .batch_get_tiles(con, batch_coords.clone())
                .await
            {
                Ok(tiles) => {
                    let mut temp_fetched_map: HashMap<AxialCoords, InnerTileData> = HashMap::new();

                    for (coords, tile) in tiles {
                        match self
                            .computed_tile(redis_client, con, &coords, &tile, &mut temp_fetched_map)
                            .await
                        {
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
        }

        Err(format!("Batch {} does not exist", batch))
    }

    pub fn all_grid_coords(&self) -> Vec<AxialCoords> {
        self.precomputed_neighbors.keys().cloned().collect()
    }

    pub async fn init_from_config<R, C>(con: &mut C, redis_client: &R, config: &GameConfig) -> Self
    where
        C: redis::aio::ConnectionLike + Send,
        R: RedisHandler,
    {
        if config.use_benchmark_data {
            let user = User::new("benchmark-user");

            let _ = redis_client.add_user(con, user.clone()).await.unwrap();

            return create_benchmark_game_data(
                con,
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
    pub fn contiguous_neighbors_of_tile(
        &self,
        prefetched: &HashMap<AxialCoords, InnerTileData>,
        tile_coords: &AxialCoords,
        user_id: &str,
        radius: u8,
    ) -> (Vec<(AxialCoords, InnerTileData)>, u8) {
        let mut count = 0;
        let mut processed_set: HashSet<AxialCoords> = HashSet::new();
        let mut results = Vec::new();
        let mut to_check = vec![*tile_coords];

        for _ in 0..radius {
            let mut next_to_check = Vec::new();

            for coords_to_check in to_check.drain(..) {
                if let Some(ring) = self.precomputed_neighbors.get(&coords_to_check) {
                    let filtered_neighbors: Vec<(AxialCoords, InnerTileData)> = ring
                        .iter()
                        .filter_map(|rc| {
                            rc.and_then(|drc| {
                                if &drc == tile_coords || processed_set.contains(&drc) {
                                    return None;
                                }

                                if let Some(nb) = prefetched.get(&drc) {
                                    if nb.user_id == user_id {
                                        return Some((drc.clone(), nb.clone()));
                                    }
                                }

                                return None;
                            })
                        })
                        .collect();

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

        (results, count)
    }

    /// helper fn to prefetch the HashMap<AxialCoords, InnerTileData>
    /// that will be used by `contiguous_neighbors_of_tile`
    pub async fn fetch_within<'a, R, C>(
        &self,
        redis_client: &R,
        con: &mut C,
        coords: &AxialCoords,
        previously_fetched: &mut HashMap<AxialCoords, InnerTileData>,
    ) -> redis::RedisResult<bool>
    where
        R: RedisHandler,
        C: redis::aio::ConnectionLike + Send + 'a,
    {
        let coords_to_fetch = cube_spiral(&coords.as_cube(), 2)
            .iter()
            .filter_map(|c| {
                let ac = c.as_axial();
                if is_within_grid(ac, self.settings.radius) && !previously_fetched.contains_key(&ac)
                {
                    return Some(ac);
                }
                return None;
            })
            .collect();

        let res = redis_client.batch_get_tiles(con, coords_to_fetch).await?;

        for (coord, data) in res.iter() {
            previously_fetched.insert(*coord, data.clone());
        }

        Ok(true)
    }

    pub async fn computed_tile<R, C>(
        &self,
        redis_client: &R,
        con: &mut C,
        coords: &AxialCoords,
        tile: &InnerTileData,
        prefetched: &mut HashMap<AxialCoords, InnerTileData>,
    ) -> redis::RedisResult<TileData>
    where
        R: RedisHandler,
        C: redis::aio::ConnectionLike + Send,
    {
        let _ = self
            .fetch_within(redis_client, con, coords, prefetched)
            .await
            .unwrap();

        let (_, nb_neighboors) =
            self.contiguous_neighbors_of_tile(prefetched, coords, &tile.user_id, 2);

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

    pub async fn handle_click<R, C>(
        &self,
        redis_client: &R,
        con: &mut C,
        click_coords: &AxialCoords,
        click_user_id: &str,
    ) -> Result<Vec<(AxialCoords, TileData)>, redis::RedisError>
    where
        R: RedisHandler,
        C: redis::aio::ConnectionLike + Send,
    {
        let mut updated_tiles: Vec<(AxialCoords, InnerTileData)> = Vec::new();

        // helpful hashmap to recompute strength and avoir additionnal redis access
        let mut tmp_hash = HashMap::new();

        let _ = self
            .fetch_within(redis_client, con, click_coords, &mut tmp_hash)
            .await;

        // If the tile exists (aka is owned by someone)
        if let Some(current_tile) = tmp_hash.get(click_coords).cloned() {
            let mut updated_tile = current_tile.clone();

            let current_owner = current_tile.user_id.clone();
            let mut damage = current_tile.clone().damage as i8;

            let (_, nb_neighboors) =
                { self.contiguous_neighbors_of_tile(&tmp_hash, click_coords, &current_owner, 2) };

            // If the tile is not owned by the clicking user
            if current_tile.user_id != click_user_id {
                // raise damage only if on a tile owned by another user,
                // do that to avoid issue with remaining_strength calculus below
                let remaining_strength: i8;

                // when clicking on a tile owned by someone => raise damage
                damage += 1;
                remaining_strength = max(0, 1 + nb_neighboors as i8 - damage);

                // Handle the tile change in ownership
                if remaining_strength == 0 {
                    updated_tile.user_id = click_user_id.to_string();
                    updated_tile.damage = 0;

                    // propagate change to redis shared state
                    tmp_hash.insert(click_coords.clone(), updated_tile.clone());

                    // 0 => insert tile with new user_id (effectively write the data in shared state)
                    let _ = redis_client
                        .set_tile(con, click_coords, updated_tile.clone())
                        .await
                        .expect(&format!(
                            "Could not update tile at {click_coords:?} with new user id"
                        ));

                    // 1. append former owner tiles to `update_tiles`
                    let (tiles, _) = {
                        self.contiguous_neighbors_of_tile(
                            &tmp_hash,
                            &click_coords,
                            &current_owner,
                            2,
                        )
                    };

                    updated_tiles.append(&mut tiles.clone());

                    // 2 => append new owner's tiles to `update_tiles` vec, will compute final strength at the end
                    let (tiles, _) = {
                        self.contiguous_neighbors_of_tile(
                            &tmp_hash,
                            &click_coords,
                            click_user_id,
                            2,
                        )
                    };
                    updated_tiles.append(&mut tiles.clone());
                } else {
                    // Update current tile without changing ownership, not yet "destroyed"
                    // but with augmented damage
                    updated_tile.damage += 1;
                    tmp_hash.insert(click_coords.clone(), updated_tile.clone());
                    let _ = redis_client
                        .set_tile(con, click_coords, updated_tile.clone())
                        .await
                        .expect(&format!(
                            "Could not update tile at {click_coords:?} to raise damage"
                        ));
                }
            } else {
                // Clicking user clicks on its tile

                // if has some damage => heals its tile
                if current_tile.damage > 0 {
                    updated_tile.damage -= 1;
                    tmp_hash.insert(click_coords.clone(), updated_tile.clone());
                    let _ = redis_client
                        .set_tile(con, click_coords, updated_tile.clone())
                        .await
                        .expect(&format!(
                            "Could not update tile at {click_coords:?} to decrease damage"
                        ));
                }
            }

            // append new_tile to updated tiles if changed (either damage change or ownership change)
            if updated_tile.damage != current_tile.damage
                || updated_tile.user_id != current_tile.user_id
            {
                updated_tiles.push((*click_coords, updated_tile))
            }
        } else {
            // if not then we create the tile
            let new_tile = InnerTileData {
                user_id: click_user_id.to_string(),
                damage: 0,
            };
            tmp_hash.insert(click_coords.clone(), new_tile.clone());

            match redis_client
                .set_tile(con, click_coords, new_tile.clone())
                .await
            {
                Ok(_) => {
                    updated_tiles.push((click_coords.clone(), new_tile.clone()));

                    // append its neighboors to have new strength
                    let (tiles, _) = self.contiguous_neighbors_of_tile(
                        &mut tmp_hash,
                        &click_coords,
                        click_user_id,
                        2,
                    );

                    updated_tiles.append(&mut tiles.clone());
                }
                Err(e) => {
                    log::error!(
                        "A redis error occured while updating tile at {click_coords:?}: {e}"
                    );
                    return Err(e);
                }
            }
        }

        let mut res = Vec::new();
        for (coords, tile) in updated_tiles {
            let computed = self
                .computed_tile(redis_client, con, &coords, &tile, &mut tmp_hash)
                .await
                .unwrap();

            res.push((coords.to_owned(), computed.clone()));
        }

        Ok(res)
    }
}
