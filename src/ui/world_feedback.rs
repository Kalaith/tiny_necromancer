//! World markers for active routes, loose resources, and work destinations.

use super::components::GridView;
use super::UiContext;
use crate::engine::{districts, navigation};
use crate::state::{
    Building, BuildingKind, JobKind, PlotStatus, ResourceKind, Selection, Worker, WorkerStatus,
    WorldState, ZoneKind,
};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;

pub(super) fn draw_loose_resource_feedback(ctx: &UiContext<'_>, view: &GridView) {
    if ctx.session.economy.loose_bones > 0 {
        let source = ctx.session.economy.loose_bones_source.or_else(|| {
            ctx.session
                .world
                .plots
                .iter()
                .find(|plot| plot.status == PlotStatus::Dug)
                .map(|plot| plot.position)
        });
        if let Some(source) = source {
            draw_resource_pile(
                view.tile_rect(source).center(),
                "B",
                ctx.session.economy.loose_bones,
                Color::new(0.82, 0.82, 0.72, 1.0),
            );
        }
    }
    if ctx.session.economy.loose_wood > 0 {
        if let Some(source) = ctx
            .session
            .economy
            .loose_wood_source
            .or_else(|| ctx.session.world.forest_tiles.first().copied())
        {
            draw_resource_pile(
                view.tile_rect(source).center() + vec2(0.0, 7.0),
                "W",
                ctx.session.economy.loose_wood,
                Color::new(0.70, 0.45, 0.20, 1.0),
            );
        }
    }
}

fn draw_resource_pile(center: Vec2, glyph: &str, amount: i32, color: Color) {
    draw_circle(center.x, center.y + 9.0, 10.0, color.with_alpha(0.84));
    draw_text_centered_in_box(
        glyph,
        center.x - 9.0,
        center.y,
        18.0,
        18.0,
        13.0,
        dark::BACKGROUND,
    );
    draw_text_block(
        &format!("+{amount}"),
        center.x + 12.0,
        center.y - 10.0,
        42.0,
        18.0,
        12.0,
        0.0,
        color,
    );
}

pub(super) fn draw_actor_destinations(ctx: &UiContext<'_>, view: &GridView) {
    for (index, worker) in ctx.session.workforce.workers.iter().enumerate() {
        if ctx.domain_overlays.routes
            || ctx.session.world.selected == Some(Selection::Worker(index))
        {
            if let Some(destination) = worker_destination(ctx, worker) {
                draw_route_hint(ctx, view, worker, destination);
            }
        }
    }
    if let Some(destination) = ctx.session.world.necromancer_destination {
        let from = view.actor_center(ctx.motions.necromancer().visual_position());
        let target = view.tile_rect(destination).center();
        draw_line(
            from.x,
            from.y,
            target.x,
            target.y,
            2.0,
            dark::ACCENT.with_alpha(0.42),
        );
        let pulse = (ctx.animation_time * std::f32::consts::TAU).sin() * 2.0;
        draw_circle_lines(
            target.x,
            target.y,
            view.tile_size() * 0.28 + pulse,
            3.0,
            dark::ACCENT,
        );
        draw_text_block(
            "DESTINATION",
            target.x - 44.0,
            target.y - view.tile_size() * 0.38,
            100.0,
            15.0,
            10.0,
            0.0,
            dark::ACCENT,
        );
    }
}

fn draw_route_hint(ctx: &UiContext<'_>, view: &GridView, worker: &Worker, destination: TilePos) {
    let worker_id = worker.id;
    let start = worker.position;
    let mut cursor = start;
    let actor = ctx.motions.worker(worker_id).map_or_else(
        || view.tile_rect(start).center(),
        |motion| view.actor_center(motion.visual_position()),
    );
    let target_center = view.tile_rect(destination).center();
    draw_line(
        actor.x,
        actor.y,
        target_center.x,
        target_center.y,
        2.0,
        dark::ACCENT.with_alpha(0.28),
    );
    for _ in 0..4 {
        if cursor == destination {
            break;
        }
        let Some(next) = navigation::next_step(ctx.session, cursor, destination) else {
            break;
        };
        cursor = next;
        let marker = view.tile_rect(cursor).center();
        draw_circle(marker.x, marker.y, 3.0, dark::ACCENT.with_alpha(0.62));
    }
    let target = view.tile_rect(destination).inset(view.tile_size() * 0.25);
    draw_rectangle_lines(target.x, target.y, target.w, target.h, 2.0, dark::ACCENT);
    let route_label = if ctx.domain_overlays.routes {
        district_rule_hint(ctx, worker, destination).map_or_else(
            || format!("{} · {}", worker.name, worker.assignment.label()),
            |hint| format!("{} · {} · {hint}", worker.name, worker.assignment.label()),
        )
    } else {
        "DESTINATION".to_owned()
    };
    draw_text_block(
        &route_label,
        target.x - 44.0,
        target.y - view.tile_size() * 0.38,
        150.0,
        15.0,
        10.0,
        0.0,
        dark::ACCENT,
    );
    if let Some(worker) = ctx
        .session
        .workforce
        .workers
        .iter()
        .find(|worker| worker.id == worker_id)
    {
        if worker.status == WorkerStatus::Working || worker.status == WorkerStatus::Carrying {
            let seconds = match worker.assignment {
                JobKind::Dig => ctx.data.jobs.get("dig").map_or(8.0, |job| job.work_seconds),
                JobKind::Haul => ctx
                    .data
                    .jobs
                    .get("haul")
                    .map_or(4.0, |job| job.work_seconds),
                JobKind::Wood => ctx
                    .data
                    .jobs
                    .get("wood")
                    .map_or(7.0, |job| job.work_seconds),
                JobKind::Build => ctx
                    .data
                    .jobs
                    .get("build")
                    .map_or(1.0, |job| job.work_seconds),
                JobKind::Refine => ctx
                    .data
                    .jobs
                    .get("refine")
                    .map_or(1.0, |job| job.work_seconds),
                JobKind::Guard => 1.0,
            };
            progress_bar(
                target.x,
                target.bottom() + 5.0,
                target.w,
                6.0,
                worker.progress,
                seconds,
                if worker.status == WorkerStatus::Carrying {
                    dark::POSITIVE
                } else {
                    dark::ACCENT
                },
            );
        }
    }
}

fn district_rule_hint(
    ctx: &UiContext<'_>,
    worker: &Worker,
    destination: TilePos,
) -> Option<&'static str> {
    match worker.assignment {
        JobKind::Dig | JobKind::Wood
            if districts::work_speed_multiplier(
                ctx.session,
                &ctx.data.config.district_rules,
                destination,
            ) > 1.0 =>
        {
            Some("Work rule")
        }
        JobKind::Haul
            if worker.carrying > 0
                && districts::haul_capacity_bonus(ctx.session, &ctx.data.config.district_rules)
                    > 0 =>
        {
            Some("Storage rule")
        }
        JobKind::Guard
            if districts::guard_mitigation_multiplier(
                ctx.session,
                &ctx.data.config.district_rules,
            ) > 1.0 =>
        {
            Some("Patrol rule")
        }
        _ => None,
    }
}

fn worker_destination(ctx: &UiContext<'_>, worker: &Worker) -> Option<TilePos> {
    match worker.assignment {
        JobKind::Dig => worker
            .target_plot
            .and_then(|plot_id| ctx.session.world.plots.get(plot_id))
            .map(|plot| plot.position),
        JobKind::Haul => {
            if worker.carrying > 0 {
                Some(ctx.session.world.storage_position_for(worker.position))
            } else if ctx.session.economy.loose_bones > 0 {
                ctx.session
                    .economy
                    .loose_bones_source
                    .or_else(|| {
                        ctx.session
                            .world
                            .plots
                            .iter()
                            .find(|plot| plot.status == PlotStatus::Dug)
                            .map(|plot| plot.position)
                    })
                    .or(Some(WorldState::stockpile_position()))
            } else if ctx.session.economy.loose_wood > 0 {
                ctx.session
                    .economy
                    .loose_wood_source
                    .or_else(|| ctx.session.world.forest_tiles.first().copied())
            } else {
                None
            }
        }
        JobKind::Guard => Some(ctx.session.world.patrol_position_for(worker.position)),
        JobKind::Wood => ctx
            .session
            .world
            .forest_tiles
            .iter()
            .filter(|tile| ctx.session.world.zone_contains(ZoneKind::Work, **tile))
            .min_by_key(|tile| {
                (worker.position.x - tile.x).abs() + (worker.position.y - tile.y).abs()
            })
            .copied()
            .or_else(|| ctx.session.world.forest_tiles.first().copied()),
        JobKind::Build => ctx
            .session
            .world
            .buildings
            .iter()
            .find(|building| !building.complete)
            .map(Building::work_position),
        JobKind::Refine => ctx
            .session
            .world
            .buildings
            .iter()
            .find(|building| building.kind == BuildingKind::OssuaryKiln && building.complete)
            .map(Building::work_position),
    }
}

pub(super) fn worker_idle_reason(ctx: &UiContext<'_>, worker: &Worker) -> &'static str {
    if worker.status != WorkerStatus::Idle {
        return "Active in the clearing";
    }
    match worker.assignment {
        JobKind::Dig => {
            if ctx
                .session
                .world
                .plots
                .iter()
                .any(|plot| plot.status == PlotStatus::Ready)
            {
                "Waiting for a route to the next grave"
            } else {
                "No grave available"
            }
        }
        JobKind::Haul => {
            if worker.carrying > 0 {
                "Carrying a bundle to storage"
            } else if ctx.session.economy.loose_bones > 0 || ctx.session.economy.loose_wood > 0 {
                "Waiting for a route to loose material"
            } else {
                "No loose material"
            }
        }
        JobKind::Guard => "Waiting for a patrol route",
        JobKind::Wood => {
            if ctx.session.world.forest_tiles.is_empty() {
                "No forest edge available"
            } else {
                "Waiting for a route to the forest"
            }
        }
        JobKind::Build => {
            if ctx
                .session
                .world
                .buildings
                .iter()
                .any(|building| !building.complete)
            {
                "Waiting for a route to construction"
            } else {
                "No structure under construction"
            }
        }
        JobKind::Refine => {
            if ctx.session.progress.production.is_some() {
                "Waiting for a route to the kiln"
            } else {
                "No kiln cycle loaded"
            }
        }
    }
}

pub(super) fn worker_activity_detail(worker: &Worker) -> String {
    if worker.carrying > 0 {
        let resource = match worker.carrying_resource.unwrap_or(ResourceKind::Bones) {
            ResourceKind::Bones => "bones",
            ResourceKind::Wood => "wood",
        };
        return format!("Carrying {} {} to storage", worker.carrying, resource);
    }
    match worker.status {
        WorkerStatus::Idle => "Idle · awaiting a useful order".to_owned(),
        WorkerStatus::Walking => "Walking · routing around the cemetery".to_owned(),
        WorkerStatus::Working => "Working · tools are moving in the clearing".to_owned(),
        WorkerStatus::Carrying => "Carrying · bundle secured".to_owned(),
        WorkerStatus::Hiding => "Guarding · scanning the road".to_owned(),
    }
}
