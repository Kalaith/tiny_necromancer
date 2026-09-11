//! Contextual controls for a selected grave and its nearby worker loop.

use super::super::components::virtual_button;
use super::super::{UiAction, UiContext};
use crate::state::{GamePhase, JobKind, PlotStatus, UndeadKind};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    index: usize,
) {
    let Some(plot) = ctx.session.world.plots.get(index) else {
        return;
    };
    draw_text_block(
        &format!("Grave {:02}", index + 1),
        panel.x + 18.0,
        panel.y + 54.0,
        panel.w - 36.0,
        28.0,
        24.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    let state = match plot.status {
        PlotStatus::Ready => "Undisturbed",
        PlotStatus::Digging => "Excavation underway",
        PlotStatus::Dug => "Open excavation",
        PlotStatus::Locked => "Outside the clearing",
    };
    draw_text_block(
        state,
        panel.x + 18.0,
        panel.y + 94.0,
        panel.w - 36.0,
        22.0,
        15.0,
        0.0,
        if plot.status == PlotStatus::Digging {
            dark::WARNING
        } else {
            dark::TEXT
        },
    );
    if plot.status == PlotStatus::Digging {
        progress_bar(
            panel.x + 18.0,
            panel.y + 128.0,
            panel.w - 36.0,
            12.0,
            plot.progress,
            ctx.data.jobs.get("dig").map_or(8.0, |job| job.work_seconds),
            dark::WARNING,
        );
    }
    draw_text_block(
        if plot.status == PlotStatus::Dug {
            "The earth is yielding bones and the chance of a corpse remnant."
        } else {
            "A worker can dig here. Need shed timber? Assign Gather Wood, then Haul the loose wood into the stockpile."
        },
        panel.x + 18.0,
        panel.y + 158.0,
        panel.w - 36.0,
        54.0,
        14.0,
        4.0,
        dark::TEXT_DIM,
    );
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 244.0, panel.w - 36.0, 44.0),
        "Assign selected worker · Dig",
        plot.status == PlotStatus::Ready && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::AssignJob(JobKind::Dig));
    }
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 300.0, panel.w - 36.0, 44.0),
        "Gather Wood",
        ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::AssignJob(JobKind::Wood));
    }
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 356.0, panel.w - 36.0, 44.0),
        "Haul Loose Material",
        (ctx.session.economy.loose_bones > 0 || ctx.session.economy.loose_wood > 0)
            && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::AssignJob(JobKind::Haul));
    }
    let skeleton_def = ctx
        .data
        .undead
        .get(UndeadKind::Skeleton.id())
        .expect("validated skeleton recipe");
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 412.0, panel.w - 36.0, 44.0),
        &format!(
            "Raise skeleton · B{} M{}",
            skeleton_def.bones_cost, skeleton_def.mana_cost
        ),
        plot.status == PlotStatus::Dug
            && ctx.session.economy.bones >= skeleton_def.bones_cost
            && ctx.session.economy.mana >= skeleton_def.mana_cost,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::Raise(UndeadKind::Skeleton));
    }
}
