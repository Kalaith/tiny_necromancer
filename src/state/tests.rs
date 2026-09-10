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
    let save = session.to_save(&data.config.version);
    let restored = GameSession::from_save(save.clone());
    assert_eq!(restored.phase, GamePhase::Playing);
    assert_eq!(restored.economy.wood, 47);
    assert_eq!(restored.rng.state(), save.rng_state);
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
