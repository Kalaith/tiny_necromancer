//! Embedded balance data, authored content, and startup validation.

use macroquad_toolkit::assets::TextureConfig;
use macroquad_toolkit::data_loader::{load_embedded_json_labeled, DataRegistry};
use macroquad_toolkit::grid::TilePos;
use serde::{Deserialize, Serialize};

const CONFIG_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/game_config.json");
const JOBS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/jobs.json");
const UNDEAD_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/undead.json");
const BUILDINGS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/buildings.json");
const CORPSES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/corpses.json");
const EVENTS_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/events.json");
const TEXT_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/text.json");
const TEXTURES_JSON: &str =
    macroquad_toolkit::include_json_str!("../assets/data/texture_manifest.json");

mod validation;

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
    pub world_layout: WorldLayoutConfig,
    pub starting_content: StartingContentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldLayoutConfig {
    pub road_x: i32,
    pub mana_source: TilePos,
    pub stockpile_position: TilePos,
    pub necromancer_position: TilePos,
    pub forest_tiles: Vec<TilePos>,
    pub plot_positions: Vec<TilePos>,
    pub building_positions: Vec<NamedTilePosition>,
}

impl WorldLayoutConfig {
    pub fn building_position(&self, building_id: &str) -> Option<TilePos> {
        self.building_positions
            .iter()
            .find(|entry| entry.id == building_id)
            .map(|entry| entry.position)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedTilePosition {
    pub id: String,
    pub position: TilePos,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartingContentConfig {
    pub worker: StartingWorkerConfig,
    pub initial_feedback: String,
    pub initial_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartingWorkerConfig {
    pub id: u32,
    pub name: String,
    pub undead_id: String,
    pub job_id: String,
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
    pub recipes: Vec<ProductionRecipeDef>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionRecipeKind {
    #[default]
    WardCharge,
    HushAsh,
}

impl ProductionRecipeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::WardCharge => "Ward Charge",
            Self::HushAsh => "Hush Ash",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::WardCharge => "Ward",
            Self::HushAsh => "Hush",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionRecipeDef {
    pub kind: ProductionRecipeKind,
    pub name: String,
    pub bones_cost: i32,
    pub wood_cost: i32,
    pub seconds: f32,
    pub output_amount: i32,
    pub suspicion_delta: f32,
    pub effect_text: String,
}

impl ProductionDef {
    pub fn recipe(&self, kind: ProductionRecipeKind) -> Option<&ProductionRecipeDef> {
        self.recipes.iter().find(|recipe| recipe.kind == kind)
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameText {
    pub new_game: String,
    pub victory: String,
    pub assets_loaded: String,
    pub movement_cancelled_feed: String,
    pub movement_cancelled_notification: String,
    pub movement_started_feed: String,
    pub movement_started_notification: String,
    pub automation_locked: String,
    pub production_cancelled: String,
    pub save_success: String,
    pub load_success: String,
    pub placement_guidance: String,
    pub grave_open_hint: String,
    pub grave_ready_hint: String,
    pub lantern_hint: String,
    pub invalid_zone_tile: String,
    pub research: Vec<ResearchTextDef>,
    pub stewardship: Vec<StewardshipTextDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTextDef {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StewardshipTextDef {
    pub id: String,
    pub label: String,
    pub description: String,
}

impl GameText {
    pub fn research(&self, id: &str) -> Option<&ResearchTextDef> {
        self.research.iter().find(|entry| entry.id == id)
    }

    pub fn stewardship(&self, id: &str) -> Option<&StewardshipTextDef> {
        self.stewardship.iter().find(|entry| entry.id == id)
    }
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub config: GameConfig,
    pub jobs: DataRegistry<JobDef>,
    pub undead: DataRegistry<UndeadDef>,
    pub buildings: DataRegistry<BuildingDef>,
    pub corpse_bands: DataRegistry<CorpseBandDef>,
    pub events: DataRegistry<EventDef>,
    pub text: GameText,
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
        let text = load_embedded_json_labeled("text.json", TEXT_JSON)?;
        let texture_manifest = load_embedded_json_labeled("texture_manifest.json", TEXTURES_JSON)?;
        let data = Self {
            config,
            jobs,
            undead,
            buildings,
            corpse_bands,
            events,
            text,
            texture_manifest,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<(), String> {
        validation::validate(self)
    }

    pub fn event_for_stage(&self, stage: SuspicionStage) -> Option<&EventDef> {
        self.events
            .iter()
            .map(|(_, event)| event)
            .find(|event| event.stage == stage)
    }
}
