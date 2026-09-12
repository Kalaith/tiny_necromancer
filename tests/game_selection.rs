use macroquad_toolkit::grid::TilePos;
use tiny_necromancer::data::GameData;
use tiny_necromancer::engine::movement::MotionState;
use tiny_necromancer::game::selection_for_tile;
use tiny_necromancer::state::{Building, BuildingKind, GameSession, Selection};

fn selection_fixture() -> (GameSession, MotionState) {
    let data = GameData::load().expect("game data should load");
    let mut session = GameSession::new(&data.config);
    session.begin();
    session.world.buildings.push(Building {
        kind: BuildingKind::WorkShed,
        progress: 1.0,
        complete: true,
        position: TilePos::new(7, 2),
        width: 2,
        height: 2,
    });
    session.workforce.workers[0].position = TilePos::new(1, 1);
    let motions = MotionState::new(&session);
    (session, motions)
}

#[test]
fn tile_selection_keeps_worker_targets_selectable_after_necromancer_selection() {
    let (session, motions) = selection_fixture();
    let tile = session.workforce.workers[0].position;

    assert_eq!(
        selection_for_tile(&session, &motions, tile),
        Selection::Worker(0)
    );
}

#[test]
fn tile_selection_keeps_necromancer_target_selectable() {
    let (session, motions) = selection_fixture();

    assert_eq!(
        selection_for_tile(&session, &motions, session.world.necromancer_position),
        Selection::Necromancer
    );
}

#[test]
fn tile_selection_keeps_building_targets_selectable() {
    let (session, motions) = selection_fixture();

    assert_eq!(
        selection_for_tile(&session, &motions, TilePos::new(7, 2)),
        Selection::Building(0)
    );
}

#[test]
fn tile_selection_keeps_grave_targets_selectable() {
    let (session, motions) = selection_fixture();
    let tile = session.world.plots[0].position;

    assert_eq!(
        selection_for_tile(&session, &motions, tile),
        Selection::Grave(0)
    );
}

#[test]
fn open_ground_remains_a_necromancer_move_target() {
    let (session, motions) = selection_fixture();
    let tile = TilePos::new(1, 7);

    assert_eq!(
        selection_for_tile(&session, &motions, tile),
        Selection::Ground(tile)
    );
}
