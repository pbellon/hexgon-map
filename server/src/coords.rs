use std::hash::{DefaultHasher, Hash, Hasher};

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

// pub fn cube_substract(a: &CubeCoords, b: &CubeCoords) -> CubeCoords {
//     CubeCoords::new(a.q - b.q, a.r - b.r, a.s - b.s)
// }

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

/**
 * Does not include center countrary to red blob games' implementation
 */
pub fn cube_spiral(center: &CubeCoords, radius: i32, with_center: bool) -> Vec<CubeCoords> {
    let mut results: Vec<CubeCoords> = Vec::new();

    if with_center {
        results.push(center.clone());
    }

    let max = radius + 1;

    for k in 1..max {
        results.append(&mut cube_ring(center, k));
    }

    results
}

pub fn cube_spiral_without_center(center: &CubeCoords, radius: i32) -> Vec<CubeCoords> {
    cube_spiral(center, radius, false)
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

    pub fn as_cube(&self) -> CubeCoords {
        CubeCoords::new(self.q, self.r, -self.q - self.r)
    }
}

// implement hash for storage in HashMap
impl Hash for AxialCoords {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut int_hasher = DefaultHasher::new();

        // Compute individual hashes for q and r
        self.q.hash(&mut int_hasher);
        let hq = int_hasher.finish();
        int_hasher = DefaultHasher::new(); // Reset the hasher for `r`
        self.r.hash(&mut int_hasher);
        let hr = int_hasher.finish();

        // Combine the hashes using the same logic
        let combined_hash = hq
            ^ (hr
                .wrapping_add(0x9e3779b9)
                .wrapping_add(hq << 6)
                .wrapping_add(hq >> 2));

        // Feed the combined hash to the state
        state.write_u64(combined_hash);
    }
}
