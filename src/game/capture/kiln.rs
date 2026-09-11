//! Kiln-focused presentation scenes for supply and production feedback.

use super::Game;
use crate::state::{
    BuildingKind, HaulDestination, JobKind, ProductionOrder, ProductionRecipeKind, RoutePolicy,
    Selection, Technology, WorkerStatus, WorldState,
};
use macroquad_toolkit::grid::TilePos;

impl Game {
    pub(super) fn prepare_capture_kiln_hush(&mut self) {
        self.prepare_capture_kiln();
        self.session.progress.production = None;
        self.session.progress.production_queue = 0;
        self.session.progress.production_recipe = ProductionRecipeKind::HushAsh;
        self.session.pressure.suspicion = 18.0;
    }

    pub(super) fn prepare_capture_production(&mut self) {
        self.session.economy.bones = 48;
        self.session.economy.mana = 60;
        self.session.economy.wood = 36;
        self.session.economy.ward_charges = 2;
        self.session.research.completed = vec![
            Technology::BindingRoutines,
            Technology::Gravecraft,
            Technology::OssuaryLogistics,
        ];
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
                kind: BuildingKind::OssuaryKiln,
                progress: 14.0,
                complete: true,
                position: TilePos::new(6, 4),
                width: 2,
                height: 1,
            },
        ];
        self.session.progress.production = Some(ProductionOrder {
            building: BuildingKind::OssuaryKiln,
            progress: 4.0,
            recipe: ProductionRecipeKind::WardCharge,
            bones_remaining: 0,
            wood_remaining: 0,
        });
        self.session.progress.production_queue = 1;
        self.session.workforce.workers[0].assignment = JobKind::Guard;
        self.session.workforce.workers[0].status = WorkerStatus::Hiding;
        self.session.workforce.workers[0].position = TilePos::new(5, 4);
        self.session.world.selected = Some(Selection::Building(1));
    }

    pub(super) fn prepare_capture_kiln_supply(&mut self) {
        self.session.research.completed = vec![
            Technology::BindingRoutines,
            Technology::Gravecraft,
            Technology::OssuaryLogistics,
        ];
        self.session.economy.bones = 8;
        self.session.economy.wood = 6;
        let position = TilePos::new(6, 4);
        self.session.world.buildings = vec![crate::state::Building {
            kind: BuildingKind::OssuaryKiln,
            progress: 14.0,
            complete: true,
            position,
            width: 2,
            height: 1,
        }];
        self.session.progress.production = Some(ProductionOrder {
            building: BuildingKind::OssuaryKiln,
            progress: 0.0,
            recipe: ProductionRecipeKind::WardCharge,
            bones_remaining: 12,
            wood_remaining: 6,
        });
        let source = WorldState::stockpile_position();
        let destination = crate::engine::progression::production_destination(&self.session)
            .expect("capture kiln should have a work position");
        let worker = &mut self.session.workforce.workers[0];
        worker.position = source;
        worker.assignment = JobKind::Haul;
        worker.status = WorkerStatus::Walking;
        worker.haul_plan = Some(crate::state::HaulPlan {
            resource: crate::state::ResourceKind::Bones,
            source,
            destination,
            storage_policy: RoutePolicy::MarkedFirst,
            destination_kind: HaulDestination::Kiln,
        });
        self.session.world.selected = Some(Selection::Worker(0));
    }

    pub(super) fn prepare_capture_kiln_supply_inspector(&mut self) {
        self.prepare_capture_kiln_supply();
        self.session.world.selected = Some(Selection::Building(0));
    }

    pub(super) fn prepare_capture_kiln_supply_loose(&mut self) {
        self.prepare_capture_kiln_supply();
        self.session.economy.bones = 0;
        let source = self.session.world.plots[0].position;
        self.session
            .economy
            .add_loose(crate::state::ResourceKind::Bones, source, 8);
        let destination = crate::engine::progression::production_destination(&self.session)
            .expect("capture kiln should have a work position");
        let worker = &mut self.session.workforce.workers[0];
        worker.position = source;
        worker.haul_plan = Some(crate::state::HaulPlan {
            resource: crate::state::ResourceKind::Bones,
            source,
            destination,
            storage_policy: RoutePolicy::MarkedFirst,
            destination_kind: HaulDestination::Kiln,
        });
    }
}
