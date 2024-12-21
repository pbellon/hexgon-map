use crate::coords::AxialCoords;

pub fn are_coords_in_vec<T>(
    tiles: &Vec<(AxialCoords, T)>,
    coords_to_check: &AxialCoords,
) -> Option<(AxialCoords, T)>
where
    T: Clone,
{
    tiles
        .iter()
        .find(|(coords, _)| coords == coords_to_check)
        .cloned()
}
