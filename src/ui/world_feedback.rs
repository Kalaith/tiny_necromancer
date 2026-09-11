//! World markers for active routes, loose resources, and work destinations.

use super::components::GridView;
use super::UiContext;
use crate::engine::{districts, jobs, navigation};
use crate::state::{JobKind, PlotStatus, ResourceKind, Selection, Worker, WorkerStatus};
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
    if ctx.domain_overlays.routes {
        draw_route_legend(ctx);
    }
    for (index, worker) in ctx.session.workforce.workers.iter().enumerate() {
        if ctx.domain_overlays.routes
            || ctx.session.world.selected == Some(Selection::Worker(index))
        {
            if let Some(destination) = jobs::destination_for_worker(ctx.session, worker) {
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

fn draw_route_legend(ctx: &UiContext<'_>) {
    let rect = if ctx.layout.compact {
        Rect::new(206.0, 146.0, 252.0, 26.0)
    } else {
        Rect::new(24.0, 92.0, 306.0, 28.0)
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.03, 0.06, 0.045, 0.90))
            .with_border(1.0, dark::ACCENT.with_alpha(0.54)),
    );
    draw_text_block(
        "ROUTE · dots = steps · amber = blocked",
        rect.x + 8.0,
        rect.y + 6.0,
        rect.w - 16.0,
        16.0,
        11.0,
        0.0,
        dark::TEXT,
    );
}

fn draw_route_hint(ctx: &UiContext<'_>, view: &GridView, worker: &Worker, destination: TilePos) {
    let worker_id = worker.id;
    let start = worker.position;
    let actor = ctx.motions.worker(worker_id).map_or_else(
        || view.tile_rect(start).center(),
        |motion| view.actor_center(motion.visual_position()),
    );
    let target_center = view.tile_rect(destination).center();
    let route = navigation::plan_route(ctx.session, start, destination);
    if let Ok(route) = &route {
        let mut previous = actor;
        for (index, tile) in route.steps().iter().enumerate().skip(1) {
            let point = view.tile_rect(*tile).center();
            draw_line(
                previous.x,
                previous.y,
                point.x,
                point.y,
                2.0,
                dark::ACCENT.with_alpha(if index == 1 { 0.58 } else { 0.42 }),
            );
            draw_circle(point.x, point.y, 3.0, dark::ACCENT.with_alpha(0.72));
            previous = point;
        }
    } else {
        draw_line(
            actor.x,
            actor.y,
            target_center.x,
            target_center.y,
            2.0,
            dark::WARNING.with_alpha(0.48),
        );
    }

    let route_color = if route.is_ok() {
        dark::ACCENT
    } else {
        dark::WARNING
    };
    let target = view.tile_rect(destination).inset(view.tile_size() * 0.25);
    draw_rectangle_lines(target.x, target.y, target.w, target.h, 2.0, route_color);
    let route_summary = match &route {
        Ok(route) => format!("ROUTE · {} steps", route.step_count()),
        Err(failure) => format!("NO ROUTE · {}", failure.label()),
    };
    let route_label = if ctx.domain_overlays.routes {
        district_rule_hint(ctx, worker, destination).map_or_else(
            || {
                format!(
                    "{} · {} · {route_summary}",
                    worker.name,
                    worker.assignment.label()
                )
            },
            |hint| {
                format!(
                    "{} · {} · {hint} · {route_summary}",
                    worker.name,
                    worker.assignment.label()
                )
            },
        )
    } else {
        route_summary
    };
    draw_text_block(
        &route_label,
        target.x - 44.0,
        target.y - view.tile_size() * 0.38,
        220.0,
        30.0,
        10.0,
        2.0,
        route_color,
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
) -> Option<String> {
    match worker.assignment {
        JobKind::Dig | JobKind::Wood => {
            let multiplier = districts::work_speed_multiplier(
                ctx.session,
                &ctx.data.config.district_rules,
                destination,
            );
            (multiplier > 1.0)
                .then(|| format!("Work +{:.0}% at target", (multiplier - 1.0) * 100.0))
        }
        JobKind::Haul if worker.carrying > 0 => {
            let bonus = districts::haul_capacity_bonus(
                ctx.session,
                &ctx.data.config.district_rules,
                destination,
            );
            (bonus > 0).then(|| format!("Storage +{bonus} at drop"))
        }
        JobKind::Guard => {
            let multiplier = districts::guard_mitigation_multiplier(
                ctx.session,
                &ctx.data.config.district_rules,
                worker.position,
            );
            (multiplier > 1.0)
                .then(|| format!("Patrol +{:.0}% at post", (multiplier - 1.0) * 100.0))
        }
        _ => None,
    }
}

pub(super) fn worker_district_hint(ctx: &UiContext<'_>, worker: &Worker) -> Option<String> {
    jobs::destination_for_worker(ctx.session, worker)
        .and_then(|destination| district_rule_hint(ctx, worker, destination))
}

pub(super) fn worker_route_summary(ctx: &UiContext<'_>, worker: &Worker) -> Option<String> {
    let destination = jobs::destination_for_worker(ctx.session, worker)?;
    let summary = match navigation::plan_route(ctx.session, worker.position, destination) {
        Ok(route) if route.step_count() == 0 => "AT DESTINATION".to_owned(),
        Ok(route) => format!("ROUTE · {} steps", route.step_count()),
        Err(failure) => format!("NO ROUTE · {}", failure.label()),
    };
    let summary = if worker.assignment == JobKind::Haul
        && worker.carrying > 0
        && ctx
            .session
            .world
            .zone_contains(crate::state::ZoneKind::Storage, destination)
    {
        format!(
            "DROP {},{} · {summary}",
            destination.x + 1,
            destination.y + 1
        )
    } else if worker.assignment == JobKind::Wood
        && ctx
            .session
            .world
            .zone_contains(crate::state::ZoneKind::Work, destination)
        && ctx.session.world.forest_tiles.contains(&destination)
    {
        format!(
            "WOOD {},{} · {summary}",
            destination.x + 1,
            destination.y + 1
        )
    } else if worker.assignment == JobKind::Dig
        && ctx
            .session
            .world
            .zone_contains(crate::state::ZoneKind::Work, destination)
    {
        format!(
            "DIG {},{} · {summary}",
            destination.x + 1,
            destination.y + 1
        )
    } else {
        summary
    };
    Some(
        jobs::patrol_post_number(ctx.session, worker)
            .map_or(summary.clone(), |post| format!("POST P{post} · {summary}")),
    )
}

pub(super) fn route_counts(ctx: &UiContext<'_>) -> (usize, usize) {
    let mut clear = 0;
    let mut total = 0;
    for worker in &ctx.session.workforce.workers {
        let Some(destination) = jobs::destination_for_worker(ctx.session, worker) else {
            continue;
        };
        total += 1;
        if navigation::plan_route(ctx.session, worker.position, destination).is_ok() {
            clear += 1;
        }
    }
    (clear, total)
}

pub(super) fn worker_idle_reason(ctx: &UiContext<'_>, worker: &Worker) -> String {
    if worker.status != WorkerStatus::Idle {
        return "Active in the clearing".to_owned();
    }
    if let Some(destination) = jobs::destination_for_worker(ctx.session, worker) {
        if let Err(failure) = navigation::plan_route(ctx.session, worker.position, destination) {
            return format!("No route · {}", failure.label());
        }
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
                "Waiting for a route to the next grave".to_owned()
            } else {
                "No grave available".to_owned()
            }
        }
        JobKind::Haul => {
            if worker.carrying > 0 {
                "Carrying a bundle to storage".to_owned()
            } else if ctx.session.economy.loose_bones > 0 || ctx.session.economy.loose_wood > 0 {
                "Waiting for a route to loose material".to_owned()
            } else {
                "No loose material".to_owned()
            }
        }
        JobKind::Guard => "Waiting for a patrol route".to_owned(),
        JobKind::Wood => {
            if ctx.session.world.forest_tiles.is_empty() {
                "No forest edge available".to_owned()
            } else {
                "Waiting for a route to the forest".to_owned()
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
                "Waiting for a route to construction".to_owned()
            } else {
                "No structure under construction".to_owned()
            }
        }
        JobKind::Refine => {
            if ctx.session.progress.production.is_some() {
                "Waiting for a route to the kiln".to_owned()
            } else {
                "No kiln cycle loaded".to_owned()
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
