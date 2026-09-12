//! Semantic checks for the authored balance, layout, and text registries.

use super::*;
use std::collections::HashSet;

pub(super) fn validate(data: &GameData) -> Result<(), String> {
    validate_config(&data.config)?;
    validate_balances(data)?;
    validate_references(data)?;
    validate_world_layout(&data.config)?;
    validate_starting_content(data)?;
    validate_text(&data.text)?;
    Ok(())
}

fn validate_config(config: &GameConfig) -> Result<(), String> {
    if config.world_width == 0 || config.world_height == 0 {
        return Err("game_config.json: world dimensions must be positive".to_owned());
    }
    if config.tick_seconds <= 0.0 || config.mana_regen_per_second < 0.0 {
        return Err("game_config.json: tick and mana rates are invalid".to_owned());
    }
    if config.starting_bones < 0
        || config.starting_mana < 0
        || config.starting_wood < 0
        || config.storage_capacity <= 0
        || config.max_mana <= 0
        || config.worker_wood_reserve < 0
        || config.starting_unlocked_plots == 0
        || config.starting_unlocked_plots > config.victory_plots
        || config.victory_plots > 6
    {
        return Err("game_config.json: starting resources or plot targets are invalid".to_owned());
    }
    if !(0.0..=1.0).contains(&config.corpse_discovery_chance) {
        return Err("game_config.json: corpse chance must be between 0 and 1".to_owned());
    }
    if config.suspicion_thresholds[0] <= 0.0
        || config.suspicion_thresholds[0] >= config.suspicion_thresholds[1]
        || config.suspicion_thresholds[1] >= config.suspicion_thresholds[2]
        || config.suspicion_thresholds[2] > 100.0
    {
        return Err(
            "game_config.json: suspicion thresholds must increase within 0..100".to_owned(),
        );
    }
    if config.plot_unlock_base_wood < 0 || config.plot_unlock_step_wood < 0 {
        return Err("game_config.json: plot unlock costs cannot be negative".to_owned());
    }
    if config.district_rules.work_speed_multiplier < 1.0
        || config.district_rules.storage_capacity_bonus < 0
        || config.district_rules.storage_volume_per_tile < 0
        || config.district_rules.patrol_mitigation_multiplier < 1.0
    {
        return Err(
            "game_config.json: district rules must provide non-negative bonuses".to_owned(),
        );
    }
    let durations = [
        config.research_durations.binding_routines,
        config.research_durations.gravecraft,
        config.research_durations.ossuary_logistics,
        config.research_durations.domain_stewardship,
    ];
    if durations.iter().any(|duration| *duration <= 0.0) {
        return Err("game_config.json: research durations must be positive".to_owned());
    }
    let costs = [
        config.research_costs.binding_routines,
        config.research_costs.gravecraft,
        config.research_costs.ossuary_logistics,
        config.research_costs.domain_stewardship,
    ];
    if costs
        .iter()
        .any(|cost| cost.bones < 0 || cost.mana < 0 || (cost.bones == 0 && cost.mana == 0))
    {
        return Err(
            "game_config.json: research costs must be non-zero and non-negative".to_owned(),
        );
    }
    Ok(())
}

fn validate_balances(data: &GameData) -> Result<(), String> {
    for (id, job) in data.jobs.iter() {
        if job.work_seconds <= 0.0 || job.base_speed <= 0.0 || job.output_amount < 0 {
            return Err(format!("jobs.json: invalid balance for '{id}'"));
        }
    }
    for (id, undead) in data.undead.iter() {
        if undead.bones_cost < 0 || undead.mana_cost < 0 || undead.work_speed <= 0.0 {
            return Err(format!("undead.json: impossible cost or speed for '{id}'"));
        }
    }
    for (id, building) in data.buildings.iter() {
        validate_building(id, building)?;
    }
    for (id, event) in data.events.iter() {
        if event.choices.is_empty() || event.choices.iter().any(|choice| choice.id.is_empty()) {
            return Err(format!(
                "events.json: event '{id}' needs choices with stable IDs"
            ));
        }
    }
    for (id, band) in data.corpse_bands.iter() {
        if !(0.0..=1.0).contains(&band.min_integrity) || !(0.0..=1.0).contains(&band.min_strength) {
            return Err(format!("corpses.json: invalid quality range for '{id}'"));
        }
    }
    Ok(())
}

fn validate_building(id: &str, building: &BuildingDef) -> Result<(), String> {
    let invalid = building.bones_cost < 0
        || building.mana_cost < 0
        || building.wood_cost < 0
        || building.upgrade_bones_cost < 0
        || building.upgrade_mana_cost < 0
        || building.upgrade_wood_cost < 0
        || building.build_seconds <= 0.0
        || building.speed_multiplier <= 0.0
        || building.suspicion_multiplier <= 0.0
        || building.upgrade_speed_multiplier < 1.0
        || building.upgrade_suspicion_multiplier <= 0.0
        || building.upgrade_effect_text.is_empty();
    if invalid {
        return Err(format!(
            "buildings.json: impossible cost or time for '{id}'"
        ));
    }
    let Some(production) = &building.production else {
        return Ok(());
    };
    if production.recipes.is_empty()
        || production.recipes.iter().any(|recipe| {
            recipe.name.is_empty()
                || recipe.bones_cost < 0
                || recipe.wood_cost < 0
                || recipe.seconds <= 0.0
                || recipe.output_amount <= 0
                || !recipe.suspicion_delta.is_finite()
        })
    {
        return Err(format!(
            "buildings.json: impossible production recipe for '{id}'"
        ));
    }
    let unique_kinds = production
        .recipes
        .iter()
        .map(|recipe| recipe.kind)
        .collect::<HashSet<_>>();
    if unique_kinds.len() != production.recipes.len()
        || !unique_kinds.contains(&ProductionRecipeKind::WardCharge)
    {
        return Err(format!(
            "buildings.json: production recipes for '{id}' need one unique ward_charge entry"
        ));
    }
    Ok(())
}

fn validate_references(data: &GameData) -> Result<(), String> {
    for required in ["poor", "sound", "notable"] {
        if !data.corpse_bands.contains(required) {
            return Err(format!("corpses.json: missing quality band '{required}'"));
        }
    }
    if !data
        .corpse_bands
        .get("notable")
        .is_some_and(|band| band.brute_eligible)
    {
        return Err("corpses.json: notable band must permit Brute resurrection".to_owned());
    }
    for stage in [
        SuspicionStage::Rumour,
        SuspicionStage::Questioning,
        SuspicionStage::Investigation,
    ] {
        if data.event_for_stage(stage).is_none() {
            return Err(format!("events.json: missing event for {stage:?}"));
        }
    }
    for required in ["dig", "haul", "guard", "wood", "refine"] {
        if !data.jobs.contains(required) {
            return Err(format!("jobs.json: missing required job '{required}'"));
        }
    }
    for required in ["skeleton", "brute_skeleton"] {
        if !data.undead.contains(required) {
            return Err(format!("undead.json: missing required type '{required}'"));
        }
    }
    for required in ["work_shed", "grave_lantern", "ossuary_kiln"] {
        if !data.buildings.contains(required) {
            return Err(format!(
                "buildings.json: missing required building '{required}'"
            ));
        }
    }
    Ok(())
}

fn validate_world_layout(config: &GameConfig) -> Result<(), String> {
    let layout = &config.world_layout;
    if layout.road_x <= 0 || layout.road_x >= config.world_width as i32 {
        return Err("game_config.json: road must divide the world".to_owned());
    }
    for (name, tile) in [
        ("mana source", layout.mana_source),
        ("stockpile", layout.stockpile_position),
        ("necromancer", layout.necromancer_position),
    ] {
        if !valid_tile(tile, config) {
            return Err(format!(
                "game_config.json: {name} position is outside the world"
            ));
        }
    }
    if layout.forest_tiles.is_empty() || !unique_tiles(&layout.forest_tiles) {
        return Err("game_config.json: forest tiles must be non-empty and unique".to_owned());
    }
    if layout
        .forest_tiles
        .iter()
        .any(|tile| !valid_tile(*tile, config) || tile.x >= layout.road_x)
    {
        return Err("game_config.json: forest tiles must be inside the hidden side".to_owned());
    }
    if layout.plot_positions.len() < config.victory_plots || !unique_tiles(&layout.plot_positions) {
        return Err(
            "game_config.json: plot layout must cover the victory target uniquely".to_owned(),
        );
    }
    if layout
        .plot_positions
        .iter()
        .any(|tile| !valid_tile(*tile, config) || tile.x >= layout.road_x)
    {
        return Err("game_config.json: plot positions must be inside the hidden side".to_owned());
    }
    let ids = layout
        .building_positions
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<HashSet<_>>();
    if ids.len() != layout.building_positions.len()
        || ["work_shed", "grave_lantern", "ossuary_kiln"]
            .iter()
            .any(|id| !ids.contains(id))
        || layout.building_positions.iter().any(|entry| {
            entry.id.is_empty()
                || !valid_tile(entry.position, config)
                || entry.position.x >= layout.road_x
        })
    {
        return Err(
            "game_config.json: building positions must reference unique in-world tiles".to_owned(),
        );
    }
    Ok(())
}

fn validate_starting_content(data: &GameData) -> Result<(), String> {
    let starting = &data.config.starting_content;
    if starting.worker.id == 0
        || starting.worker.name.is_empty()
        || !data.undead.contains(&starting.worker.undead_id)
        || !data.jobs.contains(&starting.worker.job_id)
        || starting.initial_feedback.is_empty()
        || starting.initial_reason.is_empty()
    {
        return Err(
            "game_config.json: starting content contains an invalid worker or message".to_owned(),
        );
    }
    if starting.worker.undead_id != "skeleton" || starting.worker.job_id != "dig" {
        return Err(
            "game_config.json: the starting worker must be a skeleton assigned to dig".to_owned(),
        );
    }
    Ok(())
}

fn validate_text(text: &GameText) -> Result<(), String> {
    let required = [
        "new_game",
        "victory",
        "assets_loaded",
        "movement_cancelled_feed",
        "movement_cancelled_notification",
        "movement_started_feed",
        "movement_started_notification",
        "automation_locked",
        "production_cancelled",
        "save_success",
        "load_success",
        "placement_guidance",
        "grave_open_hint",
        "grave_ready_hint",
        "lantern_hint",
        "invalid_zone_tile",
    ];
    let values = [
        &text.new_game,
        &text.victory,
        &text.assets_loaded,
        &text.movement_cancelled_feed,
        &text.movement_cancelled_notification,
        &text.movement_started_feed,
        &text.movement_started_notification,
        &text.automation_locked,
        &text.production_cancelled,
        &text.save_success,
        &text.load_success,
        &text.placement_guidance,
        &text.grave_open_hint,
        &text.grave_ready_hint,
        &text.lantern_hint,
        &text.invalid_zone_tile,
    ];
    if values.iter().any(|value| value.trim().is_empty()) {
        return Err(format!("text.json: {required:?} must not be empty"));
    }
    validate_text_entries(
        &text.research,
        [
            "binding_routines",
            "gravecraft",
            "ossuary_logistics",
            "domain_stewardship",
        ],
    )?;
    validate_text_entries(&text.stewardship, ["balanced", "secure", "harvest"])
}

fn validate_text_entries<T, const N: usize>(
    entries: &[T],
    required: [&str; N],
) -> Result<(), String>
where
    T: TextEntry,
{
    let ids = entries.iter().map(TextEntry::id).collect::<HashSet<_>>();
    if ids.len() != entries.len()
        || required.iter().any(|id| !ids.contains(id))
        || entries.iter().any(|entry| {
            entry.id().is_empty() || entry.label().is_empty() || entry.description().is_empty()
        })
    {
        return Err(
            "text.json: authored entries need unique IDs, labels, and descriptions".to_owned(),
        );
    }
    Ok(())
}

trait TextEntry {
    fn id(&self) -> &str;
    fn label(&self) -> &str;
    fn description(&self) -> &str;
}

impl TextEntry for ResearchTextDef {
    fn id(&self) -> &str {
        &self.id
    }
    fn label(&self) -> &str {
        &self.label
    }
    fn description(&self) -> &str {
        &self.description
    }
}

impl TextEntry for StewardshipTextDef {
    fn id(&self) -> &str {
        &self.id
    }
    fn label(&self) -> &str {
        &self.label
    }
    fn description(&self) -> &str {
        &self.description
    }
}

fn valid_tile(tile: TilePos, config: &GameConfig) -> bool {
    tile.x >= 0
        && tile.y >= 0
        && tile.x < config.world_width as i32
        && tile.y < config.world_height as i32
}

fn unique_tiles(tiles: &[TilePos]) -> bool {
    tiles.iter().copied().collect::<HashSet<_>>().len() == tiles.len()
}
