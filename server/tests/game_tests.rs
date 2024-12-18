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
/// core game behavior testing, all actions, format: [<[A-Z]:user> - (<[0-9]+:q>,<[0-9]+:r>):coords]
/// 1. [A, (0, 0)]
/// 2. [A, (1, 0)]
/// 3. [A, (0,-1)]
/// 4. [A, (0,-2)]
/// 5. [A, (0,-3)]
/// 6. [B, (0, 1)]
/// 7. [B, (0, 2)]
/// 8. [A, (0, 1)] => should damage (0,1)
/// 9. [A, (0, 1)] => should take ownership of (0,1)
/// 10. [A, (-2, 0)] => within radius of (0,0) but now contiguous
/// 11. [B, (-2, 0)] => B should take ownership directly since the tile should have a strength of 1
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

    // check (0,0)
    let mut tile_to_check = game_data.get_tile(&AxialCoords::center()).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,0) should be owned by first user and have no damage"
    );

    // check computed tile of (0,0)
    let mut computed_tile_to_check = game_data
        .computed_tile(&AxialCoords::center(), &tile_to_check)
        .await;
    assert!(
        computed_tile_to_check.strength == 4
            && computed_tile_to_check.user_id == "first_user_id".to_string(),
        "(0,0) has no damage + 3 contiguous neighbors in a radius of 2 => strengh should eq 4"
    );

    // check (1,0)
    tile_to_check = game_data.get_tile(&AxialCoords::new(1, 0)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(1,0) should be owned by first user and have no damage"
    );

    // check (0,-1)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, -1)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,-1) should be owned by first user and have no damage"
    );
    // check (0,-2)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, -2)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,-2) should be owned by first user and have no damage"
    );
    // check (0,-3)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, -3)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,-3) should be owned by first user and have no damage"
    );
    // check (0,1)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 1)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("second_user_id"),
        "(0,1) should be owned by second user and have no damage"
    );
    // check (0,2)
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 2)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("second_user_id"),
        "(0,2) should be owned by second user and have no damage"
    );

    // check computed tile of (0,2)
    computed_tile_to_check = game_data
        .computed_tile(&AxialCoords::new(0, 2), &tile_to_check)
        .await;
    assert!(
        computed_tile_to_check.strength == 2
            && computed_tile_to_check.user_id == "second_user_id".to_string(),
        "(0,2) has no damage + 1 contiguous neighbors in a radius of 2 => strengh should eq 2"
    );

    // click on another user tile having a neighbor but only once => no ownership taken
    let updated_tiles = game_data
        .handle_click(&AxialCoords::new(0, 1), "first_user_id")
        .await;
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 1)).await.unwrap();
    assert!(
        tile_to_check.damage == 1 && tile_to_check.user_id == String::from("second_user_id"),
        "(0,1) should still be owned by second user but with 1 damage"
    );

    let (first_updated_tile_coords, first_updated_tile_data) = updated_tiles.get(0).unwrap();
    assert!(
        updated_tiles.len() == 1
            && first_updated_tile_coords == &AxialCoords::new(0, 1)
            && first_updated_tile_data.user_id == "second_user_id"
            && first_updated_tile_data.strength == 1,
        "updated tiles vec should contain only clicked tile with a strenght of one"
    );

    // check computed tile of (0,1) to see if damage affects overall strength
    computed_tile_to_check = game_data
        .computed_tile(&AxialCoords::new(0, 1), &tile_to_check)
        .await;
    assert!(
        computed_tile_to_check.strength == 1
            && computed_tile_to_check.user_id == "second_user_id".to_string(),
        "(0,1) has 1 damage + 1 contiguous neighbors in a radius of 2 => strengh should eq 1"
    );

    // click again and check we took ownership
    let updated_tiles = game_data
        .handle_click(&AxialCoords::new(0, 1), "first_user_id")
        .await;
    tile_to_check = game_data.get_tile(&AxialCoords::new(0, 1)).await.unwrap();
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,1) should now be owned by first user and have no damage anymore"
    );

    let len = updated_tiles.len();
    assert!(len == 5, "should have 5 updated tiles, has {len}");

    // check (0,1) in updated tiles and have proper owner
    match updated_tiles
        .iter()
        .cloned()
        .find(|(coords, _)| coords == &AxialCoords::new(0, 1))
    {
        Some((_, tile)) => {
            assert!(
                tile.user_id == "first_user_id".to_string() && tile.strength == 4,
                "(0,1) should have a strength of 4"
            );
        }
        None => {
            assert!(false, "(0,1) should be in updated tiles vector");
        }
    };
    // check (0,2) in updated tiles, still owned by second user with strength of 1
    match updated_tiles
        .iter()
        .cloned()
        .find(|(coords, _)| coords == &AxialCoords::new(0, 2))
    {
        Some((_, tile)) => {
            assert!(
                tile.user_id == "second_user_id".to_string() && tile.strength == 1,
                "(0,2) should still be owned by second user and have a strength of 1"
            );
        }
        None => {
            assert!(false, "(0,2) should be in updated tiles vector");
        }
    };

    // test contiguous behavior properly works by clicking on a tile within center point's radius
    // but not contiguous to first user tiles
    let updated_tiles = game_data
        .handle_click(&AxialCoords::new(-2, 0), "first_user_id")
        .await;

    assert!(
        updated_tiles.len() == 1,
        "Updated tiles should contain only one element"
    );

    match updated_tiles
        .iter()
        .cloned()
        .find(|(coords, _)| coords == &AxialCoords::new(-2, 0))
    {
        Some((_, tile)) => {
            assert!(
                tile.strength == 1 && tile.user_id == "first_user_id",
                "(-2,0) should be owned by first user and have a strength of 1"
            );
        }

        None => {
            assert!(false, "(-2,0) should be in updated tiles");
        }
    }

    // check (0,0) unaffacted by previous click
    let computed_tile_to_check = game_data
        .computed_tile(
            &AxialCoords::center(),
            &game_data.get_tile(&AxialCoords::center()).await.unwrap(),
        )
        .await;

    assert!(
        computed_tile_to_check.strength == 5 && computed_tile_to_check.user_id == "first_user_id",
        "(0,0) should be unaffected by previous click on (-2,0) and have a strength of 5"
    );
}
