use tiny_necromancer::data::*;

#[test]
fn authored_data_loads_and_has_required_content() {
    let data = GameData::load().unwrap();
    assert_eq!(data.config.world_width, 10);
    assert!(data.jobs.contains("dig"));
    assert!(data.undead.contains("brute_skeleton"));
    assert!(data.buildings.contains("grave_lantern"));
    assert!(data.jobs.contains("refine"));
    assert!(data.buildings.get("ossuary_kiln").is_some_and(|building| {
        building
            .production
            .as_ref()
            .is_some_and(|production| production.recipes.len() == 2)
    }));
    assert!(data
        .buildings
        .iter()
        .all(|(_, building)| !building.upgrade_effect_text.is_empty()));
    assert!(data
        .event_for_stage(SuspicionStage::Investigation)
        .is_some());
    assert_eq!(data.config.district_rules.work_speed_multiplier, 1.15);
    assert_eq!(data.config.district_rules.storage_capacity_bonus, 4);
    assert_eq!(
        data.config.district_rules.patrol_mitigation_multiplier,
        1.25
    );
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

#[test]
fn layout_starting_content_and_authored_text_are_loaded_together() {
    let data = GameData::load().unwrap();

    assert_eq!(data.config.world_layout.road_x, 8);
    assert_eq!(data.config.world_layout.plot_positions.len(), 6);
    assert_eq!(
        data.config.world_layout.building_position("ossuary_kiln"),
        Some(macroquad_toolkit::grid::TilePos::new(6, 4))
    );
    assert_eq!(data.config.starting_content.worker.name, "Rattlebones");
    assert_eq!(data.config.starting_content.worker.job_id, "dig");
    assert_eq!(
        data.text.research("domain_stewardship").unwrap().label,
        "Domain Stewardship"
    );
    assert_eq!(data.text.stewardship("harvest").unwrap().label, "Harvest");
}

#[test]
fn invalid_authored_layout_reference_is_rejected() {
    let data = GameData::load().unwrap();
    let mut config = data.config.clone();
    config.world_layout.building_positions[0].position =
        macroquad_toolkit::grid::TilePos::new(99, 99);
    let invalid = GameData { config, ..data };

    assert!(invalid.validate().is_err());
}
