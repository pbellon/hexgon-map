use std::sync::Arc;

use pixelstratwar::{
    config::GameConfig,
    coords::AxialCoords,
    game::GameData,
    store::{self, RedisHandler},
    test_utils::{self, mocks::MockRedisHandler, utils::are_coords_in_vec},
};

async fn redis_client_or_mock() -> Arc<dyn RedisHandler> {
    let _ = env_logger::try_init();

    let app_config = GameConfig::read_config_from_env();

    if app_config.with_redis_tests {
        let redis = store::init_redis_client(&app_config).await.unwrap();
        let _ = store::init_redis_indices(&redis).await.unwrap();
        return Arc::new(redis);
    }

    return Arc::new(MockRedisHandler::new());
}

#[tokio::test]
pub async fn contiguous_neighbors_of_tile_empty() {
    let mock_redis = redis_client_or_mock().await;

    let game_data = GameData::new(10, 2);
    let coords = AxialCoords::new(0, 0);
    let prefetch = game_data
        .fetch_within(mock_redis.as_ref(), &coords)
        .await
        .unwrap();
    let (tiles, nb) = game_data.contiguous_neighbors_of_tile(&prefetch, &coords, "toto", 2);

    assert!(
        tiles.len() == 0 && nb == 0,
        "When user didn't click, it should not have any neighbors tile"
    );
}

#[tokio::test]
pub async fn test_fetch_within() {
    let game_data = GameData::new(10, 2);
    let mock_redis = redis_client_or_mock().await;

    let center = AxialCoords::center();

    let updated_tiles = game_data
        .handle_click(mock_redis.as_ref(), &center, "first_user_id")
        .await
        .expect("Should be able to click on (0,0)");

    let nb_updated = updated_tiles.len();
    // check we have one updated tile
    assert!(
        nb_updated == 1 && are_coords_in_vec(&updated_tiles, &center).is_some(),
        "Updated tiles should have one element (got {nb_updated}) and contain (0,0)",
    );

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, 1),
            "first_user_id",
        )
        .await
        .expect("Should be able to click on (0,1)");

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, 2),
            "first_user_id",
        )
        .await
        .expect("Should be able to click on (0, 2)");
    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, 3),
            "first_user_id",
        )
        .await
        .expect("Should be able to click on (0, 3)");

    let prefetched = game_data
        .fetch_within(mock_redis.as_ref(), &center)
        .await
        .expect("Should be able to fetch within 2 for (0,0)");

    let center_t = prefetched
        .get(&center)
        .expect("Fetched tiles hashmap should contain (0,0)");
    assert!(
        center_t.user_id == "first_user_id".to_string() && center_t.damage == 0,
        "Prefetched data should contain center with expected data"
    );

    let zero_one_t = prefetched.get(&AxialCoords::new(0, 1)).expect(&format!(
        "Prefeteched should contain (0,1),\n\tcurrent state: {prefetched:?}"
    ));

    assert!(
        zero_one_t.user_id == "first_user_id".to_string() && zero_one_t.damage == 0,
        "(0, 1) should be owned by first user and have 0 damage, got {zero_one_t:?}"
    );

    let zero_two_t = prefetched
        .get(&AxialCoords::new(0, 2))
        .expect("Prefetched should contain (0,2)");
    assert!(
        zero_two_t.user_id == "first_user_id".to_string() && zero_two_t.damage == 0,
        "(0,2) should be owned by first user and have 0 damage, got {zero_two_t:?}"
    );

    assert!(
        prefetched.get(&AxialCoords::new(0, 3)).is_none(),
        "Prefeteched hashmap should not contain (0,3) because not in 2 radius"
    );

    let _ = mock_redis.flushdb().await.unwrap();
}

/// Scenario, format: [<[A-Z]:user> - (<[0-9]+:q>,<[0-9]+:r>):coords], each list item
/// is a click on the grid
/// 1. [A, (0, 0)]
/// 2. [A, (0,-1)]
/// 3. [A, (0,-2)] // at this point (0,0) should have 3 contiguous neighbors
/// 5. [A, (0,-3)] // (0,0) should still have 3 contiguous neighbors
/// 6. [B, (0, 1)]
/// 7. [B, (0, 2)]
/// 8. [A, (0, 1)] => should damage (0,1)
/// 9. [A, (0, 1)] => should take ownership of (0,1)
/// 10. [A, (-2, 0)] => within radius of (0,0) but now contiguous
/// 11. [B, (-2, 0)] => B should take ownership directly since the tile should have a strength of 1
#[tokio::test]
pub async fn contiguous_neighbors_of_tile_with_clicks() {
    let game_data = GameData::new(10, 2);
    let mock_redis = redis_client_or_mock().await;

    let center = AxialCoords::center();

    let updated_tiles = game_data
        .handle_click(mock_redis.as_ref(), &center, "first_user_id")
        .await
        .expect("Should be able to click on (0,0)");

    let nb_updated = updated_tiles.len();

    // check we have one updated tile
    assert!(
        nb_updated == 1 && test_utils::utils::are_coords_in_vec(&updated_tiles, &center).is_some(),
        "Updated tiles should have one element (got {nb_updated}) and contain (0,0)",
    );

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, -1),
            "first_user_id",
        )
        .await
        .expect("Should be able to click on (0,-1)");

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, -2),
            "first_user_id",
        )
        .await
        .expect("Should be able to click on (0, -2)");

    let prefetched = game_data
        .fetch_within(mock_redis.as_ref(), &center)
        .await
        .expect("Should be able to fetch within 2 for (0,0)");

    let (contiguous_tiles, nb) =
        game_data.contiguous_neighbors_of_tile(&prefetched, &center, "first_user_id", 2);

    assert!(
        nb == 2,
        "Contiguous tile should contain 2 elements (got {nb})"
    );

    assert!(
        are_coords_in_vec(&contiguous_tiles, &center).is_none(),
        "Contiguous tiles should not contain (0,0) tile because it's center of the contiguity check",
    );

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, -3),
            "first_user_id",
        )
        .await
        .expect("Should be able to click on (0, -1)");

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, 1),
            "second_user_id",
        )
        .await
        .expect("Should be able to click on (0,1)");

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(1, 0),
            "first_user_id",
        )
        .await
        .expect("Should be able to click on (1,0)");

    let coords = AxialCoords::center();

    let prefetch = game_data
        .fetch_within(mock_redis.as_ref(), &coords)
        .await
        .expect("Should be able to fetch all tiles");

    let (tiles, nb) =
        game_data.contiguous_neighbors_of_tile(&prefetch, &coords, "first_user_id", 2);

    // check proper tiles in vec

    assert!(nb == 3, "In that scenario, we should have 3 contiguous tiles (got {nb}) in a radius of 2 from (0,0) owned by first user");

    assert!(
        are_coords_in_vec(&tiles, &AxialCoords::new(0, -1)).is_some(),
        "Contiguous neighbors vector should contain (0,-1)"
    );

    assert!(
        are_coords_in_vec(&tiles, &AxialCoords::new(0, -2)).is_some(),
        "Contiguous neighbors vector should contain (0,-2)"
    );

    assert!(
        are_coords_in_vec(&tiles, &AxialCoords::new(0, -3)).is_none(),
        "Contiguous neighbors vector should NOT contain (0,-3) because it's outside asked radius"
    );

    assert!(
        are_coords_in_vec(&tiles, &AxialCoords::new(0, 1)).is_none(),
        "Contiguous neighbors vector should NOT contain (0,1) because it's owned by an other user"
    );

    let _ = mock_redis.flushdb().await.unwrap();
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
    let game_data = GameData::new(10, 2);
    let mock_redis = redis_client_or_mock().await;

    game_data
        .handle_click(mock_redis.as_ref(), &AxialCoords::center(), "first_user_id")
        .await
        .unwrap();

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(1, 0),
            "first_user_id",
        )
        .await
        .unwrap();
    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, -1),
            "first_user_id",
        )
        .await
        .unwrap();
    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, -2),
            "first_user_id",
        )
        .await
        .unwrap();
    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, -3),
            "first_user_id",
        )
        .await
        .unwrap();

    // tile with a single neighbor owned by another user, should require 2 clicks to take owner ship
    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, 1),
            "second_user_id",
        )
        .await
        .unwrap();

    game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, 2),
            "second_user_id",
        )
        .await
        .unwrap();

    // check state is OK after all clicks

    // check (0,0)
    let mut tile_to_check = mock_redis
        .get_tile(&AxialCoords::center())
        .await
        .unwrap()
        .expect("Should have tile a (0,0)");

    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,0) should be owned by first user and have no damage"
    );

    // check computed tile of (0,0)
    let mut computed_tile_to_check = game_data
        .computed_tile(mock_redis.as_ref(), &AxialCoords::center(), &tile_to_check)
        .await
        .expect("Failed to compute tile to check");

    assert!(
        computed_tile_to_check.strength == 4
            && computed_tile_to_check.user_id == "first_user_id".to_string(),
        "(0,0) has no damage + 3 contiguous neighbors in a radius of 2 => strengh should eq 4"
    );

    // check (1,0)
    tile_to_check = mock_redis
        .get_tile(&AxialCoords::new(1, 0))
        .await
        .unwrap()
        .expect("Should find tile at (1,0)");

    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(1,0) should be owned by first user and have no damage"
    );

    // check (0,-1)
    tile_to_check = mock_redis
        .get_tile(&AxialCoords::new(0, -1))
        .await
        .unwrap()
        .expect("Should find tile at (0, -1)");

    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,-1) should be owned by first user and have no damage"
    );
    // check (0,-2)
    tile_to_check = mock_redis
        .get_tile(&AxialCoords::new(0, -2))
        .await
        .unwrap()
        .expect("Should find tile at (0,-2)");

    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,-2) should be owned by first user and have no damage"
    );
    // check (0,-3)
    tile_to_check = mock_redis
        .get_tile(&AxialCoords::new(0, -3))
        .await
        .unwrap()
        .expect("Should find tile at (0,-3)");
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("first_user_id"),
        "(0,-3) should be owned by first user and have no damage"
    );
    // check (0,1)
    tile_to_check = mock_redis
        .get_tile(&AxialCoords::new(0, 1))
        .await
        .unwrap()
        .expect("Should find tile at (0,1)");
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("second_user_id"),
        "(0,1) should be owned by second user and have no damage"
    );
    // check (0,2)
    tile_to_check = mock_redis
        .get_tile(&AxialCoords::new(0, 2))
        .await
        .unwrap()
        .expect("should find tile at (0,2)");
    assert!(
        tile_to_check.damage == 0 && tile_to_check.user_id == String::from("second_user_id"),
        "(0,2) should be owned by second user and have no damage"
    );

    // check computed tile of (0,2)
    computed_tile_to_check = game_data
        .computed_tile(mock_redis.as_ref(), &AxialCoords::new(0, 2), &tile_to_check)
        .await
        .unwrap();

    assert!(
        computed_tile_to_check.strength == 2
            && computed_tile_to_check.user_id == "second_user_id".to_string(),
        "(0,2) has no damage + 1 contiguous neighbors in a radius of 2 => strengh should eq 2"
    );

    // click on another user tile having a neighbor but only once => no ownership taken
    let updated_tiles = game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, 1),
            "first_user_id",
        )
        .await
        .expect("Should update tile properly");

    tile_to_check = mock_redis
        .get_tile(&AxialCoords::new(0, 1))
        .await
        .unwrap()
        .expect("Should find tile at (0,1)");
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
        .computed_tile(mock_redis.as_ref(), &AxialCoords::new(0, 1), &tile_to_check)
        .await
        .unwrap();
    assert!(
        computed_tile_to_check.strength == 1
            && computed_tile_to_check.user_id == "second_user_id".to_string(),
        "(0,1) has 1 damage + 1 contiguous neighbors in a radius of 2 => strengh should eq 1"
    );

    // click again and check we took ownership
    let updated_tiles = game_data
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(0, 1),
            "first_user_id",
        )
        .await
        .expect("Should update tile properly");

    tile_to_check = mock_redis
        .get_tile(&AxialCoords::new(0, 1))
        .await
        .unwrap()
        .expect("Should find tile at (0,1)");
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
        .handle_click(
            mock_redis.as_ref(),
            &AxialCoords::new(-2, 0),
            "first_user_id",
        )
        .await
        .expect("Should be able to update tiles");

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
            mock_redis.as_ref(),
            &AxialCoords::center(),
            &mock_redis
                .get_tile(&AxialCoords::center())
                .await
                .unwrap()
                .expect("Should find tile at (0,0)"),
        )
        .await
        .unwrap();

    assert!(
        computed_tile_to_check.strength == 5 && computed_tile_to_check.user_id == "first_user_id",
        "(0,0) should be unaffected by previous click on (-2,0) and have a strength of 5"
    );

    let _ = mock_redis.flushdb().await.unwrap();
}
