use macroquad_toolkit::grid::TilePos;
use tiny_necromancer::state::*;

#[test]
fn fresh_session_starts_with_one_usable_plot_and_skeleton() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let session = GameSession::new(&data.config);
    assert_eq!(session.progress.unlocked_plots, 1);
    assert_eq!(session.workforce.workers.len(), 1);
    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
    assert_eq!(session.phase, GamePhase::MainMenu);
}
#[test]
fn save_round_trip_preserves_rng_and_operation() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.begin();
    session.economy.wood = 47;
    session.stewardship_policy = StewardshipPolicy::Secure;
    session.progress.district_ledger.work_cycles = 3;
    session.progress.district_ledger.storage_bonus_items = 5;
    session.progress.district_ledger.patrol_quieting = 1.25;
    session
        .progress
        .district_ledger
        .recent_activity
        .push(DistrictActivity {
            kind: DistrictActivityKind::StorageBonus,
            amount: 4.0,
            elapsed_seconds: 12.0,
        });
    let save = session.to_save(&data.config.version);
    let restored = GameSession::from_save(save.clone(), &data.config);
    assert_eq!(restored.phase, GamePhase::Playing);
    assert_eq!(restored.economy.wood, 47);
    assert_eq!(restored.stewardship_policy, StewardshipPolicy::Secure);
    assert_eq!(restored.progress.district_ledger.work_cycles, 3);
    assert_eq!(restored.progress.district_ledger.storage_bonus_items, 5);
    assert!((restored.progress.district_ledger.patrol_quieting - 1.25).abs() < 0.001);
    assert_eq!(restored.progress.district_ledger.recent_activity.len(), 1);
    assert_eq!(
        restored.progress.district_ledger.recent_activity[0].kind,
        DistrictActivityKind::StorageBonus
    );
    assert_eq!(restored.rng.state(), save.rng_state);
}
#[test]
fn legacy_save_migrates_to_a_playable_session() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let value = serde_json::json!({ "points": 42, "energy": 99.0, "turn": 3 });
    let migrated = migrate_save_value(Some("1.0.0".to_owned()), value, &data.config).unwrap();
    assert_eq!(migrated.version, data.config.version);
    assert_eq!(migrated.economy.bones, 42);
    assert_eq!(migrated.phase, GamePhase::Playing);
}
#[test]
fn zone_helpers_expose_storage_and_patrol_anchors() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let storage = TilePos::new(4, 5);
    let patrol = TilePos::new(5, 1);
    session.world.zones = vec![
        Zone {
            kind: ZoneKind::Storage,
            tiles: vec![storage],
        },
        Zone {
            kind: ZoneKind::Patrol,
            tiles: vec![patrol],
        },
    ];
    assert_eq!(session.world.storage_position(), storage);
    assert_eq!(session.world.patrol_position(), patrol);
    assert!(session.world.zone_contains(ZoneKind::Storage, storage));
}
#[test]
fn zone_tools_allow_work_tiles_but_reject_roads_and_structures() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    assert!(session
        .world
        .is_zone_tile_allowed(session.world.plots[0].position));
    assert!(session
        .world
        .is_zone_tile_allowed(session.world.forest_tiles[0]));
    assert!(!session
        .world
        .is_zone_tile_allowed(TilePos::new(session.world.road_x, 0)));
    session.world.buildings.push(Building {
        kind: BuildingKind::GraveLantern,
        progress: 1.0,
        complete: true,
        position: TilePos::new(4, 4),
        width: 1,
        height: 1,
    });
    assert!(!session.world.is_zone_tile_allowed(TilePos::new(4, 4)));
}
