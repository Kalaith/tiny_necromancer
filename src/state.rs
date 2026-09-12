//! Serializable Tiny Necromancer session state and versioned save migration.

use crate::data::{GameConfig, SuspicionStage};
use macroquad_toolkit::grid::TilePos;
use serde::{Deserialize, Serialize};

mod building_upgrades;
mod economy;
mod market;
mod production;
mod research;
mod route_policy;
mod session;
mod stewardship;
mod workforce;
pub use crate::data::ProductionRecipeKind;
pub use building_upgrades::BuildingUpgrades;
pub use economy::{EconomyState, LooseResourcePile};
pub use market::{
    MarketContract, MarketState, MARKET_CONTRACT_BONUS_FAVOR, MARKET_CONTRACT_SECONDS,
};
pub use production::ProductionLedger;
pub use research::ResearchState;
pub use route_policy::{DistrictRoutePolicies, RoutePolicy};
pub use session::{migrate_save_value, GameSession};
pub use stewardship::StewardshipPolicy;
pub use workforce::{HaulDestination, HaulPlan, Worker, WorkforceState};

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
    pub fn id(self) -> &'static str {
        match self {
            Self::BindingRoutines => "binding_routines",
            Self::Gravecraft => "gravecraft",
            Self::OssuaryLogistics => "ossuary_logistics",
            Self::DomainStewardship => "domain_stewardship",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BindingRoutines => "Binding Routines",
            Self::Gravecraft => "Gravecraft",
            Self::OssuaryLogistics => "Ossuary Logistics",
            Self::DomainStewardship => "Domain Stewardship",
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
    #[serde(default = "default_missing_tile")]
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

fn default_building_width() -> i32 {
    2
}

fn default_building_height() -> i32 {
    2
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
    #[serde(default = "default_missing_tile")]
    pub stockpile_position: TilePos,
    pub forest_tiles: Vec<TilePos>,
    #[serde(default = "default_missing_tile")]
    pub necromancer_position: TilePos,
    #[serde(default)]
    pub necromancer_destination: Option<TilePos>,
    #[serde(default)]
    pub selected: Option<Selection>,
    #[serde(default)]
    pub zones: Vec<Zone>,
    #[serde(default)]
    pub route_policies: DistrictRoutePolicies,
}

fn default_missing_tile() -> TilePos {
    TilePos::new(-1, -1)
}

impl WorldState {
    pub fn guard_position(road_x: i32) -> TilePos {
        TilePos::new(road_x.saturating_sub(1), 1)
    }

    pub fn stockpile_position(&self) -> TilePos {
        self.stockpile_position
    }

    pub fn storage_position(&self) -> TilePos {
        self.storage_position_for(self.stockpile_position)
    }

    pub fn storage_position_for(&self, origin: TilePos) -> TilePos {
        self.zones
            .iter()
            .filter(|zone| zone.kind == ZoneKind::Storage)
            .flat_map(|zone| zone.tiles.iter().copied())
            .min_by_key(|tile| tile_distance(origin, *tile))
            .unwrap_or(self.stockpile_position)
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
    #[serde(default)]
    pub production_recipe: ProductionRecipeKind,
    #[serde(default)]
    pub production_ledger: ProductionLedger,
    #[serde(default)]
    pub building_upgrades: BuildingUpgrades,
    #[serde(default)]
    pub district_ledger: DistrictLedger,
    #[serde(default)]
    pub market: MarketState,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistrictLedger {
    #[serde(default)]
    pub work_cycles: u32,
    #[serde(default)]
    pub storage_bonus_items: i32,
    #[serde(default)]
    pub patrol_quieting: f32,
    #[serde(default)]
    pub recent_activity: Vec<DistrictActivity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistrictActivityKind {
    WorkCycle,
    StorageBonus,
    PatrolQuieting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictActivity {
    pub kind: DistrictActivityKind,
    pub amount: f32,
    pub elapsed_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub building: BuildingKind,
    pub progress: f32,
    #[serde(default)]
    pub recipe: ProductionRecipeKind,
    #[serde(default)]
    pub bones_remaining: i32,
    #[serde(default)]
    pub wood_remaining: i32,
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

pub fn next_plot_unlock_cost(config: &GameConfig, unlocked_plots: usize) -> i32 {
    config.plot_unlock_base_wood + unlocked_plots as i32 * config.plot_unlock_step_wood
}
