//! Selection actions presented inside the compact command sheet.

use super::super::components::{compact_virtual_button, virtual_button};
use super::super::{UiAction, UiContext};
use crate::state::{
    BuildingKind, GamePhase, JobKind, PlotStatus, Selection, Technology, UndeadKind,
};
use macroquad::prelude::*;
use macroquad_toolkit::grid::TilePos;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    if ctx.placement.is_some() {
        draw_placement(pointer, actions, sheet);
        return;
    }
    match ctx.session.world.selected {
        Some(Selection::Worker(index)) => draw_worker(ctx, pointer, actions, sheet, index),
        Some(Selection::Grave(index)) => draw_grave(ctx, pointer, actions, sheet, index),
        Some(Selection::Building(index)) => draw_building(ctx, pointer, actions, sheet, index),
        Some(Selection::Necromancer) => draw_necromancer(ctx, pointer, actions, sheet),
        Some(Selection::Ground(tile)) => draw_ground(ctx, pointer, actions, sheet, tile),
        None => draw_empty(sheet),
    }
}

fn draw_placement(pointer: Pointer, actions: &mut Vec<UiAction>, sheet: Rect) {
    draw_text_block(
        "PLACEMENT",
        sheet.x + 16.0,
        sheet.y + 88.0,
        140.0,
        18.0,
        12.0,
        0.0,
        dark::WARNING,
    );
    draw_text_block(
        "Tap an open tile in the world, or cancel placement below.",
        sheet.x + 16.0,
        sheet.y + 112.0,
        sheet.w - 32.0,
        30.0,
        14.0,
        3.0,
        dark::TEXT,
    );
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 164.0, sheet.w - 32.0, 48.0),
        "Cancel placement",
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::CancelPlacement);
    }
}

fn draw_ground(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    tile: TilePos,
) {
    draw_text_block(
        &format!("GROUND · tile {}, {}", tile.x + 1, tile.y + 1),
        sheet.x + 16.0,
        sheet.y + 90.0,
        sheet.w - 32.0,
        22.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    if let Some(summary) =
        crate::engine::districts::tile_summary(ctx.session, &ctx.data.config.district_rules, tile)
    {
        let summary_height = if summary.contains('\n') { 82.0 } else { 52.0 };
        draw_text_block(
            &summary,
            sheet.x + 16.0,
            sheet.y + 122.0,
            sheet.w - 32.0,
            summary_height,
            14.0,
            3.0,
            dark::ACCENT,
        );
        let button = Rect::new(
            sheet.x + 16.0,
            sheet.y + 122.0 + summary_height + 8.0,
            sheet.w - 32.0,
            44.0,
        );
        let (button_label, destination) = if ctx
            .session
            .research
            .is_unlocked(Technology::DomainStewardship)
        {
            ("Open Domain rules", super::super::Panel::Domain)
        } else {
            ("Open Research", super::super::Panel::Research)
        };
        if virtual_button(button, button_label, true, ButtonTone::Secondary, pointer) {
            actions.push(UiAction::TogglePanel(destination));
        }
    } else {
        draw_text_block(
            "Tap a worker, grave, or structure in the world for its actions.",
            sheet.x + 16.0,
            sheet.y + 122.0,
            sheet.w - 32.0,
            36.0,
            14.0,
            3.0,
            dark::TEXT_DIM,
        );
    }
}

fn draw_empty(sheet: Rect) {
    draw_text_block(
        "Tap an actor or structure in the world to give an order.",
        sheet.x + 16.0,
        sheet.y + 100.0,
        sheet.w - 32.0,
        24.0,
        15.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn draw_worker(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    index: usize,
) {
    let Some(worker) = ctx.session.workforce.workers.get(index) else {
        return;
    };
    draw_text_block(
        &format!("{} · {}", worker.name, worker.assignment.label()),
        sheet.x + 16.0,
        sheet.y + 84.0,
        sheet.w - 32.0,
        22.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    if let Some(route_summary) = super::super::world_feedback::worker_route_summary(ctx, worker) {
        let route_failed = route_summary.contains("NO ROUTE");
        let route_summary = super::super::world_feedback::worker_district_hint(ctx, worker)
            .map_or(route_summary.clone(), |hint| {
                format!("{hint} · {route_summary}")
            });
        draw_text_block(
            &route_summary,
            sheet.x + 16.0,
            sheet.y + 104.0,
            sheet.w - 32.0,
            16.0,
            11.0,
            0.0,
            if route_failed {
                dark::WARNING
            } else {
                dark::ACCENT
            },
        );
    }
    let jobs = [
        JobKind::Dig,
        JobKind::Haul,
        JobKind::Guard,
        JobKind::Wood,
        JobKind::Build,
        JobKind::Refine,
    ];
    let gap = 6.0;
    let button_width = (sheet.w - 32.0 - gap * 2.0) / 3.0;
    for (job_index, job) in jobs.into_iter().enumerate() {
        let button = Rect::new(
            sheet.x + 16.0 + (job_index % 3) as f32 * (button_width + gap),
            sheet.y + 120.0 + (job_index / 3) as f32 * 50.0,
            button_width,
            44.0,
        );
        let enabled = ctx.session.phase == GamePhase::Playing
            && (job != JobKind::Refine || ctx.session.has_building(BuildingKind::OssuaryKiln));
        if compact_virtual_button(
            button,
            job.label(),
            enabled,
            ButtonTone::Primary,
            13.0,
            pointer,
        ) {
            actions.push(UiAction::AssignJob(job));
        }
    }
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 218.0, sheet.w - 32.0, 44.0),
        if worker.priority_mode {
            "Direct orders"
        } else {
            "Repeat priorities"
        },
        ctx.session
            .research
            .is_unlocked(Technology::BindingRoutines)
            && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::ToggleAutomation);
    }
}

fn draw_grave(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    index: usize,
) {
    let Some(plot) = ctx.session.world.plots.get(index) else {
        return;
    };
    draw_text_block(
        &format!("GRAVE {:02} · {}", index + 1, plot_status(plot.status)),
        sheet.x + 16.0,
        sheet.y + 86.0,
        sheet.w - 32.0,
        22.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    let skeleton = ctx
        .data
        .undead
        .get(UndeadKind::Skeleton.id())
        .expect("validated skeleton");
    let button_width = (sheet.w - 42.0) / 2.0;
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 120.0, button_width, 48.0),
        "Assign Dig",
        plot.status == PlotStatus::Ready && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::AssignJob(JobKind::Dig));
    }
    if virtual_button(
        Rect::new(
            sheet.x + 26.0 + button_width,
            sheet.y + 120.0,
            button_width,
            48.0,
        ),
        &format!("Raise · B{} M{}", skeleton.bones_cost, skeleton.mana_cost),
        plot.status == PlotStatus::Dug
            && ctx.session.economy.bones >= skeleton.bones_cost
            && ctx.session.economy.mana >= skeleton.mana_cost
            && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::Raise(UndeadKind::Skeleton));
    }
}

fn draw_building(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    index: usize,
) {
    let Some(building) = ctx.session.world.buildings.get(index) else {
        return;
    };
    draw_text_block(
        ctx.data
            .buildings
            .get(building.kind.id())
            .map_or(building.kind.id(), |def| def.name.as_str()),
        sheet.x + 16.0,
        sheet.y + 86.0,
        sheet.w - 32.0,
        22.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    match building.kind {
        BuildingKind::OssuaryKiln => draw_kiln(ctx, pointer, actions, sheet),
        BuildingKind::WorkShed => {
            if virtual_button(
                Rect::new(sheet.x + 16.0, sheet.y + 120.0, sheet.w - 32.0, 48.0),
                "Study Binding Routines",
                building.complete && ctx.session.research.can_start(Technology::BindingRoutines),
                ButtonTone::Positive,
                pointer,
            ) {
                actions.push(UiAction::StartResearch(Technology::BindingRoutines));
            }
        }
        BuildingKind::GraveLantern => {
            draw_text_block(
                "The lantern softens suspicion around every grave.",
                sheet.x + 16.0,
                sheet.y + 122.0,
                sheet.w - 32.0,
                30.0,
                14.0,
                3.0,
                dark::TEXT_DIM,
            );
        }
    }
}

fn draw_kiln(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>, sheet: Rect) {
    let Some(recipe) = ctx
        .data
        .buildings
        .get(BuildingKind::OssuaryKiln.id())
        .and_then(|def| def.production.as_ref())
    else {
        return;
    };
    let queued = ctx.session.progress.production_queue;
    let active = ctx.session.progress.production.is_some();
    draw_text_block(
        &format!(
            "Ward cycle · stored {} · reserved {}/{}",
            ctx.session.economy.ward_charges,
            queued,
            crate::engine::progression::MAX_PRODUCTION_QUEUE
        ),
        sheet.x + 16.0,
        sheet.y + 106.0,
        sheet.w - 32.0,
        20.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 130.0, sheet.w - 32.0, 44.0),
        if active {
            "Queue ward cycle"
        } else {
            "Load kiln"
        },
        ctx.session.phase == GamePhase::Playing
            && queued < crate::engine::progression::MAX_PRODUCTION_QUEUE
            && ctx.session.economy.bones >= recipe.bones_cost
            && ctx.session.economy.wood >= recipe.wood_cost,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::StartProduction(BuildingKind::OssuaryKiln));
    }
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 176.0, sheet.w - 32.0, 44.0),
        if queued > 0 {
            "Cancel reserved cycle"
        } else {
            "No reserved cycle"
        },
        queued > 0 && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Warning,
        pointer,
    ) {
        actions.push(UiAction::CancelProduction(BuildingKind::OssuaryKiln));
    }
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 222.0, sheet.w - 32.0, 44.0),
        "Spend ward charge",
        ctx.session.economy.ward_charges > 0 && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::UseWardCharge);
    }
}

fn draw_necromancer(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    draw_text_block(
        "NECROMANCER",
        sheet.x + 16.0,
        sheet.y + 88.0,
        sheet.w - 32.0,
        22.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    let moving = ctx.session.world.necromancer_destination.is_some();
    draw_text_block(
        if moving {
            "Walking toward a marked destination."
        } else {
            "Tap the clearing to move and select a new destination."
        },
        sheet.x + 16.0,
        sheet.y + 120.0,
        sheet.w - 32.0,
        32.0,
        14.0,
        3.0,
        dark::TEXT_DIM,
    );
    if moving
        && virtual_button(
            Rect::new(sheet.x + 16.0, sheet.y + 172.0, sheet.w - 32.0, 44.0),
            "Cancel movement",
            ctx.session.phase == GamePhase::Playing,
            ButtonTone::Secondary,
            pointer,
        )
    {
        actions.push(UiAction::MoveNecromancer(
            ctx.session.world.necromancer_position,
        ));
    }
}

fn plot_status(status: PlotStatus) -> &'static str {
    match status {
        PlotStatus::Locked => "locked",
        PlotStatus::Ready => "ready",
        PlotStatus::Digging => "digging",
        PlotStatus::Dug => "open",
    }
}
