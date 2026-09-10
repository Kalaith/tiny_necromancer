//! Runtime orchestration: input intents, simulation ticks, persistence, and feedback.

use crate::data::{GameData, SuspicionStage};
use crate::engine::{self, corpses, jobs, movement, progression, suspicion};
use crate::state::{
    BuildingKind, GamePhase, GameSession, JobKind, ProductionOrder, SaveData, Selection,
    Technology, UndeadKind, WorkerStatus, Zone, ZoneKind,
};
use crate::ui::animation::AnimationClock;
use crate::ui::{self, Panel, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::camera::{CameraBounds, CameraBoundsPolicy, CameraTransform};
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::persistence::{
    load_from_slot_with_migration, save_to_slot_with_version, slot_exists,
};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame, InputState};
use macroquad_toolkit::ui::VirtualUi;

pub struct Game {
    data: GameData,
    session: GameSession,
    assets: AssetManager,
    notifications: NotificationManager,
    camera: CameraTransform,
    camera_drag: Option<Vec2>,
    events: EventBus<UiAction>,
    save_exists: bool,
    tick_accumulator: f32,
    motions: movement::MotionState,
    animation: AnimationClock,
    panel: Panel,
    placement: Option<BuildingKind>,
    zone_mode: Option<ZoneKind>,
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
            events: EventBus::new(),
            save_exists: false,
            tick_accumulator: 0.0,
            motions,
            animation: AnimationClock::default(),
            panel: Panel::None,
            placement: None,
            zone_mode: None,
        };
        game.refresh_save_state();
        game
    }

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
        match scene {
            "menu" | "gameplay" | "scrolled" => {}
            "research" => self.prepare_capture_research(),
            "orders" => self.prepare_capture_orders(),
            "colony" => self.prepare_capture_colony(),
            "production" => self.prepare_capture_production(),
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

    fn prepare_capture_colony(&mut self) {
        self.session.economy.bones = 240;
        self.session.economy.mana = 120;
        self.session.economy.wood = 180;
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
                tiles: vec![TilePos::new(7, 1)],
            },
        ];
        self.session.world.selected = Some(Selection::Ground(TilePos::new(5, 5)));
        self.panel = Panel::Zones;
        self.zone_mode = Some(ZoneKind::Work);
    }

    fn prepare_capture_production(&mut self) {
        self.session.economy.bones = 120;
        self.session.economy.mana = 60;
        self.session.economy.wood = 90;
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
        });
        self.session.progress.production_queue = 1;
        self.session.workforce.workers[0].assignment = JobKind::Refine;
        self.session.workforce.workers[0].status = WorkerStatus::Working;
        self.session.workforce.workers[0].position = TilePos::new(5, 4);
        self.session.world.selected = Some(Selection::Building(1));
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
        self.session.economy.bones = 120;
        self.session.economy.wood = 90;
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
        });
        let worker = &mut self.session.workforce.workers[0];
        worker.position = TilePos::new(5, 4);
        worker.assignment = JobKind::Refine;
        worker.status = WorkerStatus::Working;
        self.session.world.selected = Some(Selection::Building(0));
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
        let viewport = VirtualUi::new(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
        let mouse = viewport.mouse_position();
        let rect = ui::world_grid_rect();
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
        let virtual_ui = begin_virtual_ui_frame(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT);
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
            UiAction::StartResearch(technology) => {
                let result = progression::start_research(&mut self.session, technology);
                self.notify_result(result);
            }
            UiAction::StartProduction(kind) => {
                let result = progression::start_production(&mut self.session, &self.data, kind);
                self.notify_result(result);
            }
            UiAction::UseWardCharge => {
                let result = progression::use_ward_charge(&mut self.session);
                self.notify_result(result);
            }
            UiAction::MovePriority(job, direction) => {
                let result = jobs::move_priority(&mut self.session, job, direction);
                self.notify_result(result);
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
