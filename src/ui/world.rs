//! World and sprite rendering.

use super::components::{selected_tile_at, GridView};
use super::{world_grid_rect, UiContext, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use crate::state::{
    Building, BuildingKind, GamePhase, PlotStatus, Selection, Technology, WorkerStatus, ZoneKind,
};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::VirtualUi;

pub(super) fn draw_world_scene(ctx: &UiContext<'_>) {
    if ctx.session.phase == GamePhase::MainMenu {
        draw_title_background(ctx.title_background);
        return;
    }
    let view = GridView::new(ctx, world_grid_rect());
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.055, 0.12, 0.085, 1.0),
    );
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.02, 0.04, 0.035, 0.20),
    );
    for y in 0..ctx.session.world_height() {
        for x in 0..ctx.session.world_width() {
            let tile_pos = TilePos::new(x as i32, y as i32);
            let tile = view.tile_rect(tile_pos);
            let is_road = x as i32 >= ctx.session.world.road_x;
            let base = if is_road {
                Color::new(0.24, 0.19, 0.15, 1.0)
            } else if x == 0 {
                Color::new(0.10, 0.20, 0.13, 1.0)
            } else {
                Color::new(0.12, 0.25, 0.16, 1.0)
            };
            draw_rectangle(tile.x, tile.y, tile.w + 1.0, tile.h + 1.0, base);
            let mote = ((x * 17 + y * 31) % 7) as f32;
            if !is_road {
                draw_circle(
                    tile.x + tile.w * (0.25 + mote * 0.04),
                    tile.y + tile.h * 0.30,
                    tile.w * 0.018,
                    Color::new(0.24, 0.39, 0.20, 0.55),
                );
                draw_circle(
                    tile.x + tile.w * 0.72,
                    tile.y + tile.h * 0.72,
                    tile.w * 0.012,
                    Color::new(0.06, 0.14, 0.09, 0.65),
                );
            } else {
                draw_line(
                    tile.x + tile.w * 0.12,
                    tile.y + tile.h * 0.26,
                    tile.right() - tile.w * 0.12,
                    tile.y + tile.h * 0.26,
                    2.0,
                    Color::new(0.38, 0.30, 0.23, 0.8),
                );
            }
        }
    }
    draw_path_and_props(ctx, &view);
    draw_zones(ctx, &view);
    draw_graves(ctx, &view);
    draw_buildings(ctx, &view);
    draw_workers(ctx, &view);
    draw_necromancer(ctx, &view);
    if let Some(kind) = ctx.placement {
        draw_placement_preview(ctx, &view, kind);
    }
}

fn draw_title_background(texture: Option<&Texture2D>) {
    draw_rectangle(
        0.0,
        0.0,
        LOGICAL_WIDTH,
        LOGICAL_HEIGHT,
        Color::new(0.025, 0.045, 0.055, 1.0),
    );
    if let Some(texture) = texture {
        draw_texture_ex(
            texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(LOGICAL_WIDTH, LOGICAL_HEIGHT)),
                ..Default::default()
            },
        );
    }
}

fn draw_path_and_props(ctx: &UiContext<'_>, view: &GridView) {
    for (index, tree) in ctx.session.world.forest_tiles.iter().enumerate() {
        let rect = view.tile_rect(*tree);
        let center = vec2(rect.center().x, rect.y + rect.h * 0.55);
        draw_rectangle(
            center.x - rect.w * 0.06,
            center.y - rect.h * 0.05,
            rect.w * 0.12,
            rect.h * 0.38,
            Color::new(0.24, 0.16, 0.10, 1.0),
        );
        draw_circle(
            center.x,
            center.y - rect.h * 0.24,
            rect.w * (0.24 + (index % 3) as f32 * 0.025),
            Color::new(0.08, 0.25, 0.14, 1.0),
        );
        draw_circle(
            center.x - rect.w * 0.12,
            center.y - rect.h * 0.10,
            rect.w * 0.14,
            Color::new(0.12, 0.33, 0.18, 1.0),
        );
    }
    let stock = view
        .tile_rect(crate::state::WorldState::stockpile_position())
        .inset(10.0);
    draw_rectangle(
        stock.x,
        stock.y + stock.h * 0.45,
        stock.w,
        stock.h * 0.32,
        Color::new(0.37, 0.24, 0.12, 1.0),
    );
    draw_line(
        stock.x,
        stock.y + stock.h * 0.45,
        stock.right(),
        stock.y + stock.h * 0.45,
        3.0,
        Color::new(0.60, 0.42, 0.20, 1.0),
    );
    draw_text_centered_in_box(
        "STOCKPILE",
        stock.x - 16.0,
        stock.bottom() - 2.0,
        stock.w + 32.0,
        18.0,
        11.0,
        Color::new(0.85, 0.72, 0.54, 0.75),
    );
}

fn draw_zones(ctx: &UiContext<'_>, view: &GridView) {
    for zone in &ctx.session.world.zones {
        let color = match zone.kind {
            ZoneKind::Work => Color::new(0.27, 0.73, 0.62, 0.20),
            ZoneKind::Storage => Color::new(0.82, 0.60, 0.26, 0.20),
            ZoneKind::Patrol => Color::new(0.46, 0.58, 0.86, 0.20),
        };
        for tile_pos in &zone.tiles {
            let tile = view.tile_rect(*tile_pos).inset(3.0);
            draw_rectangle(tile.x, tile.y, tile.w, tile.h, color);
            draw_rectangle_lines(tile.x, tile.y, tile.w, tile.h, 1.0, color.with_alpha(0.55));
        }
    }
}

fn draw_graves(ctx: &UiContext<'_>, view: &GridView) {
    for plot in &ctx.session.world.plots {
        let tile = view.tile_rect(plot.position);
        let ground = tile.inset(tile.w * 0.14);
        let selected = ctx.session.world.selected == Some(Selection::Grave(plot.id));
        if plot.status == PlotStatus::Locked {
            draw_rectangle(
                ground.x,
                ground.y,
                ground.w,
                ground.h,
                Color::new(0.07, 0.12, 0.09, 0.6),
            );
            draw_text_centered_in_box(
                "·",
                ground.x,
                ground.y + ground.h * 0.16,
                ground.w,
                ground.h * 0.5,
                26.0,
                Color::new(0.54, 0.62, 0.48, 0.40),
            );
            continue;
        }
        let earth = if plot.status == PlotStatus::Dug {
            Color::new(0.20, 0.12, 0.09, 1.0)
        } else {
            Color::new(0.28, 0.18, 0.11, 1.0)
        };
        draw_rectangle(
            ground.x,
            ground.y + ground.h * 0.48,
            ground.w,
            ground.h * 0.32,
            earth,
        );
        if plot.status == PlotStatus::Dug {
            draw_rectangle(
                ground.x + ground.w * 0.16,
                ground.y + ground.h * 0.54,
                ground.w * 0.68,
                ground.h * 0.12,
                Color::new(0.06, 0.07, 0.06, 0.95),
            );
            draw_line(
                ground.x + ground.w * 0.23,
                ground.y + ground.h * 0.50,
                ground.right() - ground.w * 0.23,
                ground.y + ground.h * 0.50,
                2.0,
                Color::new(0.48, 0.34, 0.20, 1.0),
            );
        } else {
            draw_rectangle(
                ground.x + ground.w * 0.39,
                ground.y + ground.h * 0.19,
                ground.w * 0.22,
                ground.h * 0.46,
                Color::new(0.62, 0.65, 0.57, 1.0),
            );
            draw_triangle(
                vec2(ground.x + ground.w * 0.39, ground.y + ground.h * 0.19),
                vec2(ground.x + ground.w * 0.61, ground.y + ground.h * 0.19),
                vec2(ground.x + ground.w * 0.61, ground.y + ground.h * 0.28),
                Color::new(0.40, 0.44, 0.39, 1.0),
            );
            draw_line(
                ground.center().x,
                ground.y + ground.h * 0.28,
                ground.center().x,
                ground.y + ground.h * 0.49,
                2.0,
                Color::new(0.35, 0.40, 0.35, 1.0),
            );
            draw_line(
                ground.x + ground.w * 0.47,
                ground.y + ground.h * 0.36,
                ground.x + ground.w * 0.53,
                ground.y + ground.h * 0.36,
                2.0,
                Color::new(0.35, 0.40, 0.35, 1.0),
            );
        }
        if plot.status == PlotStatus::Digging {
            progress_bar(
                ground.x + 4.0,
                ground.bottom() - 10.0,
                ground.w - 8.0,
                6.0,
                plot.progress,
                ctx.data.jobs.get("dig").map_or(8.0, |job| job.work_seconds),
                dark::WARNING,
            );
        }
        if selected {
            draw_selection_rect(ground, dark::ACCENT);
        }
    }
}

fn draw_buildings(ctx: &UiContext<'_>, view: &GridView) {
    for (index, building) in ctx.session.world.buildings.iter().enumerate() {
        let footprint = building_footprint(view, building);
        let selected = ctx.session.world.selected == Some(Selection::Building(index));
        if !building.complete {
            draw_rectangle(
                footprint.x,
                footprint.y,
                footprint.w,
                footprint.h,
                Color::new(0.28, 0.22, 0.14, 0.88),
            );
            draw_line(
                footprint.x,
                footprint.y,
                footprint.right(),
                footprint.bottom(),
                3.0,
                Color::new(0.70, 0.52, 0.27, 0.85),
            );
            draw_line(
                footprint.right(),
                footprint.y,
                footprint.x,
                footprint.bottom(),
                3.0,
                Color::new(0.70, 0.52, 0.27, 0.85),
            );
            progress_bar(
                footprint.x + 8.0,
                footprint.bottom() - 14.0,
                footprint.w - 16.0,
                8.0,
                building.progress,
                ctx.data
                    .buildings
                    .get(building.kind.id())
                    .map_or(10.0, |def| def.build_seconds),
                dark::WARNING,
            );
        } else if let Some(texture) = ctx.sprites {
            let quadrant = match building.kind {
                BuildingKind::WorkShed => 2,
                BuildingKind::GraveLantern | BuildingKind::OssuaryKiln => 3,
            };
            draw_sheet_sprite(
                texture,
                quadrant,
                footprint.center() + vec2(0.0, -footprint.h * 0.06),
                vec2(footprint.w * 1.10, footprint.h * 1.16),
            );
        }
        if selected {
            draw_selection_rect(footprint, dark::ACCENT);
        }
    }
}

fn draw_workers(ctx: &UiContext<'_>, view: &GridView) {
    for (index, worker) in ctx.session.workforce.workers.iter().enumerate() {
        let tile = view.tile_rect(worker.position);
        let selected = ctx.session.world.selected == Some(Selection::Worker(index));
        if let Some(texture) = ctx.sprites {
            draw_sheet_sprite(
                texture,
                1,
                tile.center() + vec2(0.0, -tile.h * 0.10),
                vec2(tile.w * 0.82, tile.h * 0.92),
            );
        } else {
            draw_circle(
                tile.center().x,
                tile.center().y,
                tile.w * 0.24,
                Color::new(0.75, 0.78, 0.72, 1.0),
            );
        }
        let marker = tile.center() + vec2(tile.w * 0.30, -tile.h * 0.26);
        draw_circle(
            marker.x,
            marker.y,
            5.0,
            match worker.status {
                WorkerStatus::Idle => dark::WARNING,
                WorkerStatus::Carrying => dark::POSITIVE,
                WorkerStatus::Walking => dark::TEXT_BRIGHT,
                _ => dark::ACCENT,
            },
        );
        if worker.carrying > 0 || worker.status == WorkerStatus::Carrying {
            draw_rectangle(
                tile.center().x - 7.0,
                tile.center().y - tile.h * 0.05,
                14.0,
                10.0,
                Color::new(0.52, 0.34, 0.16, 1.0),
            );
        }
        if selected {
            draw_selection_circle(tile.center(), tile.w * 0.35, dark::ACCENT);
        }
    }
}

fn draw_necromancer(ctx: &UiContext<'_>, view: &GridView) {
    let tile = view.tile_rect(ctx.session.world.necromancer_position);
    if let Some(texture) = ctx.sprites {
        draw_sheet_sprite(
            texture,
            0,
            tile.center() + vec2(0.0, -tile.h * 0.12),
            vec2(tile.w * 0.92, tile.h * 1.02),
        );
    } else {
        draw_circle(
            tile.center().x,
            tile.center().y,
            tile.w * 0.25,
            Color::new(0.12, 0.10, 0.15, 1.0),
        );
    }
    if ctx.session.world.selected == Some(Selection::Necromancer) {
        draw_selection_circle(tile.center(), tile.w * 0.38, dark::ACCENT);
    }
}

fn draw_sheet_sprite(texture: &Texture2D, quadrant: usize, center: Vec2, size: Vec2) {
    let half_w = texture.width() * 0.5;
    let half_h = texture.height() * 0.5;
    let source = Rect::new(
        (quadrant % 2) as f32 * half_w,
        (quadrant / 2) as f32 * half_h,
        half_w,
        half_h,
    );
    draw_texture_ex(
        texture,
        center.x - size.x * 0.5,
        center.y - size.y * 0.5,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            source: Some(source),
            ..Default::default()
        },
    );
}

fn building_footprint(view: &GridView, building: &Building) -> Rect {
    let start = view.tile_rect(building.position);
    let end = view.tile_rect(TilePos::new(
        building.position.x + building.width - 1,
        building.position.y + building.height - 1,
    ));
    Rect::new(
        start.x + 3.0,
        start.y + 3.0,
        end.right() - start.x - 6.0,
        end.bottom() - start.y - 6.0,
    )
}

fn draw_selection_rect(rect: Rect, color: Color) {
    draw_rectangle_lines(
        rect.x - 4.0,
        rect.y - 4.0,
        rect.w + 8.0,
        rect.h + 8.0,
        3.0,
        color,
    );
}
fn draw_selection_circle(center: Vec2, radius: f32, color: Color) {
    draw_circle_lines(center.x, center.y, radius, 3.0, color);
}

fn draw_placement_preview(ctx: &UiContext<'_>, view: &GridView, kind: BuildingKind) {
    let mouse = VirtualUi::new(LOGICAL_WIDTH, LOGICAL_HEIGHT).mouse_position();
    let tile =
        selected_tile_at(ctx, mouse).unwrap_or(crate::state::WorldState::stockpile_position());
    let (width, height) = kind.dimensions();
    let start = view.tile_rect(tile);
    let end = view.tile_rect(TilePos::new(tile.x + width - 1, tile.y + height - 1));
    let footprint = Rect::new(
        start.x,
        start.y,
        end.right() - start.x,
        end.bottom() - start.y,
    );
    let (valid, reason) = placement_valid(ctx, kind, tile);
    let outline = if valid { dark::ACCENT } else { dark::NEGATIVE };
    draw_rectangle(
        footprint.x,
        footprint.y,
        footprint.w,
        footprint.h,
        outline.with_alpha(0.18),
    );
    draw_rectangle_lines(
        footprint.x,
        footprint.y,
        footprint.w,
        footprint.h,
        3.0,
        outline,
    );
    draw_text_block(
        if valid {
            "Tap CANCEL PLACEMENT to return"
        } else {
            reason
        },
        footprint.x,
        footprint.y - 20.0,
        340.0,
        18.0,
        13.0,
        0.0,
        if valid {
            dark::TEXT_BRIGHT
        } else {
            dark::NEGATIVE
        },
    );
}

fn placement_valid(
    ctx: &UiContext<'_>,
    kind: BuildingKind,
    position: TilePos,
) -> (bool, &'static str) {
    let (width, height) = kind.dimensions();
    if position.x < 0
        || position.y < 0
        || position.x + width > ctx.session.world.width as i32
        || position.y + height > ctx.session.world.height as i32
    {
        return (false, "BLOCKED · outside the cemetery");
    }
    if !ctx.session.research.is_unlocked(Technology::Gravecraft)
        && position != crate::state::default_building_position_for_kind(kind)
    {
        return (false, "BLOCKED · study Gravecraft");
    }
    let overlaps = ctx.session.world.buildings.iter().any(|building| {
        position.x < building.position.x + building.width
            && position.x + width > building.position.x
            && position.y < building.position.y + building.height
            && position.y + height > building.position.y
    });
    if overlaps {
        (false, "BLOCKED · overlaps a structure")
    } else if (0..width).any(|x| {
        (0..height).any(|y| {
            ctx.session
                .world
                .is_building_obstacle(TilePos::new(position.x + x, position.y + y))
        })
    }) {
        (false, "BLOCKED · grave, trees, or road")
    } else {
        (true, "PLACE · Tap CANCEL PLACEMENT to return")
    }
}
