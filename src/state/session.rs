//! Session construction, save normalization, and legacy save migration.

use super::*;
use crate::data::GameConfig;
use macroquad_toolkit::rng::SeededRng;
use serde::Deserialize;
use serde_json::Value;

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
        let unlocked = config
            .starting_unlocked_plots
            .min(config.world_layout.plot_positions.len());
        Self {
            phase: GamePhase::MainMenu,
            world: world_state(config, unlocked),
            workforce: workforce_state(config),
            economy: EconomyState {
                bones: config.starting_bones,
                mana: config.starting_mana,
                mana_fraction: 0.0,
                wood: config.starting_wood,
                storage_capacity: config.storage_capacity,
                shovels: 1,
                loose_bones: 0,
                loose_wood: 0,
                ward_charges: 0,
                loose_bones_source: None,
                loose_wood_source: None,
                loose_bones_piles: Vec::new(),
                loose_wood_piles: Vec::new(),
                corpses: Vec::new(),
            },
            pressure: PressureState {
                suspicion: 0.0,
                stage: SuspicionStage::Calm,
                active_event: None,
                event_history: Vec::new(),
                feed: vec![FeedEntry {
                    message: config.starting_content.initial_feedback.clone(),
                    age_seconds: 0.0,
                }],
                last_reason: config.starting_content.initial_reason.clone(),
            },
            progress: ProgressState {
                unlocked_plots: unlocked,
                elapsed_seconds: 0.0,
                first_corpse_found: false,
                first_building_started: false,
                production: None,
                production_queue: 0,
                production_recipe: ProductionRecipeKind::default(),
                production_ledger: ProductionLedger::default(),
                building_upgrades: BuildingUpgrades::default(),
                district_ledger: DistrictLedger::default(),
                market: MarketState::default(),
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

    pub fn from_save(mut save: SaveData, config: &GameConfig) -> Self {
        normalize_world(&mut save, config);
        if save.economy.storage_capacity <= 0 {
            save.economy.storage_capacity = economy::DEFAULT_STORAGE_CAPACITY;
        }
        save.progress
            .building_upgrades
            .normalize(&save.world.buildings);
        save.progress.market.normalize();
        normalize_buildings(&mut save, config);
        normalize_workers(&mut save);
        infer_legacy_sources(&mut save);
        save.economy.normalize_loose_piles();
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

fn world_state(config: &GameConfig, unlocked: usize) -> WorldState {
    let plots = config
        .world_layout
        .plot_positions
        .iter()
        .copied()
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
    WorldState {
        width: config.world_width,
        height: config.world_height,
        selected_plot: (unlocked > 0).then_some(0),
        plots,
        buildings: Vec::new(),
        road_x: config.world_layout.road_x,
        mana_source: config.world_layout.mana_source,
        stockpile_position: config.world_layout.stockpile_position,
        forest_tiles: config.world_layout.forest_tiles.clone(),
        necromancer_position: config.world_layout.necromancer_position,
        necromancer_destination: None,
        selected: (unlocked > 0).then_some(Selection::Grave(0)),
        zones: Vec::new(),
        route_policies: DistrictRoutePolicies::default(),
    }
}

fn workforce_state(config: &GameConfig) -> WorkforceState {
    let starting_worker = &config.starting_content.worker;
    let start = config
        .world_layout
        .plot_positions
        .first()
        .copied()
        .unwrap_or_else(super::default_missing_tile);
    WorkforceState {
        workers: vec![Worker {
            id: starting_worker.id,
            name: starting_worker.name.clone(),
            kind: undead_kind_from_id(&starting_worker.undead_id),
            assignment: job_kind_from_id(&starting_worker.job_id),
            position: start,
            status: WorkerStatus::Idle,
            progress: 0.0,
            target_plot: None,
            carrying: 0,
            carrying_resource: None,
            priority_mode: false,
            haul_plan: None,
        }],
        selected_worker: 0,
        next_worker_id: starting_worker.id.saturating_add(1),
        priorities: WorkforceState::default_priorities(),
    }
}

fn normalize_world(save: &mut SaveData, config: &GameConfig) {
    if save.world.necromancer_position == super::default_missing_tile() {
        save.world.necromancer_position = config.world_layout.necromancer_position;
    }
    if save.world.stockpile_position == super::default_missing_tile() {
        save.world.stockpile_position = config.world_layout.stockpile_position;
    }
}

fn normalize_buildings(save: &mut SaveData, config: &GameConfig) {
    let legacy_work_shed_position = config
        .world_layout
        .building_position(BuildingKind::WorkShed.id());
    for building in &mut save.world.buildings {
        if building.position == super::default_missing_tile()
            || Some(building.position) == legacy_work_shed_position
        {
            if let Some(position) = config.world_layout.building_position(building.kind.id()) {
                building.position = position;
            }
        }
        let (width, height) = building.kind.dimensions();
        building.width = width;
        building.height = height;
    }
}

fn normalize_workers(save: &mut SaveData) {
    for worker in &mut save.workforce.workers {
        if worker.carrying > 0 && worker.carrying_resource.is_none() {
            worker.carrying_resource = Some(ResourceKind::Bones);
        }
        if worker.carrying <= 0 {
            worker.haul_plan = None;
        }
    }
}

fn infer_legacy_sources(save: &mut SaveData) {
    if save.economy.loose_bones > 0 && save.economy.loose_bones_source.is_none() {
        save.economy.loose_bones_source = save
            .world
            .plots
            .iter()
            .find(|plot| plot.status == PlotStatus::Dug)
            .map(|plot| plot.position);
    }
    if save.economy.loose_wood > 0 && save.economy.loose_wood_source.is_none() {
        save.economy.loose_wood_source = save.world.forest_tiles.first().copied();
    }
}

fn undead_kind_from_id(id: &str) -> UndeadKind {
    match id {
        "brute_skeleton" => UndeadKind::BruteSkeleton,
        _ => UndeadKind::Skeleton,
    }
}

fn job_kind_from_id(id: &str) -> JobKind {
    match id {
        "haul" => JobKind::Haul,
        "guard" => JobKind::Guard,
        "wood" => JobKind::Wood,
        "build" => JobKind::Build,
        "refine" => JobKind::Refine,
        _ => JobKind::Dig,
    }
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
