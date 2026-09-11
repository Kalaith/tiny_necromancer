//! Embedded balance data, authored content, and startup validation.

use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::{load_embedded_json_labeled, DataRegistry};
use serde::{Deserialize, Serialize};

const CONFIG_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/game_config.json");
const JOBS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/jobs.json");
const UNDEAD_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/undead.json");
const BUILDINGS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/buildings.json");
const CORPSES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/corpses.json");
const EVENTS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/events.json");
const TEXTURES_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/texture_manifest.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub game_name: String,
    pub display_name: String,
    pub save_slot: String,
    pub version: String,
    pub world_width: usize,
    pub world_height: usize,
    pub tick_seconds: f32,
    pub starting_bones: i32,
    pub starting_mana: i32,
    pub starting_wood: i32,
    pub storage_capacity: i32,
    pub max_mana: i32,
    pub worker_wood_reserve: i32,
    pub starting_unlocked_plots: usize,
    pub starting_seed: u64,
    pub mana_regen_per_second: f32,
    pub corpse_discovery_chance: f32,
    pub victory_undead: usize,
    pub victory_plots: usize,
    pub victory_suspicion_max: f32,
    pub suspicion_thresholds: [f32; 3],
    pub plot_unlock_base_wood: i32,
    pub plot_unlock_step_wood: i32,
    pub district_rules: DistrictRules,
    pub research_durations: ResearchDurations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictRules {
    pub work_speed_multiplier: f32,
    pub storage_capacity_bonus: i32,
    pub storage_volume_per_tile: i32,
    pub patrol_mitigation_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchDurations {
    pub binding_routines: f32,
    pub gravecraft: f32,
    pub ossuary_logistics: f32,
    pub domain_stewardship: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub work_seconds: f32,
    pub output_amount: i32,
    pub base_speed: f32,
    pub suspicion_per_cycle: f32,
    pub guard_mitigation_per_second: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndeadDef {
    pub id: String,
    pub name: String,
    pub bones_cost: i32,
    pub mana_cost: i32,
    pub work_speed: f32,
    pub haul_capacity: i32,
    pub conspicuousness: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDef {
    pub id: String,
    pub name: String,
    pub bones_cost: i32,
    pub mana_cost: i32,
    pub wood_cost: i32,
    pub build_seconds: f32,
    pub speed_multiplier: f32,
    pub suspicion_multiplier: f32,
    pub suspicion_delta: f32,
    pub effect_text: String,
    pub upgrade_bones_cost: i32,
    pub upgrade_mana_cost: i32,
    pub upgrade_wood_cost: i32,
    pub upgrade_speed_multiplier: f32,
    pub upgrade_suspicion_multiplier: f32,
    pub upgrade_effect_text: String,
    #[serde(default)]
    pub production: Option<ProductionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionDef {
    pub bones_cost: i32,
    pub wood_cost: i32,
    pub seconds: f32,
    pub output_amount: i32,
    pub effect_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpseBandDef {
    pub id: String,
    pub name: String,
    pub min_integrity: f32,
    pub min_strength: f32,
    pub brute_eligible: bool,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuspicionStage {
    Calm,
    Rumour,
    Questioning,
    Investigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChoiceDef {
    pub id: String,
    pub label: String,
    pub description: String,
    pub suspicion_delta: f32,
    pub bones_delta: i32,
    pub mana_delta: i32,
    pub wood_delta: i32,
    pub pause_digging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    pub id: String,
    pub stage: SuspicionStage,
    pub title: String,
    pub body: String,
    pub choices: Vec<EventChoiceDef>,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub jobs: DataRegistry<JobDef>,
    pub undead: DataRegistry<UndeadDef>,
    pub buildings: DataRegistry<BuildingDef>,
    pub corpse_bands: DataRegistry<CorpseBandDef>,
    pub events: DataRegistry<EventDef>,
    pub texture_manifest: Vec<TextureConfig>,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let config = load_embedded_json_labeled("game_config.json", CONFIG_JSON)?;
        let jobs = DataRegistry::from_embedded_json(JOBS_JSON, "id")
            .map_err(|error| format!("jobs.json: {error}"))?;
        let undead = DataRegistry::from_embedded_json(UNDEAD_JSON, "id")
            .map_err(|error| format!("undead.json: {error}"))?;
        let buildings = DataRegistry::from_embedded_json(BUILDINGS_JSON, "id")
            .map_err(|error| format!("buildings.json: {error}"))?;
        let corpse_bands = DataRegistry::from_embedded_json(CORPSES_JSON, "id")
            .map_err(|error| format!("corpses.json: {error}"))?;
        let events = DataRegistry::from_embedded_json(EVENTS_JSON, "id")
            .map_err(|error| format!("events.json: {error}"))?;
        let texture_manifest = load_embedded_json_labeled("texture_manifest.json", TEXTURES_JSON)?;
        let data = Self {
            config,
            jobs,
            undead,
            buildings,
            corpse_bands,
            events,
            texture_manifest,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<(), String> {
        let config = &self.config;
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
            return Err(
                "game_config.json: starting resources or plot targets are invalid".to_owned(),
            );
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
        if [
            config.research_durations.binding_routines,
            config.research_durations.gravecraft,
            config.research_durations.ossuary_logistics,
            config.research_durations.domain_stewardship,
        ]
        .iter()
        .any(|duration| *duration <= 0.0)
        {
            return Err("game_config.json: research durations must be positive".to_owned());
        }
        for (id, job) in self.jobs.iter() {
            if job.work_seconds <= 0.0 || job.base_speed <= 0.0 || job.output_amount < 0 {
                return Err(format!("jobs.json: invalid balance for '{id}'"));
            }
        }
        for (id, undead) in self.undead.iter() {
            if undead.bones_cost < 0 || undead.mana_cost < 0 || undead.work_speed <= 0.0 {
                return Err(format!("undead.json: impossible cost or speed for '{id}'"));
            }
        }
        for (id, building) in self.buildings.iter() {
            if building.bones_cost < 0
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
                || building.upgrade_effect_text.is_empty()
            {
                return Err(format!(
                    "buildings.json: impossible cost or time for '{id}'"
                ));
            }
            if let Some(production) = &building.production {
                if production.bones_cost < 0
                    || production.wood_cost < 0
                    || production.seconds <= 0.0
                    || production.output_amount <= 0
                {
                    return Err(format!(
                        "buildings.json: impossible production recipe for '{id}'"
                    ));
                }
            }
        }
        for (id, event) in self.events.iter() {
            if event.choices.is_empty() || event.choices.iter().any(|choice| choice.id.is_empty()) {
                return Err(format!(
                    "events.json: event '{id}' needs choices with stable IDs"
                ));
            }
        }
        for (id, band) in self.corpse_bands.iter() {
            if !(0.0..=1.0).contains(&band.min_integrity)
                || !(0.0..=1.0).contains(&band.min_strength)
            {
                return Err(format!("corpses.json: invalid quality range for '{id}'"));
            }
        }
        for required in ["poor", "sound", "notable"] {
            if !self.corpse_bands.contains(required) {
                return Err(format!("corpses.json: missing quality band '{required}'"));
            }
        }
        if !self
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
            if self.event_for_stage(stage).is_none() {
                return Err(format!("events.json: missing event for {stage:?}"));
            }
        }
        for required in ["dig", "haul", "guard", "wood", "refine"] {
            if !self.jobs.contains(required) {
                return Err(format!("jobs.json: missing required job '{required}'"));
            }
        }
        for required in ["skeleton", "brute_skeleton"] {
            if !self.undead.contains(required) {
                return Err(format!("undead.json: missing required type '{required}'"));
            }
        }
        for required in ["work_shed", "grave_lantern", "ossuary_kiln"] {
            if !self.buildings.contains(required) {
                return Err(format!(
                    "buildings.json: missing required building '{required}'"
                ));
            }
        }
        Ok(())
    }

    pub fn event_for_stage(&self, stage: SuspicionStage) -> Option<&EventDef> {
        self.events
            .iter()
            .map(|(_, event)| event)
            .find(|event| event.stage == stage)
    }
}

#[cfg(test)]
mod tests;
