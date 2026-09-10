use super::*;

#[test]
fn authored_data_loads_and_has_required_content() {
    let data = GameData::load().unwrap();
    assert_eq!(data.config.world_width, 10);
    assert!(data.jobs.contains("dig"));
    assert!(data.undead.contains("brute_skeleton"));
    assert!(data.buildings.contains("grave_lantern"));
    assert!(data.jobs.contains("refine"));
    assert!(data
        .buildings
        .get("ossuary_kiln")
        .is_some_and(|building| building.production.is_some()));
    assert!(data
        .event_for_stage(SuspicionStage::Investigation)
        .is_some());
    assert_eq!(data.config.district_rules.work_speed_multiplier, 1.15);
    assert_eq!(data.config.district_rules.storage_capacity_bonus, 4);
    assert_eq!(data.config.district_rules.patrol_mitigation_multiplier, 1.25);
}

#[test]
fn invalid_thresholds_are_rejected() {
    let data = GameData::load().unwrap();
    let mut config = data.config.clone();
    config.suspicion_thresholds = [50.0, 25.0, 70.0];
    let invalid = GameData { config, ..data };
    assert!(invalid.validate().is_err());
}

#[test]
fn invalid_district_rules_are_rejected() {
    let data = GameData::load().unwrap();
    let mut config = data.config.clone();
    config.district_rules.storage_capacity_bonus = -1;
    let invalid = GameData { config, ..data };
    assert!(invalid.validate().is_err());
}
