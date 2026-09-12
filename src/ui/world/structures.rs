//! Grave and building sprites rendered on the world grid.

use super::*;

pub(super) fn draw_graves(ctx: &UiContext<'_>, view: &GridView) {
    for plot in &ctx.session.world.plots {
        let tile = view.tile_rect(plot.position);
        let ground = tile.inset(tile.w * 0.14);
        let selected = ctx.session.world.selected == Some(Selection::Grave(plot.id));
        if plot.status == PlotStatus::Locked {
            draw_locked_grave(ground);
            continue;
        }
        draw_grave_earth(ground, plot.status);
        draw_grave_progress(ctx, plot, ground);
        if selected {
            super::draw_selection_rect(ground, dark::ACCENT);
        }
    }
}

fn draw_locked_grave(ground: Rect) {
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
}

fn draw_grave_earth(ground: Rect, status: PlotStatus) {
    let earth = if status == PlotStatus::Dug {
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
    if status == PlotStatus::Dug {
        draw_open_grave(ground);
    } else {
        draw_closed_grave(ground);
    }
}

fn draw_open_grave(ground: Rect) {
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
}

fn draw_closed_grave(ground: Rect) {
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

fn draw_grave_progress(ctx: &UiContext<'_>, plot: &crate::state::Plot, ground: Rect) {
    if plot.status != PlotStatus::Digging {
        return;
    }
    progress_bar(
        ground.x + 4.0,
        ground.bottom() - 10.0,
        ground.w - 8.0,
        6.0,
        plot.progress,
        ctx.data.jobs.get("dig").map_or(8.0, |job| job.work_seconds),
        dark::WARNING,
    );
    if ctx
        .session
        .workforce
        .workers
        .iter()
        .any(|worker| worker.target_plot == Some(plot.id))
    {
        draw_rectangle_lines(
            ground.x - 3.0,
            ground.y - 3.0,
            ground.w + 6.0,
            ground.h + 6.0,
            2.0,
            dark::WARNING,
        );
    }
}

pub(super) fn draw_buildings(ctx: &UiContext<'_>, view: &GridView) {
    for (index, building) in ctx.session.world.buildings.iter().enumerate() {
        let footprint = super::building_footprint(view, building);
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
            super::draw_sheet_sprite(
                texture,
                quadrant,
                footprint.center() + vec2(0.0, -footprint.h * 0.06),
                vec2(footprint.w * 1.10, footprint.h * 1.16),
                0.0,
            );
        }
        if selected {
            super::draw_selection_rect(footprint, dark::ACCENT);
        }
    }
}
