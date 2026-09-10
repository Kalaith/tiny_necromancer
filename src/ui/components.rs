//! Small rendering and coordinate helpers shared by the world-first interface.

use super::{UiAction, UiContext, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::data::SuspicionStage;
use crate::state::{GamePhase, WorkerStatus};
use macroquad::prelude::*;
use macroquad_toolkit::camera::CameraTransform;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{
    button_rect_enabled_styled_ex_at, touch_area, ButtonStyle, ButtonTrigger, Pointer, RectExt,
};

pub(super) fn pause_control_rect() -> Rect {
    Rect::new(1172.0, 19.0, 80.0, 44.0)
}

pub(super) fn placement_cancel_rect() -> Rect {
    Rect::new(990.0, 628.0, 262.0, 52.0)
}

pub(super) fn virtual_button(
    rect: Rect,
    text: &str,
    enabled: bool,
    tone: ButtonTone,
    pointer: Pointer,
) -> bool {
    button_rect_tone_at(rect, text, enabled, tone, pointer.position);
    enabled && pointer.released_on(touch_area(rect))
}

pub(super) fn compact_virtual_button(
    rect: Rect,
    text: &str,
    enabled: bool,
    tone: ButtonTone,
    text_size: f32,
    pointer: Pointer,
) -> bool {
    let style = ButtonStyle::from_tone(tone);
    button_rect_enabled_styled_ex_at(
        rect,
        text,
        enabled,
        &style,
        TextStyle::new(text_size, style.text_color),
        ButtonTrigger::Release,
        pointer.position,
    );
    enabled && pointer.released_on(touch_area(rect))
}

pub(super) fn draw_placement_controls(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    if ctx.placement.is_none() {
        return;
    }
    draw_text_centered_in_box(
        "Tap CANCEL PLACEMENT to return to the cemetery.",
        988.0,
        602.0,
        266.0,
        18.0,
        11.0,
        dark::TEXT_DIM,
    );
    if virtual_button(
        placement_cancel_rect(),
        "Cancel placement",
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::CancelPlacement);
    }
}

pub(super) fn draw_event_modal(
    ctx: &UiContext<'_>,
    event_id: &str,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(event) = ctx.data.events.get(event_id) else {
        return;
    };
    full_screen_overlay(0.58);
    let rect = Rect::new(290.0, 150.0, 700.0, 420.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.075, 0.07, 0.09, 0.99)).with_border(2.0, dark::WARNING),
    );
    draw_text_block(
        "ROAD PRESSURE",
        rect.x + 38.0,
        rect.y + 28.0,
        250.0,
        18.0,
        13.0,
        0.0,
        dark::WARNING,
    );
    draw_text_centered_in_box(
        &event.title,
        rect.x + 30.0,
        rect.y + 50.0,
        rect.w - 60.0,
        34.0,
        27.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        &event.body,
        rect.x + 42.0,
        rect.y + 98.0,
        rect.w - 84.0,
        64.0,
        17.0,
        4.0,
        dark::TEXT,
    );
    for (index, choice) in event.choices.iter().enumerate() {
        let button_rect = Rect::new(
            rect.x + 42.0,
            rect.y + 180.0 + index as f32 * 92.0,
            rect.w - 84.0,
            72.0,
        );
        if virtual_button(
            button_rect,
            &format!("{} — {}", choice.label, choice.description),
            true,
            ButtonTone::Secondary,
            pointer,
        ) {
            actions.push(UiAction::ResolveEvent(choice.id.clone()));
        }
    }
}

pub(super) fn draw_phase_overlay(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    match ctx.session.phase {
        GamePhase::MainMenu => {
            full_screen_overlay(0.34);
            draw_surface(
                Rect::new(360.0, 150.0, 560.0, 420.0),
                &SurfaceStyle::new(Color::new(0.055, 0.055, 0.065, 0.96))
                    .with_border(2.0, dark::ACCENT),
            );
            draw_text_centered_in_box(
                &ctx.data.config.display_name,
                390.0,
                198.0,
                500.0,
                52.0,
                42.0,
                dark::TEXT_BRIGHT,
            );
            draw_text_centered_in_box(
                "A quiet craft of bones, lanterns, and patience.",
                400.0,
                260.0,
                480.0,
                24.0,
                16.0,
                dark::TEXT_DIM,
            );
            if virtual_button(
                Rect::new(500.0, 330.0, 280.0, 52.0),
                "New Game",
                true,
                ButtonTone::Positive,
                pointer,
            ) {
                actions.push(UiAction::NewGame);
            }
            if virtual_button(
                Rect::new(500.0, 394.0, 280.0, 52.0),
                "Continue",
                ctx.save_exists,
                ButtonTone::Primary,
                pointer,
            ) {
                actions.push(UiAction::Load);
            }
            draw_text_centered_in_box(
                "Settings and Quit are available through the window controls.",
                400.0,
                482.0,
                480.0,
                22.0,
                14.0,
                dark::TEXT_DIM,
            );
        }
        GamePhase::Paused => {
            full_screen_overlay(0.44);
            draw_surface(
                Rect::new(420.0, 250.0, 440.0, 240.0),
                &SurfaceStyle::new(Color::new(0.06, 0.06, 0.07, 0.98))
                    .with_border(2.0, dark::ACCENT),
            );
            draw_text_centered_in_box("Paused", 480.0, 292.0, 320.0, 32.0, 28.0, dark::TEXT_BRIGHT);
            draw_text_centered_in_box(
                "The cemetery is holding its breath.",
                480.0,
                334.0,
                320.0,
                22.0,
                15.0,
                dark::TEXT_DIM,
            );
            if virtual_button(
                Rect::new(456.0, 382.0, 128.0, 48.0),
                "Resume",
                true,
                ButtonTone::Positive,
                pointer,
            ) {
                actions.push(UiAction::TogglePause);
            }
            if virtual_button(
                Rect::new(592.0, 382.0, 128.0, 48.0),
                "Save",
                true,
                ButtonTone::Secondary,
                pointer,
            ) {
                actions.push(UiAction::Save);
            }
            if virtual_button(
                Rect::new(728.0, 382.0, 96.0, 48.0),
                "Load",
                ctx.save_exists,
                ButtonTone::Secondary,
                pointer,
            ) {
                actions.push(UiAction::Load);
            }
        }
        GamePhase::Victory => {
            full_screen_overlay(0.38);
            draw_surface(
                Rect::new(360.0, 190.0, 560.0, 330.0),
                &SurfaceStyle::new(Color::new(0.055, 0.065, 0.065, 0.99))
                    .with_border(2.0, dark::POSITIVE),
            );
            draw_text_centered_in_box(
                "Settlement established",
                390.0,
                238.0,
                500.0,
                36.0,
                30.0,
                dark::TEXT_BRIGHT,
            );
            draw_text_block("The cemetery now has workers, shelter, light, and room to grow. The road still has not found you.", 420.0, 302.0, 440.0, 60.0, 17.0, 4.0, dark::TEXT);
            if virtual_button(
                Rect::new(520.0, 410.0, 240.0, 52.0),
                "Play Again",
                true,
                ButtonTone::Positive,
                pointer,
            ) {
                actions.push(UiAction::NewGame);
            }
        }
        GamePhase::Error => {
            full_screen_overlay(0.55);
            draw_surface(
                Rect::new(360.0, 230.0, 560.0, 240.0),
                &SurfaceStyle::new(Color::new(0.15, 0.07, 0.08, 0.99))
                    .with_border(2.0, dark::NEGATIVE),
            );
            draw_text_centered_in_box(
                "The cemetery needs a moment",
                390.0,
                265.0,
                500.0,
                30.0,
                24.0,
                dark::TEXT_BRIGHT,
            );
            draw_text_centered_in_box(
                ctx.session
                    .error_message
                    .as_deref()
                    .unwrap_or("Unknown game error"),
                400.0,
                315.0,
                480.0,
                48.0,
                16.0,
                dark::TEXT,
            );
            if virtual_button(
                Rect::new(520.0, 390.0, 240.0, 52.0),
                "Recover / New Game",
                true,
                ButtonTone::Positive,
                pointer,
            ) {
                actions.push(UiAction::NewGame);
            }
        }
        GamePhase::Playing => {}
    }
}

pub(super) fn stage_label(stage: SuspicionStage) -> &'static str {
    match stage {
        SuspicionStage::Calm => "Calm",
        SuspicionStage::Rumour => "Rumour",
        SuspicionStage::Questioning => "Questioning",
        SuspicionStage::Investigation => "Investigation",
    }
}

pub(super) fn status_label(status: WorkerStatus) -> &'static str {
    match status {
        WorkerStatus::Idle => "Idle",
        WorkerStatus::Walking => "Walking",
        WorkerStatus::Working => "Working",
        WorkerStatus::Carrying => "Carrying",
        WorkerStatus::Hiding => "Hiding",
    }
}

pub fn world_grid_rect() -> Rect {
    Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT)
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GridView {
    viewport: Rect,
    tile_size: f32,
    world_center: Vec2,
    camera: CameraTransform,
}

impl GridView {
    pub(super) fn new(ctx: &UiContext<'_>, viewport: Rect) -> Self {
        let tile_size = (viewport.w / ctx.session.world_width() as f32)
            .min(viewport.h / ctx.session.world_height() as f32);
        Self {
            viewport,
            tile_size,
            world_center: vec2(
                ctx.session.world_width() as f32,
                ctx.session.world_height() as f32,
            ) * tile_size
                * 0.5,
            camera: ctx.camera,
        }
    }
    pub(super) fn tile_rect(self, pos: TilePos) -> Rect {
        let world = vec2(pos.x as f32, pos.y as f32) * self.tile_size - self.world_center;
        let origin = self
            .camera
            .world_to_screen(self.viewport, world)
            .expect("valid map viewport");
        let size = self.tile_size * self.camera.zoom() + 1.0;
        Rect::new(origin.x, origin.y, size, size)
    }
    pub(super) fn tile_at(self, point: Vec2) -> TilePos {
        let world = self
            .camera
            .screen_to_world(self.viewport, point)
            .expect("valid map viewport");
        let tile = (world + self.world_center) / self.tile_size;
        TilePos::new(tile.x.floor() as i32, tile.y.floor() as i32)
    }

    pub(super) fn tile_size(self) -> f32 {
        self.tile_size * self.camera.zoom()
    }

    pub(super) fn actor_center(self, position: Vec2) -> Vec2 {
        let world = position * self.tile_size - self.world_center;
        let origin = self
            .camera
            .world_to_screen(self.viewport, world)
            .expect("valid map viewport");
        let size = self.tile_size();
        origin + vec2(size * 0.5, size * 0.5)
    }
}

pub(super) fn selected_tile_at(ctx: &UiContext<'_>, point: Vec2) -> Option<TilePos> {
    if !world_grid_rect().contains_point(point) {
        return None;
    }
    let view = GridView::new(ctx, world_grid_rect());
    let tile = view.tile_at(point);
    (tile.x >= 0
        && tile.y >= 0
        && tile.x < ctx.session.world_width() as i32
        && tile.y < ctx.session.world_height() as i32)
        .then_some(tile)
}
