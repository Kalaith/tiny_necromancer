use super::*;

#[test]
fn authored_data_loads_and_has_required_content() {
    let data = GameData::load().unwrap();
    assert_eq!(data.config.world_width, 10);
    assert!(data.jobs.contains("dig"));
    assert!(data.undead.contains("brute_skeleton"));
    assert!(data.buildings.contains("grave_lantern"));
    assert!(data
        .event_for_stage(SuspicionStage::Investigation)
        .is_some());
}

#[test]
fn invalid_thresholds_are_rejected() {
    let data = GameData::load().unwrap();
    let mut config = data.config.clone();
    config.suspicion_thresholds = [50.0, 25.0, 70.0];
    let invalid = GameData { config, ..data };
    assert!(invalid.validate().is_err());
}
