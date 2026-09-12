//! Capture scenes for actor movement, construction, and the kiln.

use super::Game;
use crate::state::{
    Building, BuildingKind, HaulDestination, HaulPlan, JobKind, ProductionOrder,
    ProductionRecipeKind, RoutePolicy, Selection, Technology, WorkerStatus,
};
use macroquad_toolkit::grid::TilePos;

impl Game {
    pub(super) fn prepare_capture_worker_walking(&mut self) {
        let worker = &mut self.session.workforce.workers[0];
        worker.position = TilePos::new(7, 6);
        worker.assignment = JobKind::Dig;
        worker.status = WorkerStatus::Walking;
        self.session.world.selected_plot = Some(0);
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_worker_carrying(&mut self) {
        let source = self.session.world.plots[0].position;
        self.session.economy.loose_bones = 4;
        self.session.economy.loose_bones_source = Some(source);
        let destination = self.session.world.stockpile_position();
        let worker = &mut self.session.workforce.workers[0];
        worker.position = source;
        worker.assignment = JobKind::Haul;
        worker.status = WorkerStatus::Carrying;
        worker.carrying = 8;
        worker.carrying_resource = Some(crate::state::ResourceKind::Bones);
        worker.haul_plan = Some(HaulPlan {
            resource: crate::state::ResourceKind::Bones,
            source,
            destination,
            storage_policy: RoutePolicy::MarkedFirst,
            destination_kind: HaulDestination::Storage,
        });
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_worker_haul_planned(&mut self) {
        let source = self.session.world.plots[0].position;
        let destination = self.session.world.stockpile_position();
        self.session.economy.loose_bones = 8;
        self.session.economy.loose_bones_source = Some(source);
        let worker = &mut self.session.workforce.workers[0];
        worker.position = destination;
        worker.assignment = JobKind::Haul;
        worker.status = WorkerStatus::Walking;
        worker.haul_plan = Some(HaulPlan {
            resource: crate::state::ResourceKind::Bones,
            source,
            destination,
            storage_policy: RoutePolicy::MarkedFirst,
            destination_kind: HaulDestination::Storage,
        });
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_multi_source(&mut self) {
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

    pub(super) fn prepare_capture_worker_working(&mut self) {
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

    pub(super) fn prepare_capture_necromancer_walking(&mut self) {
        self.session.world.necromancer_destination = Some(TilePos::new(2, 0));
        self.session.world.selected = Some(Selection::Necromancer);
    }

    pub(super) fn prepare_capture_necromancer_ritual(&mut self) {
        self.session.world.selected = Some(Selection::Necromancer);
    }

    pub(super) fn prepare_capture_building(&mut self) {
        self.session.economy.bones = 80;
        self.session.economy.wood = 80;
        let position = self
            .data
            .config
            .world_layout
            .building_position(BuildingKind::WorkShed.id())
            .expect("validated shed position");
        self.session.world.buildings = vec![Building {
            kind: BuildingKind::WorkShed,
            progress: 2.5,
            complete: false,
            position,
            width: 2,
            height: 2,
        }];
        let worker = &mut self.session.workforce.workers[0];
        worker.position = TilePos::new(5, 2);
        worker.assignment = JobKind::Build;
        worker.status = WorkerStatus::Working;
        self.session.world.selected = Some(Selection::Building(0));
    }

    pub(super) fn prepare_capture_kiln(&mut self) {
        self.session.economy.bones = 48;
        self.session.economy.wood = 36;
        self.session.research.completed = vec![
            Technology::BindingRoutines,
            Technology::Gravecraft,
            Technology::OssuaryLogistics,
        ];
        let position = self
            .data
            .config
            .world_layout
            .building_position(BuildingKind::OssuaryKiln.id())
            .expect("validated kiln position");
        self.session.world.buildings = vec![Building {
            kind: BuildingKind::OssuaryKiln,
            progress: 14.0,
            complete: true,
            position,
            width: 2,
            height: 1,
        }];
        self.session.progress.production = Some(ProductionOrder {
            building: BuildingKind::OssuaryKiln,
            progress: 3.0,
            recipe: ProductionRecipeKind::WardCharge,
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
