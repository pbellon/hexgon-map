use std::{
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

const DIRECTIONS: [CubeCoords; 6] = [
    CubeCoords { q: 1, r: 0, s: -1 },
    CubeCoords { q: 1, r: -1, s: 0 },
    CubeCoords { q: 0, r: -1, s: 1 },
    CubeCoords { q: -1, r: 0, s: 1 },
    CubeCoords { q: -1, r: 1, s: 0 },
    CubeCoords { q: 0, r: 1, s: -1 },
];

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub struct CubeCoords {
    q: i32,
    r: i32,
    s: i32,
}

impl Debug for CubeCoords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(q: {}, r:{}, s:{})", self.q, self.r, self.s)
    }
}

impl CubeCoords {
    pub fn new(q: i32, r: i32, s: i32) -> Self {
        Self { q, r, s }
    }

    pub fn center() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn as_axial(&self) -> AxialCoords {
        AxialCoords::new(self.q, self.r)
    }
}
#[allow(dead_code)]
pub fn cube_substract(a: &CubeCoords, b: &CubeCoords) -> CubeCoords {
    CubeCoords::new(a.q - b.q, a.r - b.r, a.s - b.s)
}

pub fn cube_add(a: &CubeCoords, b: &CubeCoords) -> CubeCoords {
    CubeCoords::new(a.q + b.q, a.r + b.r, a.s + b.s)
}

pub fn cube_scale(a: &CubeCoords, factor: i32) -> CubeCoords {
    CubeCoords::new(a.q * factor, a.r * factor, a.s * factor)
}

pub fn cube_direction(dir: usize) -> CubeCoords {
    DIRECTIONS[dir]
}

pub fn cube_neighbor(coords: &CubeCoords, dir: usize) -> CubeCoords {
    cube_add(coords, &cube_direction(dir))
}

pub fn cube_ring(center: &CubeCoords, radius: u32) -> Vec<CubeCoords> {
    let mut results = Vec::new();

    let mut coords = cube_add(center, &cube_scale(&cube_direction(4), radius as i32));

    for i in 0..6 {
        for _j in 0..radius {
            results.push(coords.clone());
            coords = cube_neighbor(&coords, i)
        }
    }

    results
}

pub fn is_within_grid(coords: AxialCoords, radius: u32) -> bool {
    let radius_i32 = radius as i32;
    let q = coords.q;
    let r = coords.r;

    (q.abs() + r.abs() + (-q - r).abs()) / 2 <= radius_i32
}

pub struct ParallelogramConfig {
    start: CubeCoords,
    height: u32,
    width: u32,
    constraint_to: u32,
}

pub fn cube_parallelogram_tiles(config: ParallelogramConfig) -> Vec<AxialCoords> {
    let mut tiles = Vec::new();
    let start = config.start;

    for r in 0..config.height as i32 {
        for q in 0..config.width as i32 {
            let h_offset = cube_scale(&CubeCoords::new(1, 0, -1), q);
            let v_offset = cube_scale(&CubeCoords::new(-1, 1, 0), r);

            let coords = cube_add(&start, &cube_add(&v_offset, &h_offset));
            let axial_coords = coords.as_axial();

            if is_within_grid(axial_coords, config.constraint_to) {
                tiles.push(axial_coords);
            }
        }
    }

    tiles
}

/// divide the hexagonal grid in batches of coords defining some parallelograms
/// should create `n = rows * cols` batches.
pub fn create_parallelogram_coords_batches(
    rows: u8,
    cols: u8,
    grid_radius: u32,
) -> Vec<Vec<AxialCoords>> {
    let mut results = Vec::new();

    let radius = grid_radius as i32;

    let d = 2 * grid_radius + 1;
    let p_width = d.div_ceil(cols as u32);
    let p_height = d.div_ceil(rows as u32);

    let start = CubeCoords::new(0, -radius, radius);

    for row in 0..rows {
        for col in 0..cols {
            // Décalage horizontal (inchangé)
            let h_offset = cube_scale(
                &cube_scale(&CubeCoords::new(1, 0, -1), p_width as i32),
                col as i32,
            );

            let v_offset = cube_scale(
                &cube_scale(&CubeCoords::new(-1, 1, 0), p_height as i32),
                row as i32,
            );

            // Calcul correct du point de départ du parallélogramme
            let parallelogram_start = cube_add(&start, &cube_add(&h_offset, &v_offset));

            let tiles = cube_parallelogram_tiles(ParallelogramConfig {
                start: parallelogram_start,
                height: p_height,
                width: p_width,
                constraint_to: grid_radius,
            });

            results.push(tiles);
        }
    }

    results
}

pub fn direct_neighbors(center: &CubeCoords) -> [CubeCoords; 6] {
    let mut results = [CubeCoords::center(); 6];
    let mut coords = cube_add(center, &cube_scale(&cube_direction(4), 1));

    for i in 0..6 {
        results[i] = coords;
        coords = cube_neighbor(&coords, i)
    }

    results
}

/**
 * Does not include center countrary to red blob games' implementation
 */
pub fn cube_spiral(center: &CubeCoords, radius: u32) -> Vec<CubeCoords> {
    let mut results: Vec<CubeCoords> = vec![center.clone()];

    let max = radius + 1;

    for k in 1..max {
        results.append(&mut cube_ring(center, k));
    }

    results
}

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub struct AxialCoords {
    pub q: i32,
    pub r: i32,
}

impl Debug for AxialCoords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(q:{},r:{})", self.q, self.r)
    }
}

impl AxialCoords {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn center() -> Self {
        Self::new(0, 0)
    }

    #[allow(dead_code)]
    pub fn as_cube(&self) -> CubeCoords {
        CubeCoords::new(self.q, self.r, -self.q - self.r)
    }
}

// implement hash for storage in HashMap
impl Hash for AxialCoords {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Combine `q` and `r` into the state directly with a mix formula
        let combined_hash = self.q as u64
            ^ ((self.r as u64)
                .wrapping_add(0x9e3779b9)
                .wrapping_add((self.q as u64) << 6)
                .wrapping_add((self.q as u64) >> 2));

        // Write the combined hash
        state.write_u64(combined_hash);
    }
}

pub type PrecomputedNeighbors = HashMap<AxialCoords, [Option<AxialCoords>; 6]>;

pub fn compute_neighboors(radius: u32) -> PrecomputedNeighbors {
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
