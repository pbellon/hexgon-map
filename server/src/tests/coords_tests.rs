use crate::coords::{cube_ring, cube_spiral, CubeCoords};

#[test]
fn test_cube_ring() {
    let center = CubeCoords::new(0, 0, 0);

    // Call the coords_range function to get a list of cube coordinates.
    let result_one: Vec<CubeCoords> = cube_ring(&center, 1);
    let result_two: Vec<CubeCoords> = cube_ring(&center, 2);
    let result_three: Vec<CubeCoords> = cube_ring(&center, 3);

    assert!(
        result_one.len() == 6,
        "coords_range should return a vector with exactly 6 elements for a radius of 1"
    );

    assert!(
        result_two.len() == 12,
        "coords_range should return a vector with exactly 12 elements for a radius of 2"
    );

    assert!(
        result_three.len() == 18,
        "coords_range should return a vector with exactly 18 elements for a radius of 3"
    )
}

#[test]
fn test_coords_spiral() {
    let center = CubeCoords::new(0, 0, 0);
    let result_one: Vec<CubeCoords> = cube_spiral(&center, 1, false);
    let result_two = cube_spiral(&center, 2, false);
    let result_three = cube_spiral(&center, 3, false);
    assert!(
        result_one.len() == 6,
        "coords_range should return a vector with exactly 6 elements for a radius of 1"
    );

    assert!(
        result_two.len() == 18,
        "coords_range should return a vector with exactly 18 elements for a radius of 2"
    );

    assert!(
        result_two.len() == 36,
        "coords_range should return a vector with exactly 18 elements for a radius of 2"
    );
}

// #[test]
// fn test_cube_subtract() {
//     let a = CubeCoords::new(0, 1, -1);
//     let b = CubeCoords::new(1, 2, -3);
//     let res = cube_substract(&a, &b);
//     assert!(
//         res == CubeCoords::new(-1, -1, 2),
//         "{:?} - {:?} should equal {{q: -1, r: -1, s: 2}} got {:?}",
//         a,
//         b,
//         res
//     );
// }
