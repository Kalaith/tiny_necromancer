//! Runtime orchestration: input intents, simulation ticks, persistence, and feedback.

use crate::data::GameData;
use crate::engine::{self, corpses, jobs, movement, progression, suspicion};
use crate::state::{
    BuildingKind, GamePhase, GameSession, SaveData, Selection, Technology, Zone, ZoneKind,
};
use crate::ui::animation::AnimationClock;
use crate::ui::{self, CameraZoom, DomainOverlays, Panel, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::camera::{CameraBounds, CameraBoundsPolicy, CameraTransform};
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::input::TouchGesture;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame, InputState};
use macroquad_toolkit::ui::VirtualUi;

mod capture;

pub struct Game {
    data: GameData,
    session: GameSession,
    assets: AssetManager,
    notifications: NotificationManager,
    camera: CameraTransform,
    camera_drag: Option<Vec2>,
    touch_gesture: TouchGesture,
    touch_camera_owned: bool,
    touch_claimed: bool,
    events: EventBus<UiAction>,
    save_exists: bool,
    tick_accumulator: f32,
    motions: movement::MotionState,
    animation: AnimationClock,
    panel: Panel,
    placement: Option<BuildingKind>,
    zone_mode: Option<ZoneKind>,
    domain_overlays: DomainOverlays,
}

impl Game {
    pub async fn new() -> Self {
        let data = GameData::load()
            .unwrap_or_else(|error| panic!("Tiny Necromancer data failed validation: {error}"));
        let mut assets = AssetManager::new();
        let placeholder = Image::gen_image_color(16, 16, Color::new(0.22, 0.12, 0.28, 1.0));
        assets.set_placeholder_texture_direct(Texture2D::from_image(&placeholder));
        let loaded_assets = assets.load_texture_configs(&data.texture_manifest).await;
        let mut notifications = NotificationManager::new();
        notifications.info(format!(
            "Cemetery ready; {loaded_assets} authored textures loaded"
        ));
        let session = GameSession::new(&data.config);
        let motions = movement::MotionState::new(&session);
        let camera = CameraTransform::new(Vec2::ZERO, 1.0).expect("valid initial camera");
        let mut game = Self {
            data,
            session,
            assets,
            notifications,
            camera,
            camera_drag: None,
            touch_gesture: TouchGesture::new(),
            touch_camera_owned: false,
            touch_claimed: false,
            events: EventBus::new(),
            save_exists: false,
            tick_accumulator: 0.0,
            motions,
            animation: AnimationClock::default(),
            panel: Panel::None,
            placement: None,
            zone_mode: None,
            domain_overlays: DomainOverlays::default(),
        };
        game.refresh_save_state();
        game
    }

    pub fn update(&mut self, dt: f32) {
        let frame_dt = dt.min(0.1);
        let input = InputState::capture();
        if input.escape_pressed {
            if self.placement.is_some() || self.zone_mode.is_some() {
                self.events.push(UiAction::CancelPlacement);
            } else if self.panel != Panel::None {
                self.events.push(UiAction::TogglePanel(Panel::None));
            } else {
                self.events.push(UiAction::TogglePause);
            }
        }
        if is_key_pressed(KeyCode::S) {
            self.events.push(UiAction::Save);
        }
        if is_key_pressed(KeyCode::L) {
            self.events.push(UiAction::Load);
        }
        self.update_camera();
        for action in self.events.drain().collect::<Vec<_>>() {
            self.apply_action(action);
        }
        if self.session.phase == GamePhase::Playing && self.session.pressure.active_event.is_none()
        {
            self.tick_accumulator += frame_dt;
            while self.tick_accumulator >= self.data.config.tick_seconds {
                self.tick_accumulator -= self.data.config.tick_seconds;
                let report = engine::simulate_tick(
                    &mut self.session,
                    &self.data,
                    self.data.config.tick_seconds,
                );
                for message in report.messages {
                    self.notifications.info(message);
                }
                if report.became_victorious {
                    self.notifications
                        .success("The tiny operation has reached its finish line.");
                }
            }
        }
        let frozen =
            self.session.phase == GamePhase::Paused || self.session.pressure.active_event.is_some();
        self.motions.update(&self.session, frame_dt, frozen);
        self.animation.update(frame_dt, frozen);
        self.notifications.update(frame_dt);
    }

    fn update_camera(&mut self) {
        let layout = ui::UiLayout::current(self.panel);
        let viewport = if layout.compact {
            VirtualUi::responsive()
        } else {
            VirtualUi::new(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT)
        };
        let mouse = viewport.mouse_position();
        let rect = layout.world_rect;
        let touch_frame = self.touch_gesture.update();
        self.touch_claimed = false;
        if layout.compact {
            if touch_frame.active {
                if !self.touch_camera_owned && touch_frame.claimed {
                    self.touch_camera_owned = rect.contains(touch_frame.center);
                }
                if self.touch_camera_owned && touch_frame.claimed {
                    self.camera.apply_gesture(rect, &touch_frame, (0.75, 1.5));
                }
                self.touch_claimed = touch_frame.claimed;
            } else {
                self.touch_claimed = touch_frame.claimed;
                self.touch_camera_owned = false;
            }
        } else {
            self.touch_camera_owned = false;
        }
        if rect.contains(mouse) {
            if is_mouse_button_pressed(MouseButton::Right) {
                self.camera_drag = Some(mouse);
            }
            if is_mouse_button_down(MouseButton::Right) {
                if let Some(last) = self.camera_drag.replace(mouse) {
                    self.camera.pan_screen(mouse - last);
                }
            } else {
                self.camera_drag = None;
            }
            let wheel = mouse_wheel().1;
            if wheel != 0.0 {
                self.camera
                    .zoom_at(rect, mouse, 1.1_f32.powf(wheel), (0.75, 1.5));
            }
        } else {
            self.camera_drag = None;
        }
        self.camera.constrain(
            rect,
            CameraBounds::new(vec2(-80.0, -60.0), vec2(80.0, 60.0)),
            CameraBoundsPolicy::TargetInside,
        );
    }

    pub fn draw(&mut self) {
        clear_background(dark::BACKGROUND);
        let compact = ui::UiLayout::current(self.panel).compact;
        let virtual_ui = if compact {
            let virtual_ui = VirtualUi::responsive();
            virtual_ui.begin();
            virtual_ui
        } else {
            begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT)
        };
        let layout = ui::UiLayout::for_dimensions(
            virtual_ui.logical_width,
            virtual_ui.logical_height,
            self.panel,
        );
        let ctx = UiContext {
            data: &self.data,
            session: &self.session,
            save_exists: self.save_exists,
            camera: self.camera,
            ui: &virtual_ui,
            sprites: self.assets.get_texture("cemetery_sprites"),
            title_background: self.assets.get_texture("title_background"),
            motions: &self.motions,
            animation_time: self.animation.elapsed(),
            panel: self.panel,
            placement: self.placement,
            zone_mode: self.zone_mode,
            domain_overlays: self.domain_overlays,
            layout,
            touch_claimed: self.touch_claimed,
        };
        for action in ui::draw_game_ui(ctx) {
            self.events.push(action);
        }
        end_virtual_ui_frame();
        self.notifications
            .draw_with_config(&NotificationRenderConfig {
                anchor: NotificationAnchor::TopRight,
                ..Default::default()
            });
        let _ = self.assets.len();
    }

    fn apply_action(&mut self, action: UiAction) {
        match action {
            UiAction::NewGame => {
                self.session = GameSession::new(&self.data.config);
                self.session.begin();
                self.tick_accumulator = 0.0;
                self.motions.reset(&self.session);
                self.animation.reset();
                self.panel = Panel::None;
                self.placement = None;
                self.zone_mode = None;
                self.domain_overlays = DomainOverlays::default();
                self.notifications
                    .info("A fresh graveyard is ready. Start with Dig Graves.");
            }
            UiAction::TogglePause => match self.session.phase {
                GamePhase::Playing => self.session.phase = GamePhase::Paused,
                GamePhase::Paused => self.session.phase = GamePhase::Playing,
                _ => {}
            },
            UiAction::Save => self.save_game(),
            UiAction::Load => self.load_game(),
            UiAction::SelectWorker(index) => {
                jobs::select_worker(&mut self.session, index);
                if index < self.session.workforce.workers.len() {
                    self.session.world.selected = Some(Selection::Worker(index));
                }
            }
            UiAction::SelectPlot(tile) => {
                if let Some(plot) = self.session.world.plots.iter().find(|plot| {
                    plot.position == tile && plot.status != crate::state::PlotStatus::Locked
                }) {
                    self.session.world.selected_plot = Some(plot.id);
                    self.session.world.selected = Some(Selection::Grave(plot.id));
                    self.notifications.info(format!(
                        "Selected plot {} ({:?})",
                        plot.id + 1,
                        plot.status
                    ));
                }
            }
            UiAction::SelectTile(tile) => {
                if let Some((index, _)) =
                    self.session
                        .workforce
                        .workers
                        .iter()
                        .enumerate()
                        .find(|(_, worker)| {
                            worker.position == tile
                                || self.motions.worker_occupies_tile(worker.id, tile)
                        })
                {
                    self.session.workforce.selected_worker = index;
                    self.session.world.selected = Some(Selection::Worker(index));
                } else if self.session.world.necromancer_position == tile
                    || self.motions.necromancer_occupies_tile(tile)
                {
                    self.session.world.selected = Some(Selection::Necromancer);
                } else if let Some((index, _)) = self
                    .session
                    .world
                    .buildings
                    .iter()
                    .enumerate()
                    .find(|(_, building)| {
                        tile.x >= building.position.x
                            && tile.x < building.position.x + building.width
                            && tile.y >= building.position.y
                            && tile.y < building.position.y + building.height
                    })
                {
                    self.session.world.selected = Some(Selection::Building(index));
                } else if let Some(plot) = self.session.world.plots.iter().find(|plot| {
                    plot.position == tile && plot.status != crate::state::PlotStatus::Locked
                }) {
                    self.session.world.selected_plot = Some(plot.id);
                    self.session.world.selected = Some(Selection::Grave(plot.id));
                } else {
                    self.session.world.selected = Some(Selection::Ground(tile));
                }
            }
            UiAction::SelectNecromancer => {
                self.session.world.selected = Some(Selection::Necromancer);
            }
            UiAction::MoveNecromancer(tile) => {
                let current = self.session.world.necromancer_position;
                match movement::request_necromancer_destination(&mut self.session, tile) {
                    Ok(()) => {
                        self.session.world.selected = Some(Selection::Necromancer);
                        if tile == current {
                            self.session
                                .add_feed("The necromancer holds position; movement cancelled.");
                            self.notifications.info("Necromancer movement cancelled.");
                        } else {
                            self.session.add_feed(format!(
                                "The necromancer walks toward {}, {}.",
                                tile.x + 1,
                                tile.y + 1
                            ));
                            self.notifications
                                .info("Destination marked; the necromancer is walking.");
                        }
                    }
                    Err(error) => self.notifications.warning(error),
                }
            }
            UiAction::AssignJob(job) => {
                let result = jobs::assign_job(&mut self.session, job);
                self.notify_result(result);
            }
            UiAction::ToggleAutomation => {
                let result = if self
                    .session
                    .research
                    .is_unlocked(Technology::BindingRoutines)
                {
                    jobs::toggle_automation(&mut self.session)
                } else {
                    Err("Study Binding Routines before repeating worker priorities.".to_owned())
                };
                self.notify_result(result);
            }
            UiAction::Raise(kind) => {
                let result = corpses::raise(&mut self.session, &self.data, kind);
                self.notify_result(result);
            }
            UiAction::QueueBuilding(kind) => {
                let result = progression::queue_building(&mut self.session, &self.data, kind);
                self.notify_result(result);
            }
            UiAction::BeginPlacement(kind) => {
                self.placement = Some(kind);
                self.panel = Panel::None;
            }
            UiAction::PlaceBuilding(tile) => {
                let Some(kind) = self.placement else {
                    return;
                };
                let result = if !self.session.research.is_unlocked(Technology::Gravecraft)
                    && tile != crate::state::default_building_position_for_kind(kind)
                {
                    Err("Study Gravecraft before placing structures away from the restored footprints.".to_owned())
                } else {
                    progression::queue_building_at(&mut self.session, &self.data, kind, tile)
                };
                if result.is_ok() {
                    self.placement = None;
                    self.session.world.selected =
                        Some(Selection::Building(self.session.world.buildings.len() - 1));
                }
                self.notify_result(result);
            }
            UiAction::CancelPlacement => {
                self.placement = None;
                self.zone_mode = None;
            }
            UiAction::UnlockPlot => {
                let result = progression::unlock_plot(&mut self.session, &self.data);
                self.notify_result(result);
            }
            UiAction::ResolveEvent(choice) => {
                let result = suspicion::resolve_event(&mut self.session, &self.data, &choice);
                self.notify_result(result);
            }
            UiAction::ZoomCamera(factor) => {
                let layout = ui::UiLayout::current(self.panel);
                let factor = match factor {
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
            UiAction::CenterCamera => {
                self.camera = CameraTransform::new(Vec2::ZERO, self.camera.zoom())
                    .expect("valid camera reset");
            }
            UiAction::StartResearch(technology) => {
                let result = progression::start_research(&mut self.session, technology);
                self.notify_result(result);
            }
            UiAction::StartProduction(kind) => {
                let result = progression::start_production(&mut self.session, &self.data, kind);
                self.notify_result(result);
            }
            UiAction::CancelProduction(kind) => {
                let result = progression::cancel_production(&mut self.session, &self.data, kind);
                match result {
                    Ok(()) => self
                        .notifications
                        .success("Reserved cycle cancelled; materials returned."),
                    Err(error) => self.notifications.warning(error),
                }
            }
            UiAction::UseWardCharge => {
                let result = progression::use_ward_charge(&mut self.session);
                self.notify_result(result);
            }
            UiAction::MovePriority(job, direction) => {
                let result = jobs::move_priority(&mut self.session, job, direction);
                self.notify_result(result);
            }
            UiAction::ToggleDomainOverlay(overlay) => {
                if self
                    .session
                    .research
                    .is_unlocked(Technology::DomainStewardship)
                {
                    self.domain_overlays.toggle(overlay);
                }
            }
            UiAction::CycleStewardshipPolicy => {
                if self.session.phase == GamePhase::Playing
                    && self
                        .session
                        .research
                        .is_unlocked(Technology::DomainStewardship)
                {
                    self.session.stewardship_policy = self.session.stewardship_policy.next();
                    let policy = self.session.stewardship_policy.label();
                    self.session
                        .add_feed(format!("Stewardship policy: {policy}."));
                    self.notifications
                        .info(format!("Automated workers now follow {policy}."));
                }
            }
            UiAction::TogglePanel(panel) => {
                self.panel = if self.panel == panel {
                    Panel::None
                } else {
                    panel
                };
            }
            UiAction::ToggleZone(kind) => {
                if self.session.research.is_unlocked(Technology::Gravecraft) {
                    self.zone_mode = if self.zone_mode == Some(kind) {
                        None
                    } else {
                        Some(kind)
                    };
                }
            }
            UiAction::PaintZone(tile) => {
                let Some(kind) = self.zone_mode else {
                    return;
                };
                if !self.session.world.zone_contains(kind, tile)
                    && !self.session.world.is_zone_tile_allowed(tile)
                {
                    self.notifications
                        .warning("Mark a clearing, grave, or forest tile inside the cemetery.");
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
        }
        suspicion::update_stage(&mut self.session, &self.data);
        progression::check_victory(&mut self.session, &self.data);
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
                self.notifications.success("Saved the cemetery.");
                self.refresh_save_state();
            }
            Err(error) => self.notifications.danger(format!("Save failed: {error}")),
        }
    }

    fn load_game(&mut self) {
        let loaded: Result<SaveData, String> = load_from_slot_with_migration(
            &self.data.config.game_name,
            &self.data.config.save_slot,
            &self.data.config.version,
            |version, value| crate::state::migrate_save_value(version, value, &self.data.config),
        );
        match loaded {
            Ok(save) => {
                self.session = GameSession::from_save(save);
                self.motions.reset(&self.session);
                self.animation.reset();
                self.tick_accumulator = 0.0;
                self.panel = Panel::None;
                self.placement = None;
                self.zone_mode = None;
                self.camera_drag = None;
                self.domain_overlays = DomainOverlays::default();
                self.notifications.success("Loaded the cemetery.");
                self.refresh_save_state();
            }
            Err(error) => self.notifications.warning(format!("Load failed: {error}")),
        }
    }

    fn refresh_save_state(&mut self) {
        self.save_exists = slot_exists(&self.data.config.game_name, &self.data.config.save_slot);
    }
}
