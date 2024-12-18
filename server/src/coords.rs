use std::{
    collections::HashMap,
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

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub struct CubeCoords {
    q: i32,
    r: i32,
    s: i32,
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

pub fn cube_add(a: &CubeCoords, b: &CubeCoords) -> CubeCoords {
    CubeCoords::new(a.q + b.q, a.r + b.r, a.s + b.s)
}

#[allow(dead_code)]
pub fn cube_substract(a: &CubeCoords, b: &CubeCoords) -> CubeCoords {
    CubeCoords::new(a.q - b.q, a.r - b.r, a.s - b.s)
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

pub fn cube_ring(center: &CubeCoords, radius: i32) -> Vec<CubeCoords> {
    let mut results = Vec::new();

    let mut coords = cube_add(center, &cube_scale(&cube_direction(4), radius));

    for i in 0..6 {
        for _j in 0..radius {
            results.push(coords.clone());
            coords = cube_neighbor(&coords, i)
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
pub fn cube_spiral(center: &CubeCoords, radius: i32) -> Vec<CubeCoords> {
    let mut results: Vec<CubeCoords> = vec![center.clone()];

    let max = radius + 1;

    for k in 1..max {
        results.append(&mut cube_ring(center, k));
    }

    results
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone, Copy)]
pub struct AxialCoords {
    pub q: i32,
    pub r: i32,
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

pub fn is_within_grid(coords: AxialCoords, radius: i32) -> bool {
    coords.q >= -radius && coords.q <= radius && coords.r >= -radius && coords.r <= radius
}

pub type PrecomputedNeighbors = HashMap<AxialCoords, [Option<AxialCoords>; 6]>;

pub fn compute_neighboors(radius: i32) -> PrecomputedNeighbors {
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
