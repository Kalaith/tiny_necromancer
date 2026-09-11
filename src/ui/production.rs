//! Kiln recipe selection and production controls for building inspectors.

use super::components::{compact_virtual_button, virtual_button};
use super::{UiAction, UiContext};
use crate::engine::progression;
use crate::state::{Building, BuildingKind, GamePhase, ProductionRecipeKind};
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
    let recipe_kind = selected_recipe_kind(ctx, building.kind);
    let Some(recipes) = progression::production_recipes(ctx.data, building.kind) else {
        return;
    };
    let Some(recipe) = recipes.iter().find(|recipe| recipe.kind == recipe_kind) else {
        return;
    };
    let active = active_order(ctx, building.kind).is_some();
    let queued = ctx.session.progress.production_queue;
    let can_change_recipe = !active && ctx.session.phase == GamePhase::Playing;

    for (index, option) in recipes.iter().enumerate() {
        let width = (panel.w - 44.0) * 0.5;
        let x = panel.x + 18.0 + index as f32 * (width + 8.0);
        if virtual_button(
            Rect::new(x, panel.y + 232.0, width, 44.0),
            &format!(
                "{} B{}/W{}",
                option.kind.short_label(),
                option.bones_cost,
                option.wood_cost
            ),
            can_change_recipe,
            if option.kind == recipe_kind {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            pointer,
        ) {
            actions.push(UiAction::SelectProductionRecipe(building.kind, option.kind));
        }
    }
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 286.0, panel.w - 36.0, 44.0),
        &production_button_label(
            recipe,
            active,
            queued,
            ctx.session.economy.bones,
            ctx.session.economy.wood,
        ),
        ctx.session.phase == GamePhase::Playing
            && (!active || queued < progression::MAX_PRODUCTION_QUEUE)
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
            "Cancel reserved · +B{} W{}",
            recipe.bones_cost, recipe.wood_cost
        )
    } else if active {
        "No reserved cycle · active".to_owned()
    } else {
        "Load a cycle first".to_owned()
    };
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 340.0, panel.w - 36.0, 44.0),
        &cancel_label,
        queued > 0 && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Warning,
        pointer,
    ) {
        actions.push(UiAction::CancelProduction(building.kind));
    }
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 394.0, panel.w - 36.0, 44.0),
        "Spend ward charge · -8",
        ctx.session.economy.ward_charges > 0 && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::UseWardCharge);
    }
    let input_status = active_order(ctx, building.kind).map_or_else(
        || "Inputs · no cycle requested".to_owned(),
        |order| {
            if order.bones_remaining > 0 || order.wood_remaining > 0 {
                format!(
                    "Inputs waiting · B{} W{} · Haul",
                    order.bones_remaining, order.wood_remaining
                )
            } else {
                format!("Ready · {}", order.recipe.label())
            }
        },
    );
    draw_text_block(
        &input_status,
        panel.x + 18.0,
        panel.y + 448.0,
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
}

pub(super) fn draw_compact_kiln(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    let recipe_kind = selected_recipe_kind(ctx, BuildingKind::OssuaryKiln);
    let Some(recipes) = progression::production_recipes(ctx.data, BuildingKind::OssuaryKiln) else {
        return;
    };
    let Some(recipe) = recipes.iter().find(|recipe| recipe.kind == recipe_kind) else {
        return;
    };
    let active = active_order(ctx, BuildingKind::OssuaryKiln).is_some();
    let queued = ctx.session.progress.production_queue;
    let can_change_recipe = !active && ctx.session.phase == GamePhase::Playing;
    for (index, option) in recipes.iter().enumerate() {
        let width = (sheet.w - 40.0) * 0.5;
        let x = sheet.x + 16.0 + index as f32 * (width + 8.0);
        if compact_virtual_button(
            Rect::new(x, sheet.y + 148.0, width, 44.0),
            &format!(
                "{} · B{} W{}",
                option.name, option.bones_cost, option.wood_cost
            ),
            can_change_recipe,
            if option.kind == recipe_kind {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            11.0,
            pointer,
        ) {
            actions.push(UiAction::SelectProductionRecipe(
                BuildingKind::OssuaryKiln,
                option.kind,
            ));
        }
    }
    if compact_virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 192.0, sheet.w - 32.0, 44.0),
        &production_button_label(
            recipe,
            active,
            queued,
            ctx.session.economy.bones,
            ctx.session.economy.wood,
        ),
        ctx.session.phase == GamePhase::Playing
            && (!active || queued < progression::MAX_PRODUCTION_QUEUE)
            && (!active
                || (ctx.session.economy.bones >= recipe.bones_cost
                    && ctx.session.economy.wood >= recipe.wood_cost)),
        ButtonTone::Positive,
        12.0,
        pointer,
    ) {
        actions.push(UiAction::StartProduction(BuildingKind::OssuaryKiln));
    }
    if compact_virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 236.0, sheet.w - 32.0, 44.0),
        if queued > 0 {
            "Cancel reserved cycle"
        } else {
            "No reserved cycle"
        },
        queued > 0 && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Warning,
        12.0,
        pointer,
    ) {
        actions.push(UiAction::CancelProduction(BuildingKind::OssuaryKiln));
    }
    if compact_virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 280.0, sheet.w - 32.0, 44.0),
        "Spend ward charge",
        ctx.session.economy.ward_charges > 0 && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        12.0,
        pointer,
    ) {
        actions.push(UiAction::UseWardCharge);
    }
}

fn active_order<'a>(
    ctx: &'a UiContext<'_>,
    kind: BuildingKind,
) -> Option<&'a crate::state::ProductionOrder> {
    ctx.session
        .progress
        .production
        .as_ref()
        .filter(|order| order.building == kind)
}

fn selected_recipe_kind(ctx: &UiContext<'_>, kind: BuildingKind) -> ProductionRecipeKind {
    active_order(ctx, kind).map_or(ctx.session.progress.production_recipe, |order| order.recipe)
}

fn production_button_label(
    recipe: &crate::data::ProductionRecipeDef,
    active: bool,
    queued: usize,
    bones: i32,
    wood: i32,
) -> String {
    if active {
        if queued < progression::MAX_PRODUCTION_QUEUE {
            format!(
                "Queue {} · B{} W{}",
                recipe.name, recipe.bones_cost, recipe.wood_cost
            )
        } else {
            "Kiln queue full".to_owned()
        }
    } else {
        let verb = if bones >= recipe.bones_cost && wood >= recipe.wood_cost {
            "Load"
        } else {
            "Request"
        };
        format!(
            "{} {} · B{} W{}",
            verb, recipe.name, recipe.bones_cost, recipe.wood_cost
        )
    }
}

pub(super) fn is_kiln(building: &Building) -> bool {
    building.kind == BuildingKind::OssuaryKiln && building.complete
}
