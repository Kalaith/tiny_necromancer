//! Serializable Tiny Necromancer session state and versioned save migration.

use crate::data::{GameConfig, SuspicionStage};
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod research;
mod stewardship;
mod workforce;
pub use research::ResearchState;
pub use stewardship::StewardshipPolicy;
pub use workforce::{Worker, WorkforceState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    MainMenu,
    Playing,
    Paused,
    Victory,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotStatus {
    Locked,
    Ready,
    Digging,
    Dug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingKind {
    WorkShed,
    GraveLantern,
    OssuaryKiln,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Technology {
    BindingRoutines,
    Gravecraft,
    OssuaryLogistics,
    DomainStewardship,
}

impl Technology {
    pub fn label(self) -> &'static str {
        match self {
            Self::BindingRoutines => "Binding Routines",
            Self::Gravecraft => "Gravecraft",
            Self::OssuaryLogistics => "Ossuary Logistics",
            Self::DomainStewardship => "Domain Stewardship",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::BindingRoutines => "Repeat orders and worker priorities become reliable.",
            Self::Gravecraft => "Place structures and paint work areas across the clearing.",
            Self::OssuaryLogistics => "Link stockpiles and queue specialized production.",
            Self::DomainStewardship => "See districts, patrol routes, and ward coverage.",
        }
    }

    pub fn prerequisite(self) -> Option<Self> {
        match self {
            Self::BindingRoutines => None,
            Self::Gravecraft => Some(Self::BindingRoutines),
            Self::OssuaryLogistics => Some(Self::Gravecraft),
            Self::DomainStewardship => Some(Self::OssuaryLogistics),
        }
    }

    pub fn duration(self, config: &GameConfig) -> f32 {
        match self {
            Self::BindingRoutines => config.research_durations.binding_routines,
            Self::Gravecraft => config.research_durations.gravecraft,
            Self::OssuaryLogistics => config.research_durations.ossuary_logistics,
            Self::DomainStewardship => config.research_durations.domain_stewardship,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selection {
    Grave(usize),
    Worker(usize),
    Building(usize),
    Ground(TilePos),
    Necromancer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneKind {
    Work,
    Storage,
    Patrol,
}

impl ZoneKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Work => "Work",
            Self::Storage => "Storage",
            Self::Patrol => "Patrol",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub kind: ZoneKind,
    pub tiles: Vec<TilePos>,
}

impl Zone {
    pub fn toggle_tile(&mut self, tile: TilePos) -> bool {
        if let Some(index) = self.tiles.iter().position(|marked| *marked == tile) {
            self.tiles.remove(index);
            true
        } else {
            self.tiles.push(tile);
            false
        }
    }
}

impl BuildingKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::WorkShed => "work_shed",
            Self::GraveLantern => "grave_lantern",
            Self::OssuaryKiln => "ossuary_kiln",
        }
    }

    pub fn dimensions(self) -> (i32, i32) {
        match self {
            Self::WorkShed => (2, 2),
            Self::GraveLantern => (1, 1),
            Self::OssuaryKiln => (2, 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UndeadKind {
    Skeleton,
    BruteSkeleton,
}

impl UndeadKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::Skeleton => "skeleton",
            Self::BruteSkeleton => "brute_skeleton",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    Dig,
    Haul,
    Guard,
    Wood,
    Build,
    Refine,
}

impl JobKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::Dig => "dig",
            Self::Haul => "haul",
            Self::Guard => "guard",
            Self::Wood => "wood",
            Self::Build => "build",
            Self::Refine => "refine",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dig => "Dig",
            Self::Haul => "Haul",
            Self::Guard => "Guard",
            Self::Wood => "Wood",
            Self::Build => "Build",
            Self::Refine => "Refine",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Walking,
    Working,
    Carrying,
    Hiding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Bones,
    Wood,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plot {
    pub id: usize,
    pub position: TilePos,
    pub status: PlotStatus,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub kind: BuildingKind,
    pub progress: f32,
    pub complete: bool,
    #[serde(default = "default_building_position")]
    pub position: TilePos,
    #[serde(default = "default_building_width")]
    pub width: i32,
    #[serde(default = "default_building_height")]
    pub height: i32,
}

impl Building {
    pub fn work_position(&self) -> TilePos {
        if self.position.x > 0 {
            TilePos::new(self.position.x - 1, self.position.y)
        } else {
            TilePos::new(self.position.x + self.width, self.position.y)
        }
    }
}

fn default_building_position() -> TilePos {
    TilePos::new(6, 2)
}

fn default_building_width() -> i32 {
    2
}

fn default_building_height() -> i32 {
    2
}

pub fn default_building_position_for_kind(kind: BuildingKind) -> TilePos {
    match kind {
        BuildingKind::WorkShed => TilePos::new(6, 2),
        BuildingKind::GraveLantern => TilePos::new(7, 6),
        BuildingKind::OssuaryKiln => TilePos::new(6, 4),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub width: usize,
    pub height: usize,
    pub selected_plot: Option<usize>,
    pub plots: Vec<Plot>,
    pub buildings: Vec<Building>,
    pub road_x: i32,
    pub mana_source: TilePos,
    pub forest_tiles: Vec<TilePos>,
    #[serde(default = "default_necromancer_position")]
    pub necromancer_position: TilePos,
    #[serde(default)]
    pub necromancer_destination: Option<TilePos>,
    #[serde(default)]
    pub selected: Option<Selection>,
    #[serde(default)]
    pub zones: Vec<Zone>,
}

fn default_necromancer_position() -> TilePos {
    TilePos::new(5, 5)
}

impl WorldState {
    pub fn guard_position(road_x: i32) -> TilePos {
        TilePos::new(road_x.saturating_sub(1), 1)
    }

    pub fn stockpile_position() -> TilePos {
        TilePos::new(6, 5)
    }

    pub fn storage_position(&self) -> TilePos {
        self.storage_position_for(Self::stockpile_position())
    }

    pub fn storage_position_for(&self, origin: TilePos) -> TilePos {
        self.zones
            .iter()
            .filter(|zone| zone.kind == ZoneKind::Storage)
            .flat_map(|zone| zone.tiles.iter().copied())
            .min_by_key(|tile| tile_distance(origin, *tile))
            .unwrap_or_else(Self::stockpile_position)
    }

    pub fn patrol_position(&self) -> TilePos {
        self.zone_anchor(ZoneKind::Patrol, Self::guard_position(self.road_x))
    }

    pub fn patrol_position_for(&self, origin: TilePos) -> TilePos {
        self.zones
            .iter()
            .filter(|zone| zone.kind == ZoneKind::Patrol)
            .flat_map(|zone| zone.tiles.iter().copied())
            .min_by_key(|tile| tile_distance(origin, *tile))
            .unwrap_or_else(|| Self::guard_position(self.road_x))
    }

    pub fn zone_anchor(&self, kind: ZoneKind, fallback: TilePos) -> TilePos {
        self.zones
            .iter()
            .find(|zone| zone.kind == kind)
            .and_then(|zone| zone.tiles.first().copied())
            .unwrap_or(fallback)
    }

    pub fn zone_contains(&self, kind: ZoneKind, tile: TilePos) -> bool {
        self.zones
            .iter()
            .any(|zone| zone.kind == kind && zone.tiles.contains(&tile))
    }

    pub fn is_zone_tile_allowed(&self, tile: TilePos) -> bool {
        tile.x >= 0
            && tile.y >= 0
            && tile.x < self.width as i32
            && tile.y < self.height as i32
            && tile.x < self.road_x
            && !self.buildings.iter().any(|building| {
                tile.x >= building.position.x
                    && tile.x < building.position.x + building.width
                    && tile.y >= building.position.y
                    && tile.y < building.position.y + building.height
            })
    }

    pub fn is_building_obstacle(&self, tile: TilePos) -> bool {
        tile.x >= self.road_x
            || self.forest_tiles.contains(&tile)
            || self.plots.iter().any(|plot| plot.position == tile)
    }
}

fn tile_distance(from: TilePos, to: TilePos) -> i32 {
    (from.x - to.x).abs() + (from.y - to.y).abs()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorpseQuality {
    Poor,
    Sound,
    Notable,
}

impl CorpseQuality {
    pub fn label(self) -> &'static str {
        match self {
            Self::Poor => "Poor",
            Self::Sound => "Sound",
            Self::Notable => "Notable",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpse {
    pub id: u32,
    pub integrity: f32,
    pub strength: f32,
    pub skill: f32,
    pub magical_residue: f32,
    pub cause_of_death: String,
    pub quality: CorpseQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyState {
    pub bones: i32,
    pub mana: i32,
    pub mana_fraction: f32,
    pub wood: i32,
    pub shovels: i32,
    pub loose_bones: i32,
    pub loose_wood: i32,
    #[serde(default)]
    pub ward_charges: i32,
    #[serde(default)]
    pub loose_bones_source: Option<TilePos>,
    #[serde(default)]
    pub loose_wood_source: Option<TilePos>,
    pub corpses: Vec<Corpse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEntry {
    pub message: String,
    pub age_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureState {
    pub suspicion: f32,
    pub stage: SuspicionStage,
    pub active_event: Option<String>,
    pub event_history: Vec<String>,
    pub feed: Vec<FeedEntry>,
    pub last_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressState {
    pub unlocked_plots: usize,
    pub elapsed_seconds: f32,
    pub first_corpse_found: bool,
    pub first_building_started: bool,
    #[serde(default)]
    pub production: Option<ProductionOrder>,
    #[serde(default)]
    pub production_queue: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub building: BuildingKind,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub phase: GamePhase,
    pub world: WorldState,
    pub workforce: WorkforceState,
    pub economy: EconomyState,
    pub pressure: PressureState,
    pub progress: ProgressState,
    #[serde(default)]
    pub research: ResearchState,
    #[serde(default)]
    pub stewardship_policy: StewardshipPolicy,
    pub rng_state: u64,
}

#[derive(Debug, Clone)]
pub struct GameSession {
    pub phase: GamePhase,
    pub world: WorldState,
    pub workforce: WorkforceState,
    pub economy: EconomyState,
    pub pressure: PressureState,
    pub progress: ProgressState,
    pub research: ResearchState,
    pub stewardship_policy: StewardshipPolicy,
    pub rng: SeededRng,
    pub error_message: Option<String>,
}

impl GameSession {
    pub fn new(config: &GameConfig) -> Self {
        let plot_positions = [
            TilePos::new(2, 2),
            TilePos::new(3, 2),
            TilePos::new(4, 2),
            TilePos::new(5, 2),
            TilePos::new(2, 3),
            TilePos::new(3, 3),
        ];
        let unlocked = config.starting_unlocked_plots.min(plot_positions.len());
        let plots = plot_positions
            .into_iter()
            .enumerate()
            .map(|(id, position)| Plot {
                id,
                position,
                status: if id < unlocked {
                    PlotStatus::Ready
                } else {
                    PlotStatus::Locked
                },
                progress: 0.0,
            })
            .collect();
        let start = plot_positions[0];
        Self {
            phase: GamePhase::MainMenu,
            world: WorldState {
                width: config.world_width,
                height: config.world_height,
                selected_plot: Some(0),
                plots,
                buildings: Vec::new(),
                road_x: 8,
                mana_source: TilePos::new(7, 6),
                forest_tiles: (0..8).map(|y| TilePos::new(0, y)).collect(),
                necromancer_position: default_necromancer_position(),
                necromancer_destination: None,
                selected: Some(Selection::Grave(0)),
                zones: Vec::new(),
            },
            workforce: WorkforceState {
                workers: vec![Worker {
                    id: 1,
                    name: "Rattlebones".to_owned(),
                    kind: UndeadKind::Skeleton,
                    assignment: JobKind::Dig,
                    position: start,
                    status: WorkerStatus::Idle,
                    progress: 0.0,
                    target_plot: None,
                    carrying: 0,
                    carrying_resource: None,
                    priority_mode: false,
                }],
                selected_worker: 0,
                next_worker_id: 2,
                priorities: WorkforceState::default_priorities(),
            },
            economy: EconomyState {
                bones: config.starting_bones,
                mana: config.starting_mana,
                mana_fraction: 0.0,
                wood: config.starting_wood,
                shovels: 1,
                loose_bones: 0,
                loose_wood: 0,
                ward_charges: 0,
                loose_bones_source: None,
                loose_wood_source: None,
                corpses: Vec::new(),
            },
            pressure: PressureState {
                suspicion: 0.0,
                stage: SuspicionStage::Calm,
                active_event: None,
                event_history: Vec::new(),
                feed: vec![FeedEntry {
                    message: "The shed is empty, the grave is waiting.".to_owned(),
                    age_seconds: 0.0,
                }],
                last_reason: "Quiet cemetery".to_owned(),
            },
            progress: ProgressState {
                unlocked_plots: unlocked,
                elapsed_seconds: 0.0,
                first_corpse_found: false,
                first_building_started: false,
                production: None,
                production_queue: 0,
            },
            research: ResearchState::default(),
            stewardship_policy: StewardshipPolicy::default(),
            rng: SeededRng::new(config.starting_seed),
            error_message: None,
        }
    }

    pub fn begin(&mut self) {
        if self.phase == GamePhase::MainMenu {
            self.phase = GamePhase::Playing;
        }
    }

    pub fn to_save(&self, version: &str) -> SaveData {
        SaveData {
            version: version.to_owned(),
            phase: self.phase,
            world: self.world.clone(),
            workforce: self.workforce.clone(),
            economy: self.economy.clone(),
            pressure: self.pressure.clone(),
            progress: self.progress.clone(),
            research: self.research.clone(),
            stewardship_policy: self.stewardship_policy,
            rng_state: self.rng.state(),
        }
    }

    pub fn from_save(mut save: SaveData) -> Self {
        for building in &mut save.world.buildings {
            if building.position == default_building_position() {
                building.position = default_building_position_for_kind(building.kind);
            }
            let (width, height) = building.kind.dimensions();
            building.width = width;
            building.height = height;
        }
        for worker in &mut save.workforce.workers {
            if worker.carrying > 0 && worker.carrying_resource.is_none() {
                worker.carrying_resource = Some(ResourceKind::Bones);
            }
        }
        save.workforce.normalize_priorities();
        Self {
            phase: save.phase,
            world: save.world,
            workforce: save.workforce,
            economy: save.economy,
            pressure: save.pressure,
            progress: save.progress,
            research: save.research,
            stewardship_policy: save.stewardship_policy,
            rng: SeededRng::from_state(save.rng_state),
            error_message: None,
        }
    }

    pub fn active_undead(&self) -> usize {
        self.workforce.workers.len()
    }

    pub fn world_width(&self) -> usize {
        self.world.width
    }

    pub fn world_height(&self) -> usize {
        self.world.height
    }

    pub fn has_building(&self, kind: BuildingKind) -> bool {
        self.world
            .buildings
            .iter()
            .any(|building| building.kind == kind && building.complete)
    }

    pub fn building_in_progress(&self, kind: BuildingKind) -> bool {
        self.world
            .buildings
            .iter()
            .any(|building| building.kind == kind && !building.complete)
    }

    pub fn add_feed(&mut self, message: impl Into<String>) {
        self.pressure.feed.insert(
            0,
            FeedEntry {
                message: message.into(),
                age_seconds: 0.0,
            },
        );
        self.pressure.feed.truncate(7);
    }

    pub fn tick_feed(&mut self, dt: f32) {
        for entry in &mut self.pressure.feed {
            entry.age_seconds += dt;
        }
    }
}

pub fn next_plot_unlock_cost(config: &GameConfig, unlocked_plots: usize) -> i32 {
    config.plot_unlock_base_wood + unlocked_plots as i32 * config.plot_unlock_step_wood
}

#[derive(Debug, Default, Deserialize)]
struct LegacySave {
    points: Option<i64>,
    energy: Option<f32>,
    turn: Option<u32>,
    player: Option<LegacyPlayer>,
}

#[derive(Debug, Deserialize)]
struct LegacyPlayer {
    points: Option<i64>,
    energy: Option<f32>,
    turn: Option<u32>,
}

pub fn migrate_save_value(
    detected_version: Option<String>,
    value: Value,
    config: &GameConfig,
) -> Result<SaveData, String> {
    let payload = value.get("data").cloned().unwrap_or(value);
    if let Ok(mut current) = serde_json::from_value::<SaveData>(payload.clone()) {
        current.version = config.version.clone();
        return Ok(current);
    }
    let legacy: LegacySave = serde_json::from_value(payload)
        .map_err(|error| format!("Unsupported save format {:?}: {error}", detected_version))?;
    let mut session = GameSession::new(config);
    let legacy_player = legacy.player;
    if let Some(points) = legacy
        .points
        .or_else(|| legacy_player.as_ref().and_then(|player| player.points))
    {
        session.economy.bones = points.clamp(0, i32::MAX as i64) as i32;
    }
    if let Some(energy) = legacy
        .energy
        .or_else(|| legacy_player.as_ref().and_then(|player| player.energy))
    {
        session.economy.mana = energy.max(0.0) as i32;
    }
    if let Some(turn) = legacy
        .turn
        .or_else(|| legacy_player.as_ref().and_then(|player| player.turn))
    {
        session.progress.elapsed_seconds = turn.saturating_sub(1) as f32 * config.tick_seconds;
    }
    session.phase = GamePhase::Playing;
    Ok(session.to_save(&config.version))
}

#[cfg(test)]
mod tests;
