//! Shared structure-upgrade controls for full and compact inspectors.

use super::components::{compact_virtual_button, virtual_button};
use super::{UiAction, UiContext};
use crate::engine::progression;
use crate::state::{Building, GamePhase};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_desktop_upgrade(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    building: &Building,
) {
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 184.0, panel.w - 36.0, 44.0),
        &upgrade_label(ctx, building),
        upgrade_enabled(ctx, building),
        upgrade_tone(ctx, building),
        pointer,
    ) {
        actions.push(UiAction::UpgradeBuilding(building.kind));
    }
}

pub(super) fn draw_compact_upgrade(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    building: &Building,
) {
    let label = if upgrade_enabled(ctx, building) {
        format!(
            "{} · {}",
            upgrade_label(ctx, building),
            compact_upgrade_benefit(ctx, building)
        )
    } else {
        upgrade_label(ctx, building)
    };
    if compact_virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 104.0, sheet.w - 32.0, 44.0),
        &label,
        upgrade_enabled(ctx, building),
        upgrade_tone(ctx, building),
        11.0,
        pointer,
    ) {
        actions.push(UiAction::UpgradeBuilding(building.kind));
    }
}

fn compact_upgrade_benefit(ctx: &UiContext<'_>, building: &Building) -> String {
    let def = ctx
        .data
        .buildings
        .get(building.kind.id())
        .expect("validated building recipe");
    match building.kind {
        crate::state::BuildingKind::WorkShed => format!(
            "+{:.0}% throughput",
            (def.upgrade_speed_multiplier - 1.0) * 100.0
        ),
        crate::state::BuildingKind::GraveLantern => format!(
            "-{:.0}% suspicion",
            (1.0 - def.upgrade_suspicion_multiplier) * 100.0
        ),
        crate::state::BuildingKind::OssuaryKiln => format!(
            "+{:.0}% speed",
            (def.upgrade_speed_multiplier - 1.0) * 100.0
        ),
    }
}

pub(super) fn status_label(ctx: &UiContext<'_>, building: &Building) -> String {
    if building.complete {
        format!(
            "Finished · reinforcement L{}",
            progression::building_level(ctx.session, building.kind)
        )
    } else {
        "Scaffold · construction in progress".to_owned()
    }
}

pub(super) fn draw_desktop_upgrade_preview(ctx: &UiContext<'_>, panel: Rect, building: &Building) {
    if !building.complete
        || progression::building_level(ctx.session, building.kind)
            >= progression::MAX_BUILDING_LEVEL
    {
        return;
    }
    let Some(def) = ctx.data.buildings.get(building.kind.id()) else {
        return;
    };
    let preview = match building.kind {
        crate::state::BuildingKind::WorkShed => format!(
            "+{:.0}% throughput",
            (def.upgrade_speed_multiplier - 1.0) * 100.0
        ),
        crate::state::BuildingKind::GraveLantern => format!(
            "-{:.0}% suspicion",
            (1.0 - def.upgrade_suspicion_multiplier) * 100.0
        ),
        crate::state::BuildingKind::OssuaryKiln => format!(
            "+{:.0}% kiln speed",
            (def.upgrade_speed_multiplier - 1.0) * 100.0
        ),
    };
    draw_text_block(
        &format!("Next · {preview}"),
        panel.x + 154.0,
        panel.y + 163.0,
        panel.w - 172.0,
        18.0,
        10.0,
        0.0,
        dark::ACCENT,
    );
}

pub(super) fn draw_desktop_market_button(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    building: &Building,
) {
    if building.kind != crate::state::BuildingKind::GraveLantern || !building.complete {
        if building.kind == crate::state::BuildingKind::GraveLantern {
            draw_text_block(
                "Finish construction to meet the night broker.",
                panel.x + 18.0,
                panel.y + 232.0,
                panel.w - 36.0,
                40.0,
                13.0,
                3.0,
                dark::TEXT_DIM,
            );
        }
        return;
    }
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 232.0, panel.w - 36.0, 44.0),
        "Open night market",
        ctx.session.phase == GamePhase::Playing,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::TogglePanel(super::Panel::Market));
    }
}

pub(super) fn draw_compact_market_button(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    building: &Building,
) {
    if building.kind != crate::state::BuildingKind::GraveLantern || !building.complete {
        if building.kind == crate::state::BuildingKind::GraveLantern {
            draw_text_block(
                "Finish construction to meet the night broker.",
                sheet.x + 16.0,
                sheet.y + 228.0,
                sheet.w - 32.0,
                36.0,
                12.0,
                3.0,
                dark::TEXT_DIM,
            );
        }
        return;
    }
    if compact_virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 228.0, sheet.w - 32.0, 44.0),
        "Open night market",
        ctx.session.phase == GamePhase::Playing,
        ButtonTone::Positive,
        13.0,
        pointer,
    ) {
        actions.push(UiAction::TogglePanel(super::Panel::Market));
    }
}

fn upgrade_label(ctx: &UiContext<'_>, building: &Building) -> String {
    let level = progression::building_level(ctx.session, building.kind);
    if !building.complete {
        return "Reinforce after construction".to_owned();
    }
    if level >= progression::MAX_BUILDING_LEVEL {
        return format!("Reinforced · level {level}");
    }
    let def = ctx
        .data
        .buildings
        .get(building.kind.id())
        .expect("validated building recipe");
    if !upgrade_materials_available(ctx, building) {
        return format!(
            "Need B{} M{} W{} to reinforce",
            def.upgrade_bones_cost, def.upgrade_mana_cost, def.upgrade_wood_cost
        );
    }
    format!(
        "Reinforce · B{} M{} W{}",
        def.upgrade_bones_cost, def.upgrade_mana_cost, def.upgrade_wood_cost
    )
}

fn upgrade_enabled(ctx: &UiContext<'_>, building: &Building) -> bool {
    let level = progression::building_level(ctx.session, building.kind);
    let Some(_) = ctx.data.buildings.get(building.kind.id()) else {
        return false;
    };
    building.complete
        && level < progression::MAX_BUILDING_LEVEL
        && ctx.session.phase == GamePhase::Playing
        && upgrade_materials_available(ctx, building)
}

fn upgrade_materials_available(ctx: &UiContext<'_>, building: &Building) -> bool {
    let Some(def) = ctx.data.buildings.get(building.kind.id()) else {
        return false;
    };
    ctx.session.economy.bones >= def.upgrade_bones_cost
        && ctx.session.economy.mana >= def.upgrade_mana_cost
        && ctx.session.economy.wood >= def.upgrade_wood_cost
}

fn upgrade_tone(ctx: &UiContext<'_>, building: &Building) -> ButtonTone {
    if upgrade_enabled(ctx, building) {
        ButtonTone::Positive
    } else {
        ButtonTone::Secondary
    }
}
