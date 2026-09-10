//! Compact field notes and settlement overview.

use super::components::virtual_button;
use super::research::{draw_research_panel, draw_small_info_panel, draw_zones_panel};
use super::{Panel, UiAction, UiContext};
use crate::state::{BuildingKind, Technology, UndeadKind};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_feed(ctx: &UiContext<'_>) {
    let rect = Rect::new(20.0, 572.0, 310.0, 102.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.045, 0.055, 0.055, 0.90))
            .with_border(1.0, Color::new(0.46, 0.57, 0.50, 0.50)),
    );
    draw_text_block(
        "FIELD NOTES",
        rect.x + 14.0,
        rect.y + 10.0,
        120.0,
        16.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    for (index, entry) in ctx.session.pressure.feed.iter().take(3).enumerate() {
        draw_text_block(
            &entry.message,
            rect.x + 14.0,
            rect.y + 32.0 + index as f32 * 21.0,
            rect.w - 28.0,
            18.0,
            if index == 0 { 13.0 } else { 12.0 },
            0.0,
            if index == 0 {
                dark::TEXT
            } else {
                dark::TEXT_DIM
            },
        );
    }
}

pub(super) fn draw_minimap(ctx: &UiContext<'_>) {
    let rect = Rect::new(1050.0, 562.0, 190.0, 112.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.035, 0.07, 0.05, 0.94))
            .with_border(1.0, Color::new(0.52, 0.70, 0.56, 0.60)),
    );
    let sx = (rect.w - 20.0) / ctx.session.world_width() as f32;
    let sy = (rect.h - 28.0) / ctx.session.world_height() as f32;
    for y in 0..ctx.session.world_height() {
        for x in 0..ctx.session.world_width() {
            let c = if x as i32 >= ctx.session.world.road_x {
                Color::new(0.34, 0.27, 0.20, 1.0)
            } else {
                Color::new(0.14, 0.30, 0.19, 1.0)
            };
            draw_rectangle(
                rect.x + 10.0 + x as f32 * sx,
                rect.y + 18.0 + y as f32 * sy,
                sx + 1.0,
                sy + 1.0,
                c,
            );
        }
    }
    draw_text_block(
        "SETTLEMENT VIEW",
        rect.x + 10.0,
        rect.y + 4.0,
        rect.w - 20.0,
        13.0,
        10.0,
        0.0,
        dark::TEXT_DIM,
    );
}

pub(super) fn draw_panel(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    match ctx.panel {
        Panel::Build => draw_build_panel(ctx, pointer, actions),
        Panel::Orders => draw_orders_panel(ctx),
        Panel::Undead => draw_undead_panel(ctx, pointer, actions),
        Panel::Research => draw_research_panel(ctx, pointer, actions),
        Panel::Zones => draw_zones_panel(ctx, pointer, actions),
        Panel::None => {}
    }
}

fn draw_build_panel(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let rect = Rect::new(350.0, 460.0, 580.0, 146.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.055, 0.065, 0.065, 0.97))
            .with_border(1.0, Color::new(0.56, 0.67, 0.60, 0.70)),
    );
    draw_text_block(
        "BUILD PALETTE",
        rect.x + 18.0,
        rect.y + 14.0,
        150.0,
        16.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
    let e = &ctx.session.economy;
    if let Some(def) = ctx.data.buildings.get(BuildingKind::WorkShed.id()) {
        if virtual_button(
            Rect::new(rect.x + 18.0, rect.y + 42.0, 250.0, 48.0),
            &format!("Work shed · B{} W{}", def.bones_cost, def.wood_cost),
            !ctx.session.has_building(BuildingKind::WorkShed)
                && !ctx.session.building_in_progress(BuildingKind::WorkShed)
                && e.bones >= def.bones_cost
                && e.wood >= def.wood_cost,
            ButtonTone::Primary,
            pointer,
        ) {
            actions.push(UiAction::BeginPlacement(BuildingKind::WorkShed));
        }
    }
    if let Some(def) = ctx.data.buildings.get(BuildingKind::GraveLantern.id()) {
        if virtual_button(
            Rect::new(rect.x + 282.0, rect.y + 42.0, 280.0, 48.0),
            &format!(
                "Grave lantern · B{} M{} W{}",
                def.bones_cost, def.mana_cost, def.wood_cost
            ),
            !ctx.session.has_building(BuildingKind::GraveLantern)
                && !ctx.session.building_in_progress(BuildingKind::GraveLantern)
                && e.bones >= def.bones_cost
                && e.mana >= def.mana_cost
                && e.wood >= def.wood_cost,
            ButtonTone::Secondary,
            pointer,
        ) {
            actions.push(UiAction::BeginPlacement(BuildingKind::GraveLantern));
        }
    }
    let unlock_cost =
        crate::state::next_plot_unlock_cost(&ctx.data.config, ctx.session.progress.unlocked_plots);
    if virtual_button(
        Rect::new(rect.x + 18.0, rect.y + 102.0, 250.0, 44.0),
        &format!("Open next grave · {} wood", unlock_cost),
        ctx.session.progress.unlocked_plots < ctx.data.config.victory_plots
            && e.wood >= unlock_cost,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::UnlockPlot);
    }
    draw_text_block(
        "Placement shows footprint, cost, and collision before the order is accepted.",
        rect.x + 282.0,
        rect.y + 106.0,
        rect.w - 300.0,
        22.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn draw_orders_panel(ctx: &UiContext<'_>) {
    draw_small_info_panel(
        "ORDERS",
        if ctx
            .session
            .research
            .is_unlocked(Technology::BindingRoutines)
        {
            "Repeat priorities are unlocked. Select a worker to tune the queue."
        } else {
            "Direct orders are available now. Binding Routines will add repeat priorities."
        },
    );
}

fn draw_undead_panel(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let rect = Rect::new(350.0, 460.0, 580.0, 146.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.055, 0.065, 0.065, 0.97))
            .with_border(1.0, Color::new(0.56, 0.67, 0.60, 0.70)),
    );
    let skeleton = ctx
        .data
        .undead
        .get(UndeadKind::Skeleton.id())
        .expect("validated skeleton");
    let brute = ctx
        .data
        .undead
        .get(UndeadKind::BruteSkeleton.id())
        .expect("validated brute");
    if virtual_button(
        Rect::new(rect.x + 18.0, rect.y + 42.0, 250.0, 48.0),
        &format!(
            "Raise skeleton · B{} M{}",
            skeleton.bones_cost, skeleton.mana_cost
        ),
        ctx.session.economy.bones >= skeleton.bones_cost
            && ctx.session.economy.mana >= skeleton.mana_cost,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::Raise(UndeadKind::Skeleton));
    }
    if virtual_button(
        Rect::new(rect.x + 282.0, rect.y + 42.0, 280.0, 48.0),
        &format!("Raise brute · B{} M{}", brute.bones_cost, brute.mana_cost),
        ctx.session.economy.bones >= brute.bones_cost
            && ctx.session.economy.mana >= brute.mana_cost
            && ctx
                .session
                .economy
                .corpses
                .iter()
                .any(|corpse| corpse.quality == crate::state::CorpseQuality::Notable),
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::Raise(UndeadKind::BruteSkeleton));
    }
    draw_text_block(
        "Bones become labour; labour makes the next grave possible.",
        rect.x + 18.0,
        rect.y + 106.0,
        rect.w - 36.0,
        22.0,
        13.0,
        0.0,
        dark::TEXT_DIM,
    );
}
