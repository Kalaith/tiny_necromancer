//! Compact bottom-sheet composition for narrow touch viewports.

use super::components::{compact_virtual_button, selected_tile_at, virtual_button};
use super::{CameraZoom, Panel, UiAction, UiContext};
use crate::state::{GamePhase, Selection, Technology};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

mod panels;
mod selection;

#[cfg(test)]
mod tests;

pub(super) fn draw_compact_game_ui(ctx: UiContext<'_>) -> Vec<UiAction> {
    let pointer = Pointer::read(|point| ctx.ui.screen_to_ui(point));
    let pointer = if ctx.touch_claimed {
        pointer.suppressed()
    } else {
        pointer
    };
    let mut actions = Vec::new();
    super::world::draw_world_scene(&ctx);
    if ctx.session.phase == GamePhase::MainMenu {
        draw_compact_menu(&ctx, pointer, &mut actions);
        return actions;
    }
    draw_compact_status(&ctx, pointer, &mut actions);
    draw_compact_camera_controls(&ctx, pointer, &mut actions);
    if let Some(event_id) = &ctx.session.pressure.active_event {
        draw_compact_event(&ctx, event_id, pointer, &mut actions);
        return actions;
    }
    if ctx.session.phase == GamePhase::Victory || ctx.session.phase == GamePhase::Error {
        draw_compact_phase_overlay(&ctx, pointer, &mut actions);
        return actions;
    }
    draw_compact_sheet(&ctx, pointer, &mut actions);
    if pointer.released
        && ctx.session.phase == GamePhase::Playing
        && !super::ui_occludes(pointer.position, &ctx)
    {
        if let Some(tile) = selected_tile_at(&ctx, pointer.position) {
            if ctx.zone_mode.is_some() {
                actions.push(UiAction::PaintZone(tile));
            } else if ctx.placement.is_some() {
                actions.push(UiAction::PlaceBuilding(tile));
            } else if matches!(ctx.session.world.selected, Some(Selection::Necromancer)) {
                actions.push(UiAction::MoveNecromancer(tile));
            } else {
                actions.push(UiAction::SelectTile(tile));
            }
        }
    }
    actions
}

fn draw_compact_camera_controls(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    if ctx.session.pressure.active_event.is_some()
        || !matches!(ctx.session.phase, GamePhase::Playing | GamePhase::Paused)
    {
        return;
    }
    let [zoom_in, zoom_out, recenter] = ctx.layout.compact_camera_controls();
    let zoom = ctx.camera.zoom();
    if compact_virtual_button(
        zoom_in,
        "+",
        compact_zoom_in_enabled(zoom),
        ButtonTone::Secondary,
        16.0,
        pointer,
    ) {
        actions.push(UiAction::ZoomCamera(CameraZoom::In));
    }
    if compact_virtual_button(
        zoom_out,
        "-",
        compact_zoom_out_enabled(zoom),
        ButtonTone::Secondary,
        16.0,
        pointer,
    ) {
        actions.push(UiAction::ZoomCamera(CameraZoom::Out));
    }
    if compact_virtual_button(
        recenter,
        "Recenter",
        true,
        ButtonTone::Secondary,
        11.0,
        pointer,
    ) {
        actions.push(UiAction::CenterCamera);
    }
    draw_text_block(
        &format!("MAP ZOOM · {:.0}%", ctx.camera.zoom() * 100.0),
        recenter.right() + 12.0,
        recenter.y + 14.0,
        (ctx.layout.logical_width - recenter.right() - 24.0).max(60.0),
        16.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_text_block(
        if ctx.layout.logical_width < 520.0 {
            "DRAG / PINCH"
        } else {
            "DRAG MAP · PINCH TO ZOOM"
        },
        recenter.right() + 12.0,
        recenter.y + 28.0,
        (ctx.layout.logical_width - recenter.right() - 24.0).max(60.0),
        14.0,
        9.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn compact_zoom_in_enabled(zoom: f32) -> bool {
    zoom < 1.5 - f32::EPSILON
}

fn compact_zoom_out_enabled(zoom: f32) -> bool {
    zoom > 0.75 + f32::EPSILON
}

fn draw_compact_status(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let width = ctx.layout.logical_width;
    let pause = Rect::new(width - 88.0, 8.0, 80.0, 48.0);
    let values = [
        ("BONES", ctx.session.economy.bones, dark::TEXT_BRIGHT),
        (
            "MANA",
            ctx.session.economy.mana,
            Color::new(0.73, 0.79, 1.0, 1.0),
        ),
        (
            "WOOD",
            ctx.session.economy.wood,
            Color::new(0.86, 0.68, 0.40, 1.0),
        ),
    ];
    let show_undead_card = width >= 400.0;
    let card_count = if show_undead_card { 4.0 } else { 3.0 };
    let card_width = ((width - 104.0) / card_count).max(72.0);
    for (index, (label, value, color)) in values.into_iter().enumerate() {
        let rect = Rect::new(8.0 + index as f32 * card_width, 8.0, card_width - 4.0, 48.0);
        draw_surface(
            rect,
            &SurfaceStyle::new(Color::new(0.055, 0.07, 0.065, 0.92))
                .with_border(1.0, Color::new(0.42, 0.54, 0.46, 0.60)),
        );
        draw_text_block(
            label,
            rect.x + 8.0,
            rect.y + 7.0,
            rect.w - 16.0,
            14.0,
            9.0,
            0.0,
            dark::TEXT_DIM,
        );
        draw_text_block(
            &value.to_string(),
            rect.x + 8.0,
            rect.y + 23.0,
            rect.w - 16.0,
            20.0,
            17.0,
            0.0,
            color,
        );
    }
    if show_undead_card {
        let rect = Rect::new(8.0 + 3.0 * card_width, 8.0, card_width - 4.0, 48.0);
        draw_surface(
            rect,
            &SurfaceStyle::new(Color::new(0.055, 0.07, 0.065, 0.92))
                .with_border(1.0, Color::new(0.42, 0.54, 0.46, 0.60)),
        );
        draw_text_block(
            "UNDEAD",
            rect.x + 8.0,
            rect.y + 7.0,
            rect.w - 16.0,
            14.0,
            9.0,
            0.0,
            dark::TEXT_DIM,
        );
        draw_text_block(
            &ctx.session.active_undead().to_string(),
            rect.x + 8.0,
            rect.y + 23.0,
            rect.w - 16.0,
            20.0,
            17.0,
            0.0,
            dark::TEXT_BRIGHT,
        );
    }
    if virtual_button(
        pause,
        if ctx.session.phase == GamePhase::Paused {
            "Resume"
        } else {
            "Pause"
        },
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::TogglePause);
    }
    let suspicion = if show_undead_card {
        format!(
            "SUSPICION · {} {:.0}%",
            super::components::stage_label(ctx.session.pressure.stage),
            ctx.session.pressure.suspicion
        )
    } else {
        format!(
            "SUSPICION · {} {:.0}% · UNDEAD {}",
            super::components::stage_label(ctx.session.pressure.stage),
            ctx.session.pressure.suspicion,
            ctx.session.active_undead()
        )
    };
    draw_text_block(
        &suspicion,
        12.0,
        64.0,
        width - 24.0,
        18.0,
        12.0,
        0.0,
        if ctx.session.pressure.suspicion >= 50.0 {
            dark::WARNING
        } else {
            dark::TEXT_DIM
        },
    );
}

fn draw_compact_menu(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let rect = Rect::new(
        18.0,
        ctx.layout.logical_height * 0.24,
        ctx.layout.logical_width - 36.0,
        246.0,
    );
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.045, 0.055, 0.055, 0.97)).with_border(2.0, dark::ACCENT),
    );
    draw_text_centered_in_box(
        "TINY NECROMANCER",
        rect.x + 18.0,
        rect.y + 22.0,
        rect.w - 36.0,
        30.0,
        24.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_centered_in_box(
        "A cemetery command sheet for small screens.",
        rect.x + 18.0,
        rect.y + 58.0,
        rect.w - 36.0,
        22.0,
        13.0,
        dark::TEXT_DIM,
    );
    let button_width = (rect.w - 42.0) / 2.0;
    if virtual_button(
        Rect::new(rect.x + 14.0, rect.y + 112.0, button_width, 52.0),
        "New Game",
        true,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::NewGame);
    }
    if virtual_button(
        Rect::new(
            rect.x + 28.0 + button_width,
            rect.y + 112.0,
            button_width,
            52.0,
        ),
        "Continue",
        ctx.save_exists,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::Load);
    }
    draw_text_centered_in_box(
        if ctx.save_exists {
            "Continue restores the last cemetery."
        } else {
            "No saved cemetery yet."
        },
        rect.x + 18.0,
        rect.y + 184.0,
        rect.w - 36.0,
        22.0,
        13.0,
        dark::TEXT_DIM,
    );
}

fn draw_compact_sheet(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let sheet = ctx.layout.sheet_rect;
    draw_surface(
        sheet,
        &SurfaceStyle::new(Color::new(0.045, 0.055, 0.055, 0.98))
            .with_border(1.0, Color::new(0.58, 0.70, 0.62, 0.80)),
    );
    draw_text_block(
        "COMMAND SHEET",
        sheet.x + 12.0,
        sheet.y + 7.0,
        150.0,
        14.0,
        10.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_compact_navigation(ctx, pointer, actions, sheet);
    if ctx.session.phase == GamePhase::Paused {
        draw_text_centered_in_box(
            "PAUSED · Tap RESUME above to continue the cemetery.",
            sheet.x + 14.0,
            sheet.y + 86.0,
            sheet.w - 28.0,
            32.0,
            15.0,
            dark::WARNING,
        );
        return;
    }
    if ctx.panel == Panel::None {
        draw_compact_selection(ctx, pointer, actions, sheet);
    } else {
        panels::draw_compact_panel(ctx, pointer, actions, sheet);
    }
}

fn draw_compact_navigation(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    let entries = [
        (Panel::Build, "Build", true),
        (Panel::Orders, "Orders", true),
        (Panel::Undead, "Undead", true),
        (Panel::Research, "Research", true),
        (
            Panel::Zones,
            "Zones",
            ctx.session.research.is_unlocked(Technology::Gravecraft),
        ),
        (
            Panel::Domain,
            "Domain",
            ctx.session
                .research
                .is_unlocked(Technology::DomainStewardship),
        ),
        (Panel::Feed, "Notes", true),
    ];
    let gap = COMPACT_NAV_GAP;
    let button_width = compact_nav_button_width(sheet.w);
    let text_size = compact_nav_text_size(button_width);
    for (index, (panel, label, enabled)) in entries.into_iter().enumerate() {
        let label = if button_width < 56.0 {
            match panel {
                Panel::Orders => "Jobs",
                Panel::Undead => "Dead",
                Panel::Research => "Tech",
                Panel::Domain => "Rules",
                _ => label,
            }
        } else {
            label
        };
        let button = Rect::new(
            sheet.x + 12.0 + index as f32 * (button_width + gap),
            sheet.y + 24.0,
            button_width,
            44.0,
        );
        if compact_virtual_button(
            button,
            label,
            enabled,
            if ctx.panel == panel {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            text_size,
            pointer,
        ) {
            actions.push(UiAction::TogglePanel(panel));
        }
    }
}

const COMPACT_NAV_ENTRIES: f32 = 7.0;
const COMPACT_NAV_GAP: f32 = 4.0;
const COMPACT_NAV_MARGIN: f32 = 24.0;
const COMPACT_NAV_MIN_BUTTON: f32 = 44.0;

fn compact_nav_button_width(width: f32) -> f32 {
    ((width - COMPACT_NAV_MARGIN - COMPACT_NAV_GAP * (COMPACT_NAV_ENTRIES - 1.0))
        / COMPACT_NAV_ENTRIES)
        .max(COMPACT_NAV_MIN_BUTTON)
}

fn compact_nav_text_size(button_width: f32) -> f32 {
    if button_width < 56.0 {
        9.0
    } else {
        12.0
    }
}

fn draw_compact_selection(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    selection::draw(ctx, pointer, actions, sheet);
}

fn draw_compact_event(
    ctx: &UiContext<'_>,
    event_id: &str,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let Some(event) = ctx.data.events.get(event_id) else {
        return;
    };
    full_screen_overlay(0.72);
    let rect = Rect::new(
        12.0,
        82.0,
        ctx.layout.logical_width - 24.0,
        ctx.layout.logical_height - 96.0,
    );
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.075, 0.07, 0.09, 0.99)).with_border(2.0, dark::WARNING),
    );
    draw_text_block(
        "ROAD PRESSURE",
        rect.x + 16.0,
        rect.y + 18.0,
        rect.w - 32.0,
        18.0,
        12.0,
        0.0,
        dark::WARNING,
    );
    draw_text_centered_in_box(
        &event.title,
        rect.x + 16.0,
        rect.y + 42.0,
        rect.w - 32.0,
        32.0,
        23.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        &event.body,
        rect.x + 18.0,
        rect.y + 88.0,
        rect.w - 36.0,
        74.0,
        15.0,
        4.0,
        dark::TEXT,
    );
    for (index, choice) in event.choices.iter().enumerate() {
        let button = Rect::new(
            rect.x + 18.0,
            rect.y + 180.0 + index as f32 * 84.0,
            rect.w - 36.0,
            64.0,
        );
        if virtual_button(button, &choice.label, true, ButtonTone::Warning, pointer) {
            actions.push(UiAction::ResolveEvent(choice.id.clone()));
        }
        draw_text_block(
            &choice.description,
            button.x + 10.0,
            button.bottom() + 5.0,
            button.w - 20.0,
            24.0,
            11.0,
            3.0,
            dark::TEXT_DIM,
        );
    }
}

fn draw_compact_phase_overlay(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    full_screen_overlay(0.64);
    let rect = Rect::new(
        18.0,
        ctx.layout.logical_height * 0.24,
        ctx.layout.logical_width - 36.0,
        240.0,
    );
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.055, 0.065, 0.065, 0.99)).with_border(
            2.0,
            if ctx.session.phase == GamePhase::Victory {
                dark::POSITIVE
            } else {
                dark::NEGATIVE
            },
        ),
    );
    let (title, body, button) = match ctx.session.phase {
        GamePhase::Victory => (
            "Settlement established",
            "The cemetery has workers, shelter, light, and room to grow.",
            "Play Again",
        ),
        GamePhase::Error => (
            "The cemetery needs a moment",
            ctx.session
                .error_message
                .as_deref()
                .unwrap_or("Unknown game error"),
            "Recover / New Game",
        ),
        _ => return,
    };
    draw_text_centered_in_box(
        title,
        rect.x + 18.0,
        rect.y + 28.0,
        rect.w - 36.0,
        30.0,
        22.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        body,
        rect.x + 24.0,
        rect.y + 76.0,
        rect.w - 48.0,
        46.0,
        14.0,
        3.0,
        dark::TEXT,
    );
    if virtual_button(
        Rect::new(rect.x + 24.0, rect.y + 154.0, rect.w - 48.0, 52.0),
        button,
        true,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::NewGame);
    }
}
