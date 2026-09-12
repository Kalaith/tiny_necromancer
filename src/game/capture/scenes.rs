//! Capture scenes for management panels and district states.

use super::*;
use crate::engine::corpses;
use crate::state::{
    Building, BuildingKind, Corpse, CorpseQuality, DistrictActivity, DistrictActivityKind,
    GamePhase, JobKind, MarketContract, ProductionLedger, RoutePolicy, Selection,
    StewardshipPolicy, Technology, UndeadKind, WorkerStatus, Zone, ZoneKind,
};
use macroquad_toolkit::grid::TilePos;

impl Game {
    fn capture_building(&self, kind: BuildingKind, progress: f32, complete: bool) -> Building {
        let position = self
            .data
            .config
            .world_layout
            .building_position(kind.id())
            .expect("validated capture building position");
        let (width, height) = match kind {
            BuildingKind::WorkShed => (2, 2),
            BuildingKind::GraveLantern => (1, 1),
            BuildingKind::OssuaryKiln => (2, 1),
        };
        Building {
            kind,
            progress,
            complete,
            position,
            width,
            height,
        }
    }

    pub(super) fn prepare_capture_victory(&mut self) {
        self.session.economy.bones = 160;
        self.session.economy.mana = 80;
        self.session.progress.unlocked_plots = 6;
        for plot in &mut self.session.world.plots {
            plot.status = crate::state::PlotStatus::Dug;
        }
        self.session.world.buildings = vec![
            self.capture_building(BuildingKind::WorkShed, 10.0, true),
            self.capture_building(BuildingKind::GraveLantern, 12.0, true),
        ];
        self.session.economy.corpses.push(Corpse {
            id: 1,
            integrity: 0.9,
            strength: 0.86,
            skill: 0.9,
            magical_residue: 0.95,
            cause_of_death: "old battlefield wound".to_owned(),
            quality: CorpseQuality::Notable,
        });
        for _ in 0..3 {
            let _ = corpses::raise(&mut self.session, &self.data, UndeadKind::Skeleton);
        }
        let _ = corpses::raise(&mut self.session, &self.data, UndeadKind::BruteSkeleton);
        self.session.pressure.suspicion = 44.0;
        self.session.phase = GamePhase::Victory;
    }

    pub(super) fn prepare_capture_research(&mut self) {
        self.session.economy.bones = 120;
        self.session.economy.mana = 60;
        self.session.economy.wood = 70;
        self.session.world.buildings =
            vec![self.capture_building(BuildingKind::WorkShed, 10.0, true)];
        self.session.research.current = Some(Technology::BindingRoutines);
        self.session.research.progress = 2.0;
        self.session.world.selected = Some(Selection::Building(0));
        self.panel = Panel::Research;
    }

    pub(super) fn prepare_capture_orders(&mut self) {
        self.session.economy.bones = 120;
        self.session.economy.mana = 60;
        self.session.economy.wood = 70;
        self.session.research.completed = vec![Technology::BindingRoutines];
        self.session.workforce.workers[0].priority_mode = true;
        self.session.workforce.priorities = [
            JobKind::Dig,
            JobKind::Haul,
            JobKind::Guard,
            JobKind::Build,
            JobKind::Wood,
            JobKind::Refine,
        ]
        .to_vec();
        self.session.world.selected = Some(Selection::Worker(0));
        self.panel = Panel::Orders;
    }

    pub(super) fn prepare_capture_priority_route(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::None;
        self.zone_mode = None;
        self.domain_overlays.routes = true;
        self.session.economy.loose_bones = 8;
        self.session.economy.loose_bones_source = Some(TilePos::new(2, 2));
        self.session.economy.wood = 12;
        self.session.workforce.priorities = [
            JobKind::Wood,
            JobKind::Haul,
            JobKind::Guard,
            JobKind::Dig,
            JobKind::Build,
            JobKind::Refine,
        ]
        .to_vec();
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Work,
            tiles: vec![self.session.world.forest_tiles[0]],
        }];
        if let Some(worker) = self.session.workforce.workers.first_mut() {
            worker.assignment = JobKind::Wood;
            worker.priority_mode = true;
            worker.status = WorkerStatus::Idle;
            worker.position = TilePos::new(2, 2);
        }
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_route_policy(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Zones;
        self.zone_mode = None;
        self.session.world.route_policies.work = RoutePolicy::Nearest;
        self.session.world.route_policies.storage = RoutePolicy::MarkedOnly;
        self.session.world.route_policies.patrol = RoutePolicy::MarkedFirst;
        self.session.world.selected =
            Some(Selection::Ground(self.session.world.stockpile_position()));
    }

    pub(super) fn prepare_capture_route_policy_wait(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.zone_mode = None;
        self.session.world.route_policies.work = RoutePolicy::MarkedOnly;
        self.session
            .world
            .zones
            .retain(|zone| zone.kind != ZoneKind::Work);
        self.session.economy.wood = 12;
        self.session.workforce.priorities = [
            JobKind::Wood,
            JobKind::Haul,
            JobKind::Guard,
            JobKind::Dig,
            JobKind::Build,
            JobKind::Refine,
        ]
        .to_vec();
        if let Some(worker) = self.session.workforce.workers.first_mut() {
            worker.assignment = JobKind::Wood;
            worker.priority_mode = true;
            worker.status = WorkerStatus::Idle;
        }
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_colony(&mut self) {
        self.session.economy.bones = 72;
        self.session.economy.mana = 120;
        self.session.economy.wood = 36;
        self.session.economy.ward_charges = 3;
        self.session.progress.unlocked_plots = 6;
        for plot in &mut self.session.world.plots {
            plot.status = crate::state::PlotStatus::Dug;
        }
        self.session.world.buildings = vec![
            self.capture_building(BuildingKind::WorkShed, 10.0, true),
            self.capture_building(BuildingKind::GraveLantern, 12.0, true),
            self.capture_building(BuildingKind::OssuaryKiln, 14.0, true),
        ];
        self.session.research.completed = vec![
            Technology::BindingRoutines,
            Technology::Gravecraft,
            Technology::OssuaryLogistics,
            Technology::DomainStewardship,
        ];
        for _ in 0..5 {
            let _ = corpses::raise(&mut self.session, &self.data, UndeadKind::Skeleton);
        }
        self.session.world.zones = vec![
            Zone {
                kind: ZoneKind::Work,
                tiles: vec![TilePos::new(2, 2), TilePos::new(0, 3)],
            },
            Zone {
                kind: ZoneKind::Storage,
                tiles: vec![self.session.world.stockpile_position()],
            },
            Zone {
                kind: ZoneKind::Patrol,
                tiles: vec![TilePos::new(7, 1), TilePos::new(8, 3), TilePos::new(8, 5)],
            },
        ];
        self.session.progress.district_ledger.work_cycles = 4;
        self.session.progress.district_ledger.storage_bonus_items = 6;
        self.session.progress.district_ledger.patrol_quieting = 1.75;
        self.session.progress.district_ledger.recent_activity = vec![
            DistrictActivity {
                kind: DistrictActivityKind::PatrolQuieting,
                amount: 0.35,
                elapsed_seconds: 39.0,
            },
            DistrictActivity {
                kind: DistrictActivityKind::StorageBonus,
                amount: 4.0,
                elapsed_seconds: 28.0,
            },
            DistrictActivity {
                kind: DistrictActivityKind::WorkCycle,
                amount: 1.0,
                elapsed_seconds: 15.0,
            },
        ];
        self.session.world.selected = Some(Selection::Ground(TilePos::new(5, 5)));
        self.panel = Panel::Zones;
        self.zone_mode = Some(ZoneKind::Work);
    }

    pub(super) fn prepare_capture_market(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Market;
        self.zone_mode = None;
        self.session.economy.bones = 48;
        self.session.economy.mana = 36;
        self.session.economy.wood = 36;
        self.session.progress.market.offer_index = 0;
        self.session.progress.market.refresh_seconds = 24.0;
        self.session.progress.market.completed_trades = 2;
        self.session.world.selected = Some(Selection::Building(1));
    }

    pub(super) fn prepare_capture_market_contract(&mut self) {
        self.prepare_capture_market();
        self.session.progress.market.contract = Some(MarketContract {
            offer_index: 0,
            remaining_seconds: 51.0,
            bonus_favor: crate::state::MARKET_CONTRACT_BONUS_FAVOR,
        });
    }

    pub(super) fn prepare_capture_market_locked(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::None;
        self.zone_mode = None;
        self.session.world.buildings =
            vec![self.capture_building(BuildingKind::GraveLantern, 4.0, false)];
        self.session.world.selected = Some(Selection::Building(0));
    }

    pub(super) fn prepare_capture_domain(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.zone_mode = None;
        self.domain_overlays.routes = true;
        self.session.pressure.suspicion = 58.0;
        self.session.pressure.stage = SuspicionStage::Questioning;
        self.session.pressure.last_reason =
            "A patrol lantern caught a glimpse of movement.".to_owned();
        self.session.stewardship_policy = StewardshipPolicy::Secure;
        for (index, position) in [
            (0, TilePos::new(5, 5)),
            (1, TilePos::new(5, 4)),
            (2, TilePos::new(4, 5)),
        ] {
            if let Some(worker) = self.session.workforce.workers.get_mut(index) {
                worker.assignment = JobKind::Guard;
                worker.status = WorkerStatus::Hiding;
                worker.position = position;
            }
        }
    }

    pub(super) fn prepare_capture_harvest_domain(&mut self) {
        self.prepare_capture_work_domain();
        self.session.stewardship_policy = StewardshipPolicy::Harvest;
    }

    pub(super) fn prepare_capture_storage_domain(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.zone_mode = None;
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Storage,
            tiles: vec![self.session.world.stockpile_position(), TilePos::new(6, 6)],
        }];
        self.session.economy.loose_bones = 18;
        self.session.economy.loose_bones_source = Some(TilePos::new(4, 2));
        self.session.world.selected =
            Some(Selection::Ground(self.session.world.stockpile_position()));
    }

    pub(super) fn prepare_capture_storage_slots(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.domain_overlays.routes = true;
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Storage,
            tiles: vec![TilePos::new(5, 1), self.session.world.stockpile_position()],
        }];
        for worker in &mut self.session.workforce.workers {
            worker.assignment = JobKind::Refine;
            worker.status = WorkerStatus::Idle;
            worker.carrying = 0;
        }
        for (index, position) in [(0, TilePos::new(2, 2)), (1, TilePos::new(7, 6))] {
            if let Some(worker) = self.session.workforce.workers.get_mut(index) {
                worker.assignment = JobKind::Haul;
                worker.status = WorkerStatus::Carrying;
                worker.carrying = 4;
                worker.position = position;
            }
        }
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_storage_full(&mut self) {
        self.session.economy.bones = self.data.config.storage_capacity;
        let source = self.session.world.plots[0].position;
        self.session
            .economy
            .add_loose(crate::state::ResourceKind::Bones, source, 8);
        self.session.workforce.workers[0].assignment = JobKind::Guard;
        self.session.workforce.workers[0].status = WorkerStatus::Hiding;
        self.session.workforce.workers[0].position = self.session.world.stockpile_position();
        self.session.world.selected =
            Some(Selection::Ground(self.session.world.stockpile_position()));
    }

    pub(super) fn prepare_capture_wood_slots(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.domain_overlays.routes = true;
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Work,
            tiles: self.session.world.forest_tiles[..2].to_vec(),
        }];
        for worker in &mut self.session.workforce.workers {
            worker.assignment = JobKind::Refine;
            worker.status = WorkerStatus::Idle;
            worker.carrying = 0;
        }
        for (index, position) in [(0, TilePos::new(1, 0)), (1, TilePos::new(1, 1))] {
            if let Some(worker) = self.session.workforce.workers.get_mut(index) {
                worker.assignment = JobKind::Wood;
                worker.status = WorkerStatus::Working;
                worker.position = position;
            }
        }
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_patrol_gap(&mut self) {
        self.prepare_capture_domain();
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        for worker in self.session.workforce.workers.iter_mut().skip(1) {
            worker.assignment = JobKind::Dig;
            worker.status = WorkerStatus::Idle;
        }
    }

    pub(super) fn prepare_capture_work_gap(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::None;
        self.zone_mode = None;
        self.session
            .world
            .zones
            .retain(|zone| zone.kind != ZoneKind::Patrol);
        for worker in &mut self.session.workforce.workers {
            worker.assignment = JobKind::Haul;
            worker.status = WorkerStatus::Idle;
        }
        self.session.world.selected = Some(Selection::Ground(TilePos::new(0, 3)));
    }

    pub(super) fn prepare_capture_work_domain(&mut self) {
        self.prepare_capture_work_gap();
        self.panel = Panel::Domain;
    }

    pub(super) fn prepare_capture_work_overlap(&mut self) {
        self.prepare_capture_work_gap();
        self.session.world.zones.push(Zone {
            kind: ZoneKind::Storage,
            tiles: vec![TilePos::new(0, 3)],
        });
    }

    pub(super) fn prepare_capture_work_locked(&mut self) {
        self.prepare_capture_work_gap();
        self.session
            .research
            .completed
            .retain(|technology| *technology != Technology::DomainStewardship);
    }

    pub(super) fn prepare_capture_work_overlap_locked(&mut self) {
        self.prepare_capture_work_overlap();
        self.session
            .research
            .completed
            .retain(|technology| *technology != Technology::DomainStewardship);
    }

    pub(super) fn prepare_capture_route_blocked(&mut self) {
        self.prepare_capture_domain();
        self.panel = Panel::None;
        self.session.world.selected = Some(Selection::Worker(0));
        if let Some(worker) = self.session.workforce.workers.first_mut() {
            worker.position = TilePos::new(2, 2);
            worker.status = WorkerStatus::Idle;
            worker.assignment = JobKind::Guard;
        }
        for y in 0..self.session.world.height as i32 {
            self.session.world.buildings.push(Building {
                kind: BuildingKind::WorkShed,
                progress: 10.0,
                complete: true,
                position: TilePos::new(3, y),
                width: 1,
                height: 1,
            });
        }
    }

    pub(super) fn prepare_capture_route_domain(&mut self) {
        self.prepare_capture_route_blocked();
        self.panel = Panel::Domain;
    }

    pub(super) fn prepare_capture_district_route_gap(&mut self) {
        self.prepare_capture_domain();
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Storage,
            tiles: vec![self.session.world.stockpile_position()],
        }];
        if let Some(worker) = self.session.workforce.workers.first_mut() {
            worker.assignment = JobKind::Haul;
            worker.status = WorkerStatus::Idle;
            worker.position = TilePos::new(2, 2);
        }
        for worker in self.session.workforce.workers.iter_mut().skip(1) {
            worker.assignment = JobKind::Refine;
            worker.status = WorkerStatus::Idle;
        }
        for y in 0..self.session.world.height as i32 {
            self.session.world.buildings.push(Building {
                kind: BuildingKind::WorkShed,
                progress: 10.0,
                complete: true,
                position: TilePos::new(3, y),
                width: 1,
                height: 1,
            });
        }
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_notes(&mut self) {
        self.prepare_capture_domain();
        self.panel = Panel::Feed;
        self.session.progress.production_ledger = ProductionLedger {
            total_cycles: 9,
            ward_cycles: 6,
            hush_ash_cycles: 3,
            wards_sealed: 9,
            suspicion_quieted: 15.0,
        };
        for message in [
            "The western patrol reached its marked post.",
            "Harvest posture now favours loose material.",
            "A ward charge was refined in the ossuary kiln.",
        ] {
            self.session.add_feed(message);
        }
    }
}
