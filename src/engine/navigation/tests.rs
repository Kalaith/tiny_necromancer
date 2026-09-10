use super::*;

#[test]
fn route_avoids_a_building_footprint() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.world.buildings.push(crate::state::Building {
        kind: crate::state::BuildingKind::WorkShed,
        progress: 10.0,
        complete: true,
        position: TilePos::new(3, 2),
        width: 2,
        height: 2,
    });
    let first = plan_route(&session, TilePos::new(2, 2), TilePos::new(5, 2))
        .unwrap()
        .next_step()
        .unwrap();
    assert_ne!(first, TilePos::new(3, 2));
}

#[test]
fn route_can_enter_a_blocked_target_tile() {
    let data = crate::data::GameData::load().unwrap();
    let session = GameSession::new(&data.config);
    let target = session.world.forest_tiles[0];
    let route = plan_route(&session, TilePos::new(1, 0), target).unwrap();
    assert_eq!(route.next_step(), Some(target));
    assert_eq!(route.steps().last().copied(), Some(target));
}

#[test]
fn route_plan_reports_every_step_in_stable_order() {
    let data = crate::data::GameData::load().unwrap();
    let session = GameSession::new(&data.config);
    let route = plan_route(&session, TilePos::new(2, 2), TilePos::new(5, 2)).unwrap();

    assert_eq!(route.steps().first().copied(), Some(TilePos::new(2, 2)));
    assert_eq!(route.steps().last().copied(), Some(TilePos::new(5, 2)));
    assert_eq!(route.step_count(), 3);
    assert!(route
        .steps()
        .windows(2)
        .all(|pair| (pair[0].x - pair[1].x).abs() + (pair[0].y - pair[1].y).abs() == 1));
}

#[test]
fn route_plan_explains_a_sealed_cemetery() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    for y in 0..session.world.height as i32 {
        session.world.buildings.push(crate::state::Building {
            kind: crate::state::BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position: TilePos::new(3, y),
            width: 1,
            height: 1,
        });
    }

    let failure = plan_route(&session, TilePos::new(2, 2), TilePos::new(5, 2)).unwrap_err();
    assert_eq!(failure, RouteFailure::NoPath);
    assert_eq!(failure.label(), "obstructions seal the way");
}

#[test]
fn nearest_reachable_skips_a_sealed_candidate() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    for y in 0..session.world.height as i32 {
        session.world.buildings.push(crate::state::Building {
            kind: crate::state::BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position: TilePos::new(3, y),
            width: 1,
            height: 1,
        });
    }

    let target = nearest_reachable(
        &session,
        TilePos::new(2, 2),
        [TilePos::new(5, 2), TilePos::new(2, 4)],
    );
    assert_eq!(target, Some(TilePos::new(2, 4)));
}
