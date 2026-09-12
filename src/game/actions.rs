//! UI intent dispatch and persistence actions for the running game.

use super::{selection_for_tile, Game};
use crate::engine::{corpses, jobs, movement, progression, suspicion, trade};
use crate::state::{GamePhase, GameSession, Selection, Technology, Zone};
use crate::ui::{self, CameraZoom, DomainOverlay, DomainOverlays, Panel, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::camera::CameraTransform;
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
};

impl Game {
    pub(super) fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::NewGame => self.start_new_game(),
            UiAction::TogglePause => self.toggle_pause(),
            UiAction::Save => self.save_game(),
            UiAction::Load => self.load_game(),
            UiAction::SelectWorker(index) => self.select_worker(index),
            UiAction::SelectTile(tile) => self.select_tile(tile),
            UiAction::SelectNecromancer => self.select_necromancer(),
            UiAction::MoveNecromancer(tile) => self.move_necromancer(tile),
            UiAction::AssignJob(job) => self.assign_job(job),
            UiAction::ToggleAutomation => self.toggle_automation(),
            UiAction::Raise(kind) => self.raise_undead(kind),
            UiAction::BeginPlacement(kind) => self.begin_placement(kind),
            UiAction::PlaceBuilding(tile) => self.place_building(tile),
            UiAction::CancelPlacement => self.cancel_placement(),
            UiAction::UnlockPlot => self.unlock_plot(),
            UiAction::ResolveEvent(choice) => self.resolve_event(&choice),
            UiAction::ZoomCamera(factor) => self.zoom_camera(factor),
            UiAction::CenterCamera => self.center_camera(),
            UiAction::StudyBindings => self.study_bindings(),
            UiAction::StartResearch(technology) => self.start_research(technology),
            UiAction::SelectProductionRecipe(kind, recipe) => {
                self.select_production_recipe(kind, recipe)
            }
            UiAction::StartProduction(kind) => self.start_production(kind),
            UiAction::CancelProduction(kind) => self.cancel_production(kind),
            UiAction::UpgradeBuilding(kind) => self.upgrade_building(kind),
            UiAction::ExecuteTrade => self.execute_trade(),
            UiAction::AcceptMarketContract => self.accept_market_contract(),
            UiAction::UseWardCharge => self.use_ward_charge(),
            UiAction::MovePriority(job, direction) => self.move_priority(job, direction),
            UiAction::ToggleDomainOverlay(overlay) => self.toggle_domain_overlay(overlay),
            UiAction::CycleStewardshipPolicy => self.cycle_stewardship_policy(),
            UiAction::CycleRoutePolicy(kind) => self.cycle_route_policy(kind),
            UiAction::TogglePanel(panel) => self.toggle_panel(panel),
            UiAction::ToggleZone(kind) => self.toggle_zone(kind),
            UiAction::PaintZone(tile) => self.paint_zone(tile),
        }
        suspicion::update_stage(&mut self.session, &self.data);
        progression::check_victory(&mut self.session, &self.data);
    }

    fn start_new_game(&mut self) {
        self.session = GameSession::new(&self.data.config);
        self.session.begin();
        self.tick_accumulator = 0.0;
        self.motions.reset(&self.session);
        self.animation.reset();
        self.panel = Panel::None;
        self.placement = None;
        self.zone_mode = None;
        self.domain_overlays = DomainOverlays::default();
        self.notifications.info(&self.data.text.new_game);
    }

    fn toggle_pause(&mut self) {
        self.session.phase = match self.session.phase {
            GamePhase::Playing => GamePhase::Paused,
            GamePhase::Paused => GamePhase::Playing,
            phase => phase,
        };
    }

    fn select_worker(&mut self, index: usize) {
        jobs::select_worker(&mut self.session, index);
        if index < self.session.workforce.workers.len() {
            self.session.world.selected = Some(Selection::Worker(index));
        }
    }

    fn select_necromancer(&mut self) {
        self.session.world.selected = Some(Selection::Necromancer);
    }

    fn assign_job(&mut self, job: crate::state::JobKind) {
        let result = jobs::assign_job(&mut self.session, job);
        self.notify_result(result);
    }

    fn raise_undead(&mut self, kind: crate::state::UndeadKind) {
        let name = self
            .name_generator
            .take_name(&self.session.workforce.workers);
        let result = corpses::raise_with_name(&mut self.session, &self.data, kind, name.as_deref());
        self.notify_result(result);
    }

    fn begin_placement(&mut self, kind: crate::state::BuildingKind) {
        self.placement = Some(kind);
        self.panel = Panel::None;
    }

    fn cancel_placement(&mut self) {
        self.placement = None;
        self.zone_mode = None;
    }

    fn unlock_plot(&mut self) {
        let result = progression::unlock_plot(&mut self.session, &self.data);
        self.notify_result(result);
    }

    fn resolve_event(&mut self, choice: &str) {
        let result = suspicion::resolve_event(&mut self.session, &self.data, choice);
        self.notify_result(result);
    }

    fn center_camera(&mut self) {
        self.camera =
            CameraTransform::new(Vec2::ZERO, self.camera.zoom()).expect("valid camera reset");
    }

    fn start_research(&mut self, technology: Technology) {
        let result = progression::start_research(&mut self.session, &self.data, technology);
        self.notify_result(result);
    }

    fn study_bindings(&mut self) {
        let result = progression::study_bindings(&mut self.session, &self.data);
        self.notify_result(result);
    }

    fn select_production_recipe(
        &mut self,
        kind: crate::state::BuildingKind,
        recipe: crate::state::ProductionRecipeKind,
    ) {
        let result =
            progression::select_production_recipe(&mut self.session, &self.data, kind, recipe);
        self.notify_result(result);
    }

    fn start_production(&mut self, kind: crate::state::BuildingKind) {
        let result = progression::start_production(&mut self.session, &self.data, kind);
        self.notify_result(result);
    }

    fn cancel_production(&mut self, kind: crate::state::BuildingKind) {
        match progression::cancel_production(&mut self.session, &self.data, kind) {
            Ok(()) => self
                .notifications
                .success(&self.data.text.production_cancelled),
            Err(error) => self.notifications.warning(error),
        }
    }

    fn upgrade_building(&mut self, kind: crate::state::BuildingKind) {
        let result = progression::upgrade_building(&mut self.session, &self.data, kind);
        self.notify_result(result);
    }

    fn accept_market_contract(&mut self) {
        let result = trade::accept_contract(&mut self.session);
        self.notify_result(result);
    }

    fn use_ward_charge(&mut self) {
        let result = progression::use_ward_charge(&mut self.session);
        self.notify_result(result);
    }

    fn move_priority(&mut self, job: crate::state::JobKind, direction: i32) {
        let result = jobs::move_priority(&mut self.session, job, direction);
        self.notify_result(result);
    }

    fn toggle_domain_overlay(&mut self, overlay: DomainOverlay) {
        if self
            .session
            .research
            .is_unlocked(Technology::DomainStewardship)
        {
            self.domain_overlays.toggle(overlay);
        }
    }

    fn toggle_panel(&mut self, panel: Panel) {
        self.panel = if self.panel == panel {
            Panel::None
        } else {
            panel
        };
    }

    fn toggle_zone(&mut self, kind: crate::state::ZoneKind) {
        if self.session.research.is_unlocked(Technology::Gravecraft) {
            self.zone_mode = if self.zone_mode == Some(kind) {
                None
            } else {
                Some(kind)
            };
        }
    }

    fn select_tile(&mut self, tile: macroquad_toolkit::grid::TilePos) {
        let selection = selection_for_tile(&self.session, &self.motions, tile);
        match selection {
            Selection::Worker(index) => self.session.workforce.selected_worker = index,
            Selection::Grave(plot_id) => self.session.world.selected_plot = Some(plot_id),
            Selection::Building(_) | Selection::Ground(_) | Selection::Necromancer => {}
        }
        self.session.world.selected = Some(selection);
    }

    fn move_necromancer(&mut self, tile: macroquad_toolkit::grid::TilePos) {
        if !matches!(
            selection_for_tile(&self.session, &self.motions, tile),
            Selection::Ground(_)
        ) {
            self.select_tile(tile);
            return;
        }
        let current = self.session.world.necromancer_position;
        match movement::request_necromancer_destination(&mut self.session, tile) {
            Ok(()) => {
                self.session.world.selected = Some(Selection::Necromancer);
                if tile == current {
                    self.session
                        .add_feed(&self.data.text.movement_cancelled_feed);
                    self.notifications
                        .info(&self.data.text.movement_cancelled_notification);
                } else {
                    let movement_feed = self
                        .data
                        .text
                        .movement_started_feed
                        .replace("{x}", &(tile.x + 1).to_string())
                        .replace("{y}", &(tile.y + 1).to_string());
                    self.session.add_feed(movement_feed);
                    self.notifications
                        .info(&self.data.text.movement_started_notification);
                }
            }
            Err(error) => self.notifications.warning(error),
        }
    }

    fn toggle_automation(&mut self) {
        let result = if self
            .session
            .research
            .is_unlocked(Technology::BindingRoutines)
        {
            jobs::toggle_automation(&mut self.session)
        } else {
            Err(self.data.text.automation_locked.clone())
        };
        self.notify_result(result);
    }

    fn place_building(&mut self, tile: macroquad_toolkit::grid::TilePos) {
        let Some(kind) = self.placement else {
            return;
        };
        let result = progression::queue_building_at(&mut self.session, &self.data, kind, tile);
        if result.is_ok() {
            self.placement = None;
            self.session.world.selected =
                Some(Selection::Building(self.session.world.buildings.len() - 1));
        }
        self.notify_result(result);
    }

    fn zoom_camera(&mut self, zoom: CameraZoom) {
        let layout = ui::UiLayout::current(self.panel);
        let factor = match zoom {
            CameraZoom::In => 1.15,
            CameraZoom::Out => 1.0 / 1.15,
        };
        self.camera.zoom_at(
            layout.world_rect,
            layout.world_rect.center(),
            factor,
            (0.75, 1.5),
        );
    }

    fn execute_trade(&mut self) {
        let offer = trade::current_offer(&self.session);
        let previous_standing = self.session.progress.market.standing_label();
        let contract_bonus = self
            .session
            .progress
            .market
            .contract
            .as_ref()
            .map(|contract| contract.bonus_favor);
        match trade::execute_trade(&mut self.session, &self.data) {
            Ok(()) => {
                let current_standing = self.session.progress.market.standing_label();
                let receipt = if let Some(bonus) = contract_bonus {
                    format!(
                        "Request fulfilled: {} for {} · +{} favor.",
                        offer.cost,
                        offer.reward,
                        1 + bonus
                    )
                } else {
                    format!("Exchange complete: {} for {}.", offer.cost, offer.reward)
                };
                if current_standing != previous_standing {
                    self.notifications
                        .success(format!("{receipt} Standing: {current_standing}."));
                } else {
                    self.notifications.success(receipt);
                }
            }
            Err(error) => self.notifications.warning(error),
        }
    }

    fn cycle_stewardship_policy(&mut self) {
        if self.session.phase != GamePhase::Playing
            || !self
                .session
                .research
                .is_unlocked(Technology::DomainStewardship)
        {
            return;
        }
        self.session.stewardship_policy = self.session.stewardship_policy.next();
        let policy = self.session.stewardship_policy.label();
        self.session
            .add_feed(format!("Stewardship policy: {policy}."));
        self.notifications
            .info(format!("Automated workers now follow {policy}."));
    }

    fn cycle_route_policy(&mut self, kind: crate::state::ZoneKind) {
        if self.session.phase != GamePhase::Playing
            || !self
                .session
                .research
                .is_unlocked(Technology::DomainStewardship)
        {
            return;
        }
        let policy = self.session.world.route_policies.cycle(kind);
        let policy_description = policy.description();
        self.session.add_feed(format!(
            "{} route policy: {} · {}.",
            kind.label(),
            policy.label(),
            policy_description
        ));
        self.notifications.info(format!(
            "{} routes: {} · {}.",
            kind.label(),
            policy.label(),
            policy_description
        ));
    }

    fn paint_zone(&mut self, tile: macroquad_toolkit::grid::TilePos) {
        let Some(kind) = self.zone_mode else {
            return;
        };
        if !self.session.world.zone_contains(kind, tile)
            && !self.session.world.is_zone_tile_allowed(tile)
        {
            self.notifications
                .warning(&self.data.text.invalid_zone_tile);
            return;
        }
        let mut cleared = false;
        let mut remove_zone = false;
        if let Some(zone) = self
            .session
            .world
            .zones
            .iter_mut()
            .find(|zone| zone.kind == kind)
        {
            cleared = zone.toggle_tile(tile);
            remove_zone = zone.tiles.is_empty();
        } else {
            self.session.world.zones.push(Zone {
                kind,
                tiles: vec![tile],
            });
        }
        if remove_zone {
            self.session.world.zones.retain(|zone| zone.kind != kind);
        }
        self.session.add_feed(format!(
            "{} zone {} at {}, {}.",
            kind.label(),
            if cleared { "cleared" } else { "marked" },
            tile.x + 1,
            tile.y + 1
        ));
    }

    fn notify_result(&mut self, result: Result<(), String>) {
        match result {
            Ok(()) => self.notifications.success("Order accepted."),
            Err(error) => self.notifications.warning(error),
        }
    }

    fn save_game(&mut self) {
        let save = self.session.to_save(&self.data.config.version);
        match save_to_slot_with_version(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &save,
            &self.data.config.version,
        ) {
            Ok(()) => {
                self.notifications.success(&self.data.text.save_success);
                self.refresh_save_state();
            }
            Err(error) => self.notifications.danger(format!("Save failed: {error}")),
        }
    }

    fn load_game(&mut self) {
        let loaded: Result<crate::state::SaveData, String> = load_from_slot_with_migration(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &self.data.config.version,
            |version, value| crate::state::migrate_save_value(version, value, &self.data.config),
        );
        match loaded {
            Ok(save) => {
                self.session = GameSession::from_save(save, &self.data.config);
                self.motions.reset(&self.session);
                self.animation.reset();
                self.tick_accumulator = 0.0;
                self.panel = Panel::None;
                self.placement = None;
                self.zone_mode = None;
                self.camera_drag = None;
                self.domain_overlays = DomainOverlays::default();
                self.notifications.success(&self.data.text.load_success);
                self.refresh_save_state();
            }
            Err(error) => self.notifications.warning(format!("Load failed: {error}")),
        }
    }

    pub(super) fn refresh_save_state(&mut self) {
        self.save_exists = slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
    }
}
