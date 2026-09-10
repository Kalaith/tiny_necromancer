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
    } else {
        format!("Load kiln · B{} W{}", recipe.bones_cost, recipe.wood_cost)
    };
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 232.0, panel.w - 36.0, 44.0),
        &label,
        ctx.session.phase == GamePhase::Playing
            && queued < crate::engine::progression::MAX_PRODUCTION_QUEUE
            && ctx.session.economy.bones >= recipe.bones_cost
            && ctx.session.economy.wood >= recipe.wood_cost,
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
    } else {
        "No reserved cycle".to_owned()
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
    draw_text_block(
        &format!(
            "Ward charges · {} · stored {} · queued {}/{}",
            recipe.effect_text,
            ctx.session.economy.ward_charges,
            queued,
            crate::engine::progression::MAX_PRODUCTION_QUEUE
        ),
        panel.x + 18.0,
        panel.y + 402.0,
        panel.w - 36.0,
        48.0,
        14.0,
        4.0,
        dark::TEXT_DIM,
    );
}

pub(super) fn is_kiln(building: &Building) -> bool {
    building.kind == BuildingKind::OssuaryKiln && building.complete
}
