//! Runtime orchestration: input intents, simulation ticks, persistence, and feedback.

use crate::data::GameData;
use crate::engine::{self, movement};
use crate::state::{BuildingKind, GamePhase, GameSession, ZoneKind};
use crate::ui::animation::AnimationClock;
use crate::ui::{self, DomainOverlays, Panel, UiAction, UiContext};
use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::camera::{CameraBounds, CameraBoundsPolicy, CameraTransform};
use macroquad_toolkit::events::EventBus;
use macroquad_toolkit::input::{TouchGesture, TouchGestureFrame};
use macroquad_toolkit::notifications::{
    NotificationAnchor, NotificationManager, NotificationRenderConfig,
};
use macroquad_toolkit::prelude::{begin_virtual_ui_frame, dark, end_virtual_ui_frame, InputState};
use macroquad_toolkit::ui::VirtualUi;

mod actions;
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
        notifications.info(
            data.text
                .assets_loaded
                .replace("{count}", &loaded_assets.to_string()),
        );
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
                    self.notifications.success(&self.data.text.victory);
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
        let compact = ui::UiLayout::current(self.panel).compact;
        let viewport = if compact {
            VirtualUi::responsive()
        } else {
            VirtualUi::new(ui::LOGICAL_WIDTH, ui::LOGICAL_HEIGHT)
        };
        let layout = ui::UiLayout::for_dimensions(
            viewport.logical_width,
            viewport.logical_height,
            self.panel,
        );
        let mouse = viewport.mouse_position();
        let rect = layout.world_rect;
        let touch_frame = self.touch_gesture.update();
        self.touch_claimed = false;
        if layout.compact {
            if touch_frame.active {
                let frame = TouchGestureFrame {
                    pan: touch_frame.pan / viewport.scale,
                    center: viewport.screen_to_ui(touch_frame.center),
                    ..touch_frame
                };
                if !self.touch_camera_owned && frame.claimed {
                    self.touch_camera_owned = rect.contains(frame.center);
                }
                if self.touch_camera_owned && frame.claimed {
                    self.camera.apply_gesture(rect, &frame, (0.75, 1.5));
                }
                self.touch_claimed = frame.claimed;
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
    }
}
