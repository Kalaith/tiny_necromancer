//! Kiln production controls for the contextual building inspector.

use super::components::virtual_button;
use super::{UiAction, UiContext};
use crate::state::{Building, BuildingKind, GamePhase};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_kiln_inspector(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    building: &Building,
) {
    let recipe = ctx
        .data
        .buildings
        .get(building.kind.id())
        .and_then(|def| def.production.as_ref());
    let Some(recipe) = recipe else {
        return;
    };
    let active = ctx
        .session
        .progress
        .production
        .as_ref()
        .is_some_and(|order| order.building == building.kind);
    let queued = ctx.session.progress.production_queue;
    let label = if active {
        if queued < crate::engine::progression::MAX_PRODUCTION_QUEUE {
            format!(
                "Queue ward cycle · B{} W{}",
                recipe.bones_cost, recipe.wood_cost
            )
        } else {
            "Ward queue full".to_owned()
        }
    } else if ctx.session.economy.bones >= recipe.bones_cost
        && ctx.session.economy.wood >= recipe.wood_cost
    {
        format!("Load kiln · B{} W{}", recipe.bones_cost, recipe.wood_cost)
    } else {
        format!(
            "Request kiln supply · B{} W{}",
            recipe.bones_cost, recipe.wood_cost
        )
    };
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 232.0, panel.w - 36.0, 44.0),
        &label,
        ctx.session.phase == GamePhase::Playing
            && (!active || queued < crate::engine::progression::MAX_PRODUCTION_QUEUE)
            && (!active
                || (ctx.session.economy.bones >= recipe.bones_cost
                    && ctx.session.economy.wood >= recipe.wood_cost)),
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::StartProduction(building.kind));
    }
    let cancel_label = if queued > 0 {
        format!(
            "Cancel reserved cycle · +B{} W{}",
            recipe.bones_cost, recipe.wood_cost
        )
    } else if active {
        "No reserved cycle · active work stays".to_owned()
    } else {
        "Load a cycle first".to_owned()
    };
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 286.0, panel.w - 36.0, 44.0),
        &cancel_label,
        queued > 0 && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Warning,
        pointer,
    ) {
        actions.push(UiAction::CancelProduction(building.kind));
    }
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 340.0, panel.w - 36.0, 44.0),
        "Spend ward charge · -8 suspicion",
        ctx.session.economy.ward_charges > 0 && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::UseWardCharge);
    }
    let input_status = ctx
        .session
        .progress
        .production
        .as_ref()
        .filter(|order| order.building == building.kind)
        .map_or_else(
            || "Inputs · no cycle requested".to_owned(),
            |order| {
                if order.bones_remaining > 0 || order.wood_remaining > 0 {
                    format!(
                        "Inputs waiting · B{} W{} · assign Haul",
                        order.bones_remaining, order.wood_remaining
                    )
                } else {
                    "Inputs ready · B0 W0".to_owned()
                }
            },
        );
    draw_text_block(
        &input_status,
        panel.x + 18.0,
        panel.y + 398.0,
        panel.w - 36.0,
        20.0,
        13.0,
        0.0,
        if input_status.contains("waiting") {
            dark::WARNING
        } else {
            dark::ACCENT
        },
    );
    draw_text_block(
        &format!(
            "Ward charges · {} · stored {} · reserved {}/{}",
            recipe.effect_text,
            ctx.session.economy.ward_charges,
            queued,
            crate::engine::progression::MAX_PRODUCTION_QUEUE
        ),
        panel.x + 18.0,
        panel.y + 420.0,
        panel.w - 36.0,
        32.0,
        12.0,
        3.0,
        dark::TEXT_DIM,
    );
}

pub(super) fn is_kiln(building: &Building) -> bool {
    building.kind == BuildingKind::OssuaryKiln && building.complete
}
