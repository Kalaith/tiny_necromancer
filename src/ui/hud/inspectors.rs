//! Contextual worker and building inspector panels.

use super::*;
use crate::state::{Building, Worker};

pub(super) fn draw_worker_inspector(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    index: usize,
) {
    let Some(worker) = ctx.session.workforce.workers.get(index) else {
        return;
    };
    draw_worker_details(ctx, worker, panel, index);
    draw_worker_actions(ctx, worker, pointer, actions, panel);
}

fn draw_worker_details(ctx: &UiContext<'_>, worker: &Worker, panel: Rect, index: usize) {
    draw_text_block(
        &worker.name,
        panel.x + 18.0,
        panel.y + 54.0,
        panel.w - 36.0,
        28.0,
        24.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        &format!(
            "{} · {}",
            if worker.kind == UndeadKind::BruteSkeleton {
                "Brute skeleton"
            } else {
                "Skeleton worker"
            },
            status_label(worker.status)
        ),
        panel.x + 18.0,
        panel.y + 94.0,
        panel.w - 36.0,
        22.0,
        15.0,
        0.0,
        dark::ACCENT,
    );
    let district_hint = crate::ui::world_feedback::worker_district_hint(ctx, worker);
    let priority_route_hint = crate::ui::world_feedback::worker_priority_route_hint(ctx, index);
    let route_gap_hint = crate::engine::districts::route_gap_district(ctx.session, index)
        .map_or_else(String::new, |kind| format!(" · {} route gap", kind.label()));
    let job_detail = format!(
        "Current job: {}{}{}{}{}",
        worker.assignment.label(),
        if worker.priority_mode {
            " · priority mode"
        } else {
            ""
        },
        district_hint.map_or_else(String::new, |hint| format!(" · {hint}")),
        route_gap_hint,
        priority_route_hint.map_or_else(String::new, |hint| format!(" · {hint}")),
    );
    draw_text_block(
        &job_detail,
        panel.x + 18.0,
        panel.y + 132.0,
        panel.w - 36.0,
        22.0,
        14.0,
        0.0,
        dark::TEXT,
    );
    let activity = if worker.status == WorkerStatus::Idle {
        crate::ui::world_feedback::worker_idle_reason(ctx, worker).to_owned()
    } else {
        crate::ui::world_feedback::worker_activity_detail(worker)
    };
    draw_text_block(
        &activity,
        panel.x + 18.0,
        panel.y + 164.0,
        panel.w - 36.0,
        44.0,
        14.0,
        4.0,
        if worker.status == WorkerStatus::Idle {
            dark::WARNING
        } else {
            dark::TEXT_DIM
        },
    );
    let destination = crate::ui::world_feedback::worker_destination_label(ctx, worker);
    let route_summary = crate::ui::world_feedback::worker_route_summary(ctx, worker);
    let destination_detail = route_summary.map_or(destination.clone(), |summary| {
        format!("{destination} · {summary}")
    });
    draw_text_block(
        &format!("Destination · {destination_detail}"),
        panel.x + 18.0,
        panel.y + 208.0,
        panel.w - 36.0,
        18.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn draw_worker_actions(
    ctx: &UiContext<'_>,
    worker: &Worker,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
) {
    for (idx, job) in [
        JobKind::Dig,
        JobKind::Haul,
        JobKind::Guard,
        JobKind::Wood,
        JobKind::Build,
        JobKind::Refine,
    ]
    .into_iter()
    .enumerate()
    {
        let button = Rect::new(
            panel.x + 18.0 + (idx % 3) as f32 * 91.0,
            panel.y + 232.0 + (idx / 3) as f32 * 50.0,
            84.0,
            44.0,
        );
        if virtual_button(
            button,
            job.label(),
            ctx.session.phase == GamePhase::Playing
                && (job != JobKind::Refine || ctx.session.has_building(BuildingKind::OssuaryKiln)),
            if job == JobKind::Guard {
                ButtonTone::Secondary
            } else {
                ButtonTone::Primary
            },
            pointer,
        ) {
            actions.push(UiAction::AssignJob(job));
        }
    }
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 342.0, panel.w - 36.0, 44.0),
        if worker.priority_mode {
            "Direct orders"
        } else {
            "Repeat priorities"
        },
        ctx.session
            .research
            .is_unlocked(Technology::BindingRoutines),
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::ToggleAutomation);
    }
    if !ctx
        .session
        .research
        .is_unlocked(Technology::BindingRoutines)
    {
        draw_text_block(
            "Restore the shed, then study Binding Routines to repeat this order.",
            panel.x + 18.0,
            panel.y + 402.0,
            panel.w - 36.0,
            36.0,
            12.0,
            4.0,
            dark::TEXT_DIM,
        );
    }
}

pub(super) fn draw_building_inspector(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    index: usize,
) {
    let Some(building) = ctx.session.world.buildings.get(index) else {
        return;
    };
    draw_building_header(ctx, panel, building);
    draw_building_progress(ctx, panel, building);
    draw_text_block(
        &format!(
            "Footprint · {}, {}",
            building.position.x + 1,
            building.position.y + 1
        ),
        panel.x + 18.0,
        panel.y + 160.0,
        panel.w - 172.0,
        22.0,
        14.0,
        0.0,
        dark::TEXT_DIM,
    );
    buildings::draw_desktop_upgrade_preview(ctx, panel, building);
    draw_building_actions(ctx, pointer, actions, panel, building);
    if is_kiln(building) {
        draw_kiln_inspector(ctx, pointer, actions, panel, building);
    }
}

fn draw_building_header(ctx: &UiContext<'_>, panel: Rect, building: &Building) {
    let name = ctx
        .data
        .buildings
        .get(building.kind.id())
        .map_or(building.kind.id(), |def| def.name.as_str());
    draw_text_block(
        name,
        panel.x + 18.0,
        panel.y + 54.0,
        panel.w - 36.0,
        28.0,
        23.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        &buildings::status_label(ctx, building),
        panel.x + 18.0,
        panel.y + 94.0,
        panel.w - 36.0,
        22.0,
        15.0,
        0.0,
        if building.complete {
            dark::POSITIVE
        } else {
            dark::WARNING
        },
    );
}

fn draw_building_progress(ctx: &UiContext<'_>, panel: Rect, building: &Building) {
    if !building.complete {
        progress_bar(
            panel.x + 18.0,
            panel.y + 128.0,
            panel.w - 36.0,
            12.0,
            building.progress,
            ctx.data
                .buildings
                .get(building.kind.id())
                .map_or(10.0, |def| def.build_seconds),
            dark::WARNING,
        );
    }
    if building.complete && building.kind == BuildingKind::OssuaryKiln {
        if let Some(order) = ctx
            .session
            .progress
            .production
            .as_ref()
            .filter(|order| order.building == building.kind)
        {
            let seconds = ctx
                .data
                .buildings
                .get(building.kind.id())
                .and_then(|def| def.production.as_ref())
                .and_then(|production| production.recipe(order.recipe))
                .map_or(8.0, |recipe| recipe.seconds);
            draw_text_block(
                &format!(
                    "Refining {} · reserved {}/{}",
                    order.recipe.label(),
                    ctx.session.progress.production_queue,
                    crate::engine::progression::MAX_PRODUCTION_QUEUE
                ),
                panel.x + 18.0,
                panel.y + 126.0,
                panel.w - 36.0,
                18.0,
                13.0,
                0.0,
                dark::ACCENT,
            );
            progress_bar(
                panel.x + 18.0,
                panel.y + 148.0,
                panel.w - 36.0,
                10.0,
                order.progress,
                seconds,
                dark::ACCENT,
            );
        }
    }
}

fn draw_building_actions(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    building: &Building,
) {
    buildings::draw_desktop_upgrade(ctx, pointer, actions, panel, building);
    buildings::draw_desktop_market_button(ctx, pointer, actions, panel, building);
    if building.kind == BuildingKind::WorkShed
        && building.complete
        && virtual_button(
            Rect::new(panel.x + 18.0, panel.y + 232.0, panel.w - 36.0, 44.0),
            if ctx.session.research.current == Some(Technology::BindingRoutines) {
                "Bindings in progress"
            } else {
                "Study bindings"
            },
            ctx.session.research.can_start(Technology::BindingRoutines),
            ButtonTone::Positive,
            pointer,
        )
    {
        actions.push(UiAction::StartResearch(Technology::BindingRoutines));
    }
    if building.kind == BuildingKind::WorkShed && building.complete {
        draw_text_block(
            "A restored shed is the first research station.",
            panel.x + 18.0,
            panel.y + 296.0,
            panel.w - 36.0,
            42.0,
            14.0,
            4.0,
            dark::TEXT_DIM,
        );
    }
}
