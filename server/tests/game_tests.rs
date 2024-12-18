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

#[tokio::test]
// core game behavior testing
pub async fn game_behavior_taking_ownership() {
    // init game data with basic ownership
    let game_data = GameData::new(10);

    game_data
        .handle_click(&AxialCoords::center(), "first_user_id")
        .await;

    game_data
        .handle_click(&AxialCoords::new(1, 0), "first_user_id")
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

    // tile with a single neighbor owned by another user, should require 2 clicks to take owner ship
    game_data
        .handle_click(&AxialCoords::new(0, 1), "second_user_id")
        .await;
    game_data
        .handle_click(&AxialCoords::new(0, 2), "second_user_id")
        .await;

    // check state is OK after all clicks
    let mut tile_to_check = game_data.get_tile(&AxialCoords::center()).await.unwrap();
    // check (0,0)
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == Some(String::from("first_user_id")),
        "(0,0) should be owned by first user and have no damage"
    );

    // check computed tile of (0,0)
    let mut computed_tile_to_check = game_data
        .computed_tile(&AxialCoords::center(), &tile_to_check)
        .await;
    assert!(
        computed_tile_to_check.strength == 4
            && computed_tile_to_check.user_id == Some("first_user_id".to_string()),
        "(0,0) has no damage + 3 contiguous neighbors in a radius of 2 => strengh should eq 4"
    );

    // check (1,0)
    tile_to_check = game_data.get_tile(&AxialCoords::new(1, 0)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == Some(String::from("first_user_id")),
        "(1,0) should be owned by first user and have no damage"
    );

    // check (0,-1)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, -1)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == Some(String::from("first_user_id")),
        "(0,-1) should be owned by first user and have no damage"
    );
    // check (0,-2)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, -2)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == Some(String::from("first_user_id")),
        "(0,-2) should be owned by first user and have no damage"
    );
    // check (0,-3)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, -3)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == Some(String::from("first_user_id")),
        "(0,-3) should be owned by first user and have no damage"
    );
    // check (0,1)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 1)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == Some(String::from("second_user_id")),
        "(0,1) should be owned by second user and have no damage"
    );
    // check (0,2)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 2)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == Some(String::from("second_user_id")),
        "(0,2) should be owned by second user and have no damage"
    );

    // check computed tile of (0,2)
    computed_tile_to_check = game_data
        .computed_tile(&AxialCoords::new(0, 2), &tile_to_check)
        .await;
    assert!(
        computed_tile_to_check.strength == 2
            && computed_tile_to_check.user_id == Some("second_user_id".to_string()),
        "(0,2) has no damage + 1 contiguous neighbors in a radius of 2 => strengh should eq 2"
    );

    // click on another user tile having a neighbor but only once => no ownership taken
    game_data
        .handle_click(&AxialCoords::new(0, 1), "first_user_id")
        .await;

    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 1)).await.unwrap();
    assert!(
        tile_to_check.damage == 1 && tile_to_check.user_id == Some(String::from("second_user_id")),
        "(0,1) should still be owned by second user but with 1 damage"
    );

    // check computed tile of (0,1) to see if damage affects overall strength
    computed_tile_to_check = game_data
        .computed_tile(&AxialCoords::new(0, 1), &tile_to_check)
        .await;
    assert!(
        computed_tile_to_check.strength == 1
            && computed_tile_to_check.user_id == Some("second_user_id".to_string()),
        "(0,1) has 1 damage + 1 contiguous neighbors in a radius of 2 => strengh should eq 1"
    );

    // click again and check we took ownership
    game_data
        .handle_click(&AxialCoords::new(0, 1), "first_user_id")
        .await;

    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 1)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == Some(String::from("first_user_id")),
        "(0,1) should now be owned by first user and have no damage anymore"
    );

    // recheck computed tile of (0,2)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 2)).await.unwrap();
    computed_tile_to_check = game_data
        .computed_tile(&AxialCoords::new(0, 2), &tile_to_check)
        .await;

    assert!(
        computed_tile_to_check.strength == 1
            && computed_tile_to_check.user_id == Some("second_user_id".to_string()),
        "(0,2) has no damage + 0 contiguous neighbors in a radius of 2 => strengh should eq 1"
    );
}
