use super::*;

#[test]
fn fresh_session_starts_with_one_usable_plot_and_skeleton() {
    let data = crate::data::GameData::load().unwrap();
    let session = GameSession::new(&data.config);
    assert_eq!(session.progress.unlocked_plots, 1);
    assert_eq!(session.workforce.workers.len(), 1);
    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
    assert_eq!(session.phase, GamePhase::MainMenu);
}

#[test]
fn save_round_trip_preserves_rng_and_operation() {
    let data = crate::data::GameData::load().unwrap();
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
    let restored = GameSession::from_save(save.clone());
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
fn older_saves_default_the_district_ledger() {
    let data = crate::data::GameData::load().unwrap();
    let save = GameSession::new(&data.config).to_save(&data.config.version);
    let mut value = serde_json::to_value(save).unwrap();
    value
        .get_mut("progress")
        .and_then(serde_json::Value::as_object_mut)
        .expect("progress object")
        .remove("district_ledger");

    let restored = GameSession::from_save(serde_json::from_value(value).unwrap());
    assert_eq!(restored.progress.district_ledger.work_cycles, 0);
    assert_eq!(restored.progress.district_ledger.storage_bonus_items, 0);
    assert_eq!(restored.progress.district_ledger.patrol_quieting, 0.0);
    assert!(restored.progress.district_ledger.recent_activity.is_empty());
}

#[test]
fn legacy_save_migrates_to_a_playable_session() {
    let data = crate::data::GameData::load().unwrap();
    let value = serde_json::json!({ "points": 42, "energy": 99.0, "turn": 3 });
    let migrated = migrate_save_value(Some("1.0.0".to_owned()), value, &data.config).unwrap();
    assert_eq!(migrated.version, data.config.version);
    assert_eq!(migrated.economy.bones, 42);
    assert_eq!(migrated.phase, GamePhase::Playing);
}

#[test]
fn starter_save_shape_migrates_without_losing_player_values() {
    let data = crate::data::GameData::load().unwrap();
    let value = serde_json::json!({
        "version": "1.0.0",
        "player": { "points": 17, "energy": 8.0, "turn": 4 },
        "world": {}
    });
    let migrated = migrate_save_value(Some("1.0.0".to_owned()), value, &data.config).unwrap();
    assert_eq!(migrated.economy.bones, 17);
    assert_eq!(migrated.progress.elapsed_seconds, 1.5);
}

#[test]
fn zone_helpers_expose_storage_and_patrol_anchors() {
    let data = crate::data::GameData::load().unwrap();
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
fn zone_helpers_search_all_marked_districts() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let near_storage = TilePos::new(5, 5);
    let near_patrol = TilePos::new(5, 1);
    session.world.zones = vec![
        Zone {
            kind: ZoneKind::Storage,
            tiles: vec![TilePos::new(1, 1)],
        },
        Zone {
            kind: ZoneKind::Storage,
            tiles: vec![near_storage],
        },
        Zone {
            kind: ZoneKind::Patrol,
            tiles: vec![TilePos::new(1, 6)],
        },
        Zone {
            kind: ZoneKind::Patrol,
            tiles: vec![near_patrol],
        },
    ];

    assert_eq!(
        session.world.storage_position_for(near_storage),
        near_storage
    );
    assert_eq!(session.world.patrol_position_for(near_patrol), near_patrol);
}

#[test]
fn zone_tiles_can_be_toggled_back_off() {
    let tile = TilePos::new(4, 4);
    let mut zone = Zone {
        kind: ZoneKind::Work,
        tiles: vec![tile],
    };
    assert!(zone.toggle_tile(tile));
    assert!(zone.tiles.is_empty());
    assert!(!zone.toggle_tile(tile));
    assert_eq!(zone.tiles, vec![tile]);
}

#[test]
fn older_priority_lists_receive_new_jobs_without_losing_order() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.workforce.priorities = vec![JobKind::Dig, JobKind::Haul];
    let restored = GameSession::from_save(session.to_save(&data.config.version));
    assert_eq!(restored.workforce.priorities[0], JobKind::Dig);
    assert_eq!(restored.workforce.priorities[1], JobKind::Haul);
    assert!(restored.workforce.priorities.contains(&JobKind::Refine));
    assert_eq!(restored.workforce.priorities.len(), 6);
}

#[test]
fn older_saves_without_a_policy_default_to_balanced() {
    let data = crate::data::GameData::load().unwrap();
    let save = GameSession::new(&data.config).to_save(&data.config.version);
    let mut value = serde_json::to_value(save).unwrap();
    value
        .as_object_mut()
        .expect("save object")
        .remove("stewardship_policy");

    let migrated = migrate_save_value(None, value, &data.config).unwrap();

    assert_eq!(migrated.stewardship_policy, StewardshipPolicy::Balanced);
}

#[test]
fn zone_tools_allow_work_tiles_but_reject_roads_and_structures() {
    let data = crate::data::GameData::load().unwrap();
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
