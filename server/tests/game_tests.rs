use pixelstratwar::{coords::AxialCoords, game::GameData, grid::InnerTileData};

#[tokio::test]
pub async fn contiguous_neighbors_of_tile_empty() {
    let game_data = GameData::new(10);
    let (tiles, nb) = game_data
        .contiguous_neighbors_of_tile(&AxialCoords::new(0, 0), "toto", 2)
        .await;

    assert!(
        tiles.len() == 0 && nb == 0,
        "When user didn't click, it should not have any neighbors tile"
    );
}

fn check_as_coords_in_vec(
    tiles: &Vec<(AxialCoords, InnerTileData)>,
    coords_to_check: &AxialCoords,
) -> bool {
    match tiles.iter().find(|(coords, _)| coords == coords_to_check) {
        Some(_) => true,
        None => false,
    }
}

#[tokio::test]
pub async fn contiguous_neighbors_of_tile_with_clicks() {
    let game_data = GameData::new(10);

    game_data
        .handle_click(&AxialCoords::center(), "first_user_id")
        .await;
    game_data
        .handle_click(&AxialCoords::new(0, -1), "first_user_id")
        .await;
    game_data
        .handle_click(&AxialCoords::new(0, -2), "first_user_id")
        .await;
    game_data
        .handle_click(&AxialCoords::new(0, -3), "first_user_id")
        .await;
    game_data
        .handle_click(&AxialCoords::new(0, 1), "second_user_id")
        .await;
    game_data
        .handle_click(&AxialCoords::new(1, 0), "first_user_id")
        .await;

    let (tiles, nb) = game_data
        .contiguous_neighbors_of_tile(&AxialCoords::center(), "first_user_id", 2)
        .await;

    // check proper tiles in vec

    assert!(nb == 3, "In that scenario, we should have 3 contiguous tiles (got {nb}) in a radius of 2 from (0,0) owned by first user");

    assert!(
        check_as_coords_in_vec(&tiles, &AxialCoords::new(0, -1)),
        "Contiguous neighbors vector should contain (0,-1)"
    );

    assert!(
        check_as_coords_in_vec(&tiles, &AxialCoords::new(0, -2)),
        "Contiguous neighbors vector should contain (0,-2)"
    );

    assert!(
        !check_as_coords_in_vec(&tiles, &AxialCoords::new(0, -3)),
        "Contiguous neighbors vector should NOT contain (0,-3) because it's outside asked radius"
    );

    assert!(
        !check_as_coords_in_vec(&tiles, &AxialCoords::new(0, 1)),
        "Contiguous neighbors vector should NOT contain (0,1) because it's owned by an other user"
    )
}
