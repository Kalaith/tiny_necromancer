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
    let first = next_step(&session, TilePos::new(2, 2), TilePos::new(5, 2)).unwrap();
    assert_ne!(first, TilePos::new(3, 2));
}

#[test]
fn route_can_enter_a_blocked_target_tile() {
    let data = crate::data::GameData::load().unwrap();
    let session = GameSession::new(&data.config);
    let target = session.world.forest_tiles[0];
    assert_eq!(
        next_step(&session, TilePos::new(1, 0), target),
        Some(target)
    );
}
