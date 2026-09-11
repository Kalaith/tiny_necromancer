//! Deterministic presentation scenes used by the verification harness.

mod kiln;

use super::Game;
use crate::data::SuspicionStage;
use crate::engine::corpses;
use crate::state::{
    BuildingKind, DistrictActivity, DistrictActivityKind, GamePhase, GameSession, JobKind,
    ProductionOrder, RoutePolicy, Selection, StewardshipPolicy, Technology, UndeadKind,
    WorkerStatus, WorldState, Zone, ZoneKind,
};
use crate::ui::{self, DomainOverlays, Panel};
use macroquad::prelude::*;
use macroquad_toolkit::camera::CameraTransform;
use macroquad_toolkit::grid::TilePos;

impl Game {
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.session = GameSession::new(&self.data.config);
        if scene != "menu" {
            self.session.begin();
        }
        self.notifications.clear();
        self.events.drain().for_each(drop);
        self.tick_accumulator = 0.0;
        self.motions.reset(&self.session);
        self.animation.reset();
        self.camera = CameraTransform::new(Vec2::ZERO, 1.0).expect("valid initial camera");
        self.camera_drag = None;
        self.panel = Panel::None;
        self.placement = None;
        self.zone_mode = None;
        self.domain_overlays = DomainOverlays::default();
        match scene {
            "menu" | "gameplay" | "scrolled" => {}
            "research" => self.prepare_capture_research(),
            "orders" => self.prepare_capture_orders(),
            "priority-route" => self.prepare_capture_priority_route(),
            "route-policy" => self.prepare_capture_route_policy(),
            "route-policy-wait" => self.prepare_capture_route_policy_wait(),
            "colony" => self.prepare_capture_colony(),
            "market" => self.prepare_capture_market(),
            "domain" => self.prepare_capture_domain(),
            "harvest-domain" => self.prepare_capture_harvest_domain(),
            "storage-domain" => self.prepare_capture_storage_domain(),
            "storage-slots" => self.prepare_capture_storage_slots(),
            "storage-full" => self.prepare_capture_storage_full(),
            "wood-slots" => self.prepare_capture_wood_slots(),
            "patrol-gap" => self.prepare_capture_patrol_gap(),
            "work-gap" => self.prepare_capture_work_gap(),
            "work-overlap" => self.prepare_capture_work_overlap(),
            "work-locked" => self.prepare_capture_work_locked(),
            "work-overlap-locked" => self.prepare_capture_work_overlap_locked(),
            "work-domain" => self.prepare_capture_work_domain(),
            "route-blocked" => self.prepare_capture_route_blocked(),
            "route-domain" => self.prepare_capture_route_domain(),
            "district-route-gap" => self.prepare_capture_district_route_gap(),
            "notes" => self.prepare_capture_notes(),
            "production" => self.prepare_capture_production(),
            "kiln-supply" => self.prepare_capture_kiln_supply(),
            "kiln-supply-loose" => self.prepare_capture_kiln_supply_loose(),
            "kiln-supply-inspector" => self.prepare_capture_kiln_supply_inspector(),
            "placement" => {
                self.panel = Panel::Build;
                self.placement = Some(BuildingKind::WorkShed);
            }
            "paused" => self.session.phase = GamePhase::Paused,
            "event" => {
                self.session.pressure.suspicion = self.data.config.suspicion_thresholds[0];
                self.session.pressure.stage = SuspicionStage::Rumour;
                self.session.pressure.active_event = Some("rumour".to_owned());
            }
            "worker-walking" => self.prepare_capture_worker_walking(),
            "worker-haul-planned" => self.prepare_capture_worker_haul_planned(),
            "multi-source" => self.prepare_capture_multi_source(),
            "worker-carrying" => self.prepare_capture_worker_carrying(),
            "worker-working" => self.prepare_capture_worker_working(),
            "necromancer-walking" => self.prepare_capture_necromancer_walking(),
            "necromancer-ritual" => self.prepare_capture_necromancer_ritual(),
            "building" => self.prepare_capture_building(),
            "kiln" => self.prepare_capture_kiln(),
            "victory" => self.prepare_capture_victory(),
            "zoomed" => {
                self.camera.zoom_at(
                    ui::world_grid_rect(),
                    ui::world_grid_rect().center(),
                    1.25,
                    (0.75, 1.5),
                );
                self.camera.pan_screen(vec2(20.0, -12.0));
            }
            other => panic!("Unknown Tiny Necromancer capture scene: {other}"),
        }
        self.motions.reset(&self.session);
    }

    fn prepare_capture_victory(&mut self) {
        self.session.economy.bones = 160;
        self.session.economy.mana = 80;
        self.session.progress.unlocked_plots = 6;
        for plot in &mut self.session.world.plots {
            plot.status = crate::state::PlotStatus::Dug;
        }
        self.session.world.buildings = vec![
            crate::state::Building {
                kind: BuildingKind::WorkShed,
                progress: 10.0,
                complete: true,
                position: crate::state::default_building_position_for_kind(BuildingKind::WorkShed),
                width: 2,
                height: 2,
            },
            crate::state::Building {
                kind: BuildingKind::GraveLantern,
                progress: 12.0,
                complete: true,
                position: crate::state::default_building_position_for_kind(
                    BuildingKind::GraveLantern,
                ),
                width: 1,
                height: 1,
            },
        ];
        self.session.economy.corpses.push(crate::state::Corpse {
            id: 1,
            integrity: 0.9,
            strength: 0.86,
            skill: 0.9,
            magical_residue: 0.95,
            cause_of_death: "old battlefield wound".to_owned(),
            quality: crate::state::CorpseQuality::Notable,
        });
        for _ in 0..3 {
            let _ = corpses::raise(&mut self.session, &self.data, UndeadKind::Skeleton);
        }
        let _ = corpses::raise(&mut self.session, &self.data, UndeadKind::BruteSkeleton);
        self.session.pressure.suspicion = 44.0;
        self.session.phase = GamePhase::Victory;
    }

    fn prepare_capture_research(&mut self) {
        self.session.economy.bones = 120;
        self.session.economy.mana = 60;
        self.session.economy.wood = 70;
        self.session.world.buildings = vec![crate::state::Building {
            kind: BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position: crate::state::default_building_position_for_kind(BuildingKind::WorkShed),
            width: 2,
            height: 2,
        }];
        self.session.research.current = Some(Technology::BindingRoutines);
        self.session.research.progress = 2.0;
        self.session.world.selected = Some(Selection::Building(0));
        self.panel = Panel::Research;
    }

    fn prepare_capture_orders(&mut self) {
        self.session.economy.bones = 120;
        self.session.economy.mana = 60;
        self.session.economy.wood = 70;
        self.session.research.completed = vec![Technology::BindingRoutines];
        self.session.workforce.workers[0].priority_mode = true;
        self.session.workforce.priorities = vec![
            JobKind::Dig,
            JobKind::Haul,
            JobKind::Guard,
            JobKind::Build,
            JobKind::Wood,
            JobKind::Refine,
        ];
        self.session.world.selected = Some(Selection::Worker(0));
        self.panel = Panel::Orders;
    }

    fn prepare_capture_priority_route(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::None;
        self.zone_mode = None;
        self.domain_overlays.routes = true;
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.economy.loose_bones = 8;
        self.session.economy.loose_bones_source = Some(TilePos::new(2, 2));
        self.session.economy.wood = 12;
        self.session.workforce.priorities = vec![
            JobKind::Wood,
            JobKind::Haul,
            JobKind::Guard,
            JobKind::Dig,
            JobKind::Build,
            JobKind::Refine,
        ];
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
        for y in 0..self.session.world.height as i32 {
            self.session.world.buildings.push(crate::state::Building {
                kind: BuildingKind::WorkShed,
                progress: 10.0,
                complete: true,
                position: TilePos::new(1, y),
                width: 1,
                height: 1,
            });
        }
        self.session.world.selected = Some(Selection::Worker(0));
    }

    fn prepare_capture_route_policy(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Zones;
        self.zone_mode = None;
        self.session.world.route_policies.work = RoutePolicy::Nearest;
        self.session.world.route_policies.storage = RoutePolicy::MarkedOnly;
        self.session.world.route_policies.patrol = RoutePolicy::MarkedFirst;
        self.session.world.selected = Some(Selection::Ground(TilePos::new(6, 5)));
    }

    fn prepare_capture_route_policy_wait(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.zone_mode = None;
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.world.route_policies.work = RoutePolicy::MarkedOnly;
        self.session
            .world
            .zones
            .retain(|zone| zone.kind != ZoneKind::Work);
        self.session.economy.wood = 12;
        self.session.workforce.priorities = vec![
            JobKind::Wood,
            JobKind::Haul,
            JobKind::Guard,
            JobKind::Dig,
            JobKind::Build,
            JobKind::Refine,
        ];
        if let Some(worker) = self.session.workforce.workers.first_mut() {
            worker.assignment = JobKind::Wood;
            worker.priority_mode = true;
            worker.status = WorkerStatus::Idle;
        }
        self.session.world.selected = Some(Selection::Worker(0));
    }

    fn prepare_capture_colony(&mut self) {
        self.session.economy.bones = 72;
        self.session.economy.mana = 120;
        self.session.economy.wood = 36;
        self.session.economy.ward_charges = 3;
        self.session.progress.unlocked_plots = 6;
        for plot in &mut self.session.world.plots {
            plot.status = crate::state::PlotStatus::Dug;
        }
        self.session.world.buildings = vec![
            crate::state::Building {
                kind: BuildingKind::WorkShed,
                progress: 10.0,
                complete: true,
                position: TilePos::new(6, 2),
                width: 2,
                height: 2,
            },
            crate::state::Building {
                kind: BuildingKind::GraveLantern,
                progress: 12.0,
                complete: true,
                position: TilePos::new(7, 6),
                width: 1,
                height: 1,
            },
            crate::state::Building {
                kind: BuildingKind::OssuaryKiln,
                progress: 14.0,
                complete: true,
                position: TilePos::new(6, 4),
                width: 2,
                height: 1,
            },
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
                tiles: vec![TilePos::new(6, 5)],
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

    fn prepare_capture_market(&mut self) {
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

    fn prepare_capture_domain(&mut self) {
        self.prepare_capture_colony();
        self.session.economy.bones = 72;
        self.session.economy.wood = 36;
        self.panel = Panel::Domain;
        self.zone_mode = None;
        self.domain_overlays.routes = true;
        self.session.pressure.suspicion = 58.0;
        self.session.pressure.stage = SuspicionStage::Questioning;
        self.session.pressure.last_reason =
            "A patrol lantern caught a glimpse of movement.".to_owned();
        self.session.stewardship_policy = StewardshipPolicy::Secure;
        if let Some(worker) = self.session.workforce.workers.first_mut() {
            worker.assignment = JobKind::Guard;
            worker.status = WorkerStatus::Hiding;
            worker.position = TilePos::new(5, 5);
        }
        for (index, position) in [(1, TilePos::new(5, 4)), (2, TilePos::new(4, 5))] {
            if let Some(worker) = self.session.workforce.workers.get_mut(index) {
                worker.assignment = JobKind::Guard;
                worker.status = WorkerStatus::Hiding;
                worker.position = position;
            }
        }
    }

    fn prepare_capture_harvest_domain(&mut self) {
        self.prepare_capture_work_domain();
        self.session.stewardship_policy = StewardshipPolicy::Harvest;
    }

    fn prepare_capture_storage_domain(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.zone_mode = None;
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.pressure.last_reason =
            "The marked storage district is waiting for more hands.".to_owned();
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Storage,
            tiles: vec![TilePos::new(6, 5), TilePos::new(6, 6)],
        }];
        self.session.economy.loose_bones = 18;
        self.session.economy.loose_bones_source = Some(TilePos::new(4, 2));
        self.session.stewardship_policy = StewardshipPolicy::Balanced;
        for worker in &mut self.session.workforce.workers {
            worker.assignment = JobKind::Dig;
            worker.status = WorkerStatus::Idle;
        }
        self.session.world.selected = Some(Selection::Ground(TilePos::new(6, 5)));
    }

    fn prepare_capture_storage_slots(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.zone_mode = None;
        self.domain_overlays.routes = true;
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Storage,
            tiles: vec![TilePos::new(5, 1), TilePos::new(6, 5)],
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

    fn prepare_capture_storage_full(&mut self) {
        self.session.economy.bones = self.data.config.storage_capacity;
        self.session.economy.wood = 0;
        let source = self.session.world.plots[0].position;
        self.session
            .economy
            .add_loose(crate::state::ResourceKind::Bones, source, 8);
        self.session.workforce.workers[0].assignment = JobKind::Guard;
        self.session.workforce.workers[0].status = WorkerStatus::Hiding;
        self.session.workforce.workers[0].position = WorldState::stockpile_position();
        self.session.world.selected = Some(Selection::Ground(WorldState::stockpile_position()));
    }

    fn prepare_capture_wood_slots(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::Domain;
        self.zone_mode = None;
        self.domain_overlays.routes = true;
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Work,
            tiles: vec![
                self.session.world.forest_tiles[0],
                self.session.world.forest_tiles[1],
            ],
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

    fn prepare_capture_patrol_gap(&mut self) {
        self.prepare_capture_domain();
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.pressure.last_reason =
            "The marked patrol network needs more hands.".to_owned();
        for worker in self.session.workforce.workers.iter_mut().skip(1) {
            worker.assignment = JobKind::Dig;
            worker.status = WorkerStatus::Idle;
        }
    }

    fn prepare_capture_work_gap(&mut self) {
        self.prepare_capture_colony();
        self.panel = Panel::None;
        self.zone_mode = None;
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.pressure.last_reason =
            "A marked work district is waiting for its first operator.".to_owned();
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

    fn prepare_capture_work_domain(&mut self) {
        self.prepare_capture_work_gap();
        self.panel = Panel::Domain;
    }

    fn prepare_capture_work_overlap(&mut self) {
        self.prepare_capture_work_gap();
        self.session.world.zones.push(Zone {
            kind: ZoneKind::Storage,
            tiles: vec![TilePos::new(0, 3)],
        });
    }

    fn prepare_capture_work_locked(&mut self) {
        self.prepare_capture_work_gap();
        self.session
            .research
            .completed
            .retain(|technology| *technology != Technology::DomainStewardship);
    }

    fn prepare_capture_work_overlap_locked(&mut self) {
        self.prepare_capture_work_overlap();
        self.session
            .research
            .completed
            .retain(|technology| *technology != Technology::DomainStewardship);
    }

    fn prepare_capture_route_blocked(&mut self) {
        self.prepare_capture_domain();
        self.panel = Panel::None;
        self.session.world.selected = Some(Selection::Worker(0));
        if let Some(worker) = self.session.workforce.workers.first_mut() {
            worker.position = TilePos::new(2, 2);
            worker.status = WorkerStatus::Idle;
            worker.assignment = JobKind::Guard;
        }
        for y in 0..self.session.world.height as i32 {
            self.session.world.buildings.push(crate::state::Building {
                kind: BuildingKind::WorkShed,
                progress: 10.0,
                complete: true,
                position: TilePos::new(3, y),
                width: 1,
                height: 1,
            });
        }
    }

    fn prepare_capture_route_domain(&mut self) {
        self.prepare_capture_route_blocked();
        self.panel = Panel::Domain;
    }

    fn prepare_capture_district_route_gap(&mut self) {
        self.prepare_capture_domain();
        self.panel = Panel::Domain;
        self.session.pressure.suspicion = 0.0;
        self.session.pressure.stage = SuspicionStage::Calm;
        self.session.world.zones = vec![Zone {
            kind: ZoneKind::Storage,
            tiles: vec![TilePos::new(6, 5)],
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
            self.session.world.buildings.push(crate::state::Building {
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

    fn prepare_capture_notes(&mut self) {
        self.prepare_capture_domain();
        self.panel = Panel::Feed;
        for message in [
            "The western patrol reached its marked post.",
            "Harvest posture now favours loose material.",
            "A ward charge was refined in the ossuary kiln.",
        ] {
            self.session.add_feed(message);
        }
    }

    fn prepare_capture_worker_walking(&mut self) {
        let worker = &mut self.session.workforce.workers[0];
        worker.position = TilePos::new(7, 6);
        worker.assignment = JobKind::Dig;
        worker.status = WorkerStatus::Walking;
        self.session.world.selected_plot = Some(0);
        self.session.world.selected = Some(Selection::Worker(0));
    }

    fn prepare_capture_worker_carrying(&mut self) {
        let source = self.session.world.plots[0].position;
        self.session.economy.loose_bones = 4;
        self.session.economy.loose_bones_source = Some(source);
        let worker = &mut self.session.workforce.workers[0];
        worker.position = source;
        worker.assignment = JobKind::Haul;
        worker.status = WorkerStatus::Carrying;
        worker.carrying = 8;
        worker.carrying_resource = Some(crate::state::ResourceKind::Bones);
        worker.haul_plan = Some(crate::state::HaulPlan {
            resource: crate::state::ResourceKind::Bones,
            source,
            destination: crate::state::WorldState::stockpile_position(),
            storage_policy: RoutePolicy::MarkedFirst,
            destination_kind: crate::state::HaulDestination::Storage,
        });
        self.session.world.selected = Some(Selection::Worker(0));
    }

    fn prepare_capture_worker_haul_planned(&mut self) {
        let source = self.session.world.plots[0].position;
        let destination = crate::state::WorldState::stockpile_position();
        self.session.economy.loose_bones = 8;
        self.session.economy.loose_bones_source = Some(source);
        let worker = &mut self.session.workforce.workers[0];
        worker.position = destination;
        worker.assignment = JobKind::Haul;
        worker.status = WorkerStatus::Walking;
        worker.haul_plan = Some(crate::state::HaulPlan {
            resource: crate::state::ResourceKind::Bones,
            source,
            destination,
            storage_policy: RoutePolicy::MarkedFirst,
            destination_kind: crate::state::HaulDestination::Storage,
        });
        self.session.world.selected = Some(Selection::Worker(0));
    }

    fn prepare_capture_multi_source(&mut self) {
        let near_source = self.session.world.plots[0].position;
        let far_source = TilePos::new(4, 6);
        self.session
            .economy
            .add_loose(crate::state::ResourceKind::Bones, near_source, 8);
        self.session
            .economy
            .add_loose(crate::state::ResourceKind::Bones, far_source, 8);
        let worker = &mut self.session.workforce.workers[0];
        worker.position = TilePos::new(4, 4);
        worker.assignment = JobKind::Guard;
        worker.status = WorkerStatus::Hiding;
        self.session.world.selected = Some(Selection::Worker(0));
    }

    fn prepare_capture_worker_working(&mut self) {
        let plot = &mut self.session.world.plots[0];
        plot.status = crate::state::PlotStatus::Digging;
        plot.progress = 2.5;
        let position = plot.position;
        let worker = &mut self.session.workforce.workers[0];
        worker.position = position;
        worker.assignment = JobKind::Dig;
        worker.status = WorkerStatus::Working;
        worker.target_plot = Some(0);
        self.session.world.selected = Some(Selection::Worker(0));
    }

    fn prepare_capture_necromancer_walking(&mut self) {
        self.session.world.necromancer_destination = Some(TilePos::new(2, 0));
        self.session.world.selected = Some(Selection::Necromancer);
    }

    fn prepare_capture_necromancer_ritual(&mut self) {
        self.session.world.selected = Some(Selection::Necromancer);
    }

    fn prepare_capture_building(&mut self) {
        self.session.economy.bones = 80;
        self.session.economy.wood = 80;
        self.session.world.buildings = vec![crate::state::Building {
            kind: BuildingKind::WorkShed,
            progress: 2.5,
            complete: false,
            position: TilePos::new(6, 2),
            width: 2,
            height: 2,
        }];
        let worker = &mut self.session.workforce.workers[0];
        worker.position = TilePos::new(5, 2);
        worker.assignment = JobKind::Build;
        worker.status = WorkerStatus::Working;
        self.session.world.selected = Some(Selection::Building(0));
    }

    fn prepare_capture_kiln(&mut self) {
        self.session.economy.bones = 48;
        self.session.economy.wood = 36;
        self.session.research.completed = vec![
            Technology::BindingRoutines,
            Technology::Gravecraft,
            Technology::OssuaryLogistics,
        ];
        self.session.world.buildings = vec![crate::state::Building {
            kind: BuildingKind::OssuaryKiln,
            progress: 14.0,
            complete: true,
            position: TilePos::new(6, 4),
            width: 2,
            height: 1,
        }];
        self.session.progress.production = Some(ProductionOrder {
            building: BuildingKind::OssuaryKiln,
            progress: 3.0,
            bones_remaining: 0,
            wood_remaining: 0,
        });
        let worker = &mut self.session.workforce.workers[0];
        worker.position = TilePos::new(5, 4);
        worker.assignment = JobKind::Refine;
        worker.status = WorkerStatus::Working;
        self.session.world.selected = Some(Selection::Building(0));
    }
}
