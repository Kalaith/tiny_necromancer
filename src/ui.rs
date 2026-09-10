//! World-first cemetery composition. The map is always visible; panels explain the selected thing.

use crate::data::GameData;
use crate::state::{
    BuildingKind, GamePhase, GameSession, JobKind, Selection, Technology, UndeadKind, ZoneKind,
};
use macroquad::prelude::*;
use macroquad_toolkit::camera::CameraTransform;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::{Pointer, VirtualUi};

mod components;
mod hud;
mod panels;
mod research;
mod world;

pub use components::world_grid_rect;
use components::{
    draw_event_modal, draw_phase_overlay, draw_placement_controls, pause_control_rect,
    placement_cancel_rect, selected_tile_at,
};

pub const LOGICAL_WIDTH: f32 = 1280.0;
pub const LOGICAL_HEIGHT: f32 = 720.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    None,
    Build,
    Orders,
    Undead,
    Research,
    Zones,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UiAction {
    NewGame,
    TogglePause,
    Save,
    Load,
    SelectWorker(usize),
    SelectPlot(TilePos),
    SelectTile(TilePos),
    SelectNecromancer,
    MoveNecromancer(TilePos),
    AssignJob(JobKind),
    ToggleAutomation,
    Raise(UndeadKind),
    QueueBuilding(BuildingKind),
    BeginPlacement(BuildingKind),
    PlaceBuilding(TilePos),
    CancelPlacement,
    UnlockPlot,
    StartResearch(Technology),
    TogglePanel(Panel),
    ToggleZone(ZoneKind),
    PaintZone(TilePos),
    ResolveEvent(String),
}

pub struct UiContext<'a> {
    pub data: &'a GameData,
    pub session: &'a GameSession,
    pub save_exists: bool,
    pub camera: CameraTransform,
    pub ui: &'a VirtualUi,
    pub sprites: Option<&'a Texture2D>,
    pub title_background: Option<&'a Texture2D>,
    pub panel: Panel,
    pub placement: Option<BuildingKind>,
    pub zone_mode: Option<ZoneKind>,
}

pub fn draw_game_ui(ctx: UiContext<'_>) -> Vec<UiAction> {
    let pointer = Pointer::read(|point| ctx.ui.screen_to_ui(point));
    let mut actions = Vec::new();
    world::draw_world_scene(&ctx);
    if ctx.session.phase != GamePhase::MainMenu {
        hud::draw_status_strip(&ctx, pointer, &mut actions);
        hud::draw_inspector(&ctx, pointer, &mut actions);
        hud::draw_command_dock(&ctx, pointer, &mut actions);
        panels::draw_feed(&ctx);
        if ctx.session.research.is_unlocked(Technology::Gravecraft) {
            panels::draw_minimap(&ctx);
        }
    }
    if let Some(event_id) = &ctx.session.pressure.active_event {
        draw_event_modal(&ctx, event_id, pointer, &mut actions);
    } else {
        panels::draw_panel(&ctx, pointer, &mut actions);
        draw_placement_controls(&ctx, pointer, &mut actions);
        if pointer.released
            && ctx.session.phase == GamePhase::Playing
            && !ui_occludes(pointer.position, &ctx)
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
        draw_phase_overlay(&ctx, pointer, &mut actions);
    }
    actions
}

fn ui_occludes(point: Vec2, ctx: &UiContext<'_>) -> bool {
    let dock = Rect::new(304.0, 618.0, 672.0, 86.0);
    let inspector = if ctx.session.world.selected.is_some() {
        Rect::new(952.0, 86.0, 310.0, 454.0)
    } else {
        Rect::new(0.0, 0.0, 0.0, 0.0)
    };
    let panel = match ctx.panel {
        Panel::Research => Rect::new(238.0, 106.0, 680.0, 490.0),
        Panel::Build | Panel::Orders | Panel::Undead | Panel::Zones => {
            Rect::new(350.0, 460.0, 580.0, 150.0)
        }
        Panel::None => Rect::new(0.0, 0.0, 0.0, 0.0),
    };
    dock.contains_point(point)
        || inspector.contains_point(point)
        || panel.contains_point(point)
        || pause_control_rect().contains_point(point)
        || placement_cancel_rect().contains_point(point)
}

#[cfg(test)]
mod tests;
