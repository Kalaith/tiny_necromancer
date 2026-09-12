//! Route overlays and destination progress indicators.

use super::*;

pub(super) fn draw_route_legend(ctx: &UiContext<'_>) {
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

pub(super) fn draw_route_hint(
    ctx: &UiContext<'_>,
    view: &GridView,
    worker: &Worker,
    destination: TilePos,
) {
    let worker_id = worker.id;
    let start = worker.position;
    let actor = ctx.motions.worker(worker_id).map_or_else(
        || view.tile_rect(start).center(),
        |motion| view.actor_center(motion.visual_position()),
    );
    let target_center = view.tile_rect(destination).center();
    let route = navigation::plan_route(ctx.session, start, destination);
    draw_route_path(view, actor, target_center, &route);
    draw_route_label(ctx, view, worker, destination, &route);
    draw_route_progress(ctx, view, destination, worker_id);
}

fn draw_route_path(
    view: &GridView,
    actor: Vec2,
    target_center: Vec2,
    route: &Result<navigation::RoutePlan, navigation::RouteFailure>,
) {
    if let Ok(route) = route {
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
}

fn draw_route_label(
    ctx: &UiContext<'_>,
    view: &GridView,
    worker: &Worker,
    destination: TilePos,
    route: &Result<navigation::RoutePlan, navigation::RouteFailure>,
) {
    let route_color = if route.is_ok() {
        dark::ACCENT
    } else {
        dark::WARNING
    };
    let target = view.tile_rect(destination).inset(view.tile_size() * 0.25);
    draw_rectangle_lines(target.x, target.y, target.w, target.h, 2.0, route_color);
    let route_summary = match route {
        Ok(route) => format!("ROUTE · {} steps", route.step_count()),
        Err(failure) => format!("NO ROUTE · {}", failure.label()),
    };
    let route_label = if ctx.domain_overlays.routes {
        super::district_rule_hint(ctx, worker, destination).map_or_else(
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
}

fn draw_route_progress(ctx: &UiContext<'_>, view: &GridView, destination: TilePos, worker_id: u32) {
    let target = view.tile_rect(destination).inset(view.tile_size() * 0.25);
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
