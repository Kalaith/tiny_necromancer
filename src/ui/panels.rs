//! Compact field notes and settlement overview.

use super::components::virtual_button;
use super::domain::draw_domain_panel;
use super::orders::draw_orders_panel;
use super::research::{draw_research_panel, draw_zones_panel};
use super::{Panel, UiAction, UiContext};
use crate::engine::alerts::{self, AlertSeverity};
use crate::engine::districts;
use crate::state::{BuildingKind, Selection, Technology, UndeadKind};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_feed(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    if ctx.session.pressure.active_event.is_some() {
        return;
    }
    if matches!(
        ctx.panel,
        Panel::Research | Panel::Orders | Panel::Domain | Panel::Feed
    ) {
        return;
    }
    let operational = alerts::collect(ctx.session, ctx.data);
    let rect = if operational.is_empty() {
        Rect::new(20.0, 554.0, 310.0, 120.0)
    } else {
        Rect::new(20.0, 490.0, 310.0, 184.0)
    };
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.045, 0.055, 0.055, 0.90))
            .with_border(1.0, Color::new(0.46, 0.57, 0.50, 0.50)),
    );
    draw_text_block(
        if operational.is_empty() {
            "FIELD NOTES"
        } else {
            "OPERATIONAL ALERTS"
        },
        rect.x + 14.0,
        rect.y + 10.0,
        rect.w - 126.0,
        16.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    let history_label = format!("History {}", ctx.session.pressure.feed.len());
    if virtual_button(
        Rect::new(rect.right() - 104.0, rect.y + 4.0, 90.0, 44.0),
        &history_label,
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::TogglePanel(Panel::Feed));
    }
    if operational.is_empty() {
        draw_feed_history(ctx, rect);
    } else {
        for (index, alert) in operational.iter().take(3).enumerate() {
            let button = Rect::new(
                rect.x + 14.0,
                rect.y + 50.0 + index as f32 * 45.0,
                rect.w - 28.0,
                44.0,
            );
            if virtual_button(
                button,
                &format!("{} · {}", alert.title, alert.detail),
                true,
                alert_tone(alert.severity),
                pointer,
            ) {
                if let Some(target) = alert.target {
                    actions.push(alert_action(ctx, target));
                }
            }
        }
    }
}

fn draw_feed_history(ctx: &UiContext<'_>, rect: Rect) {
    for (index, entry) in ctx.session.pressure.feed.iter().take(3).enumerate() {
        let age_alpha = (1.0 - entry.age_seconds / 18.0).clamp(0.42, 1.0);
        let text_color = if index == 0 {
            dark::TEXT.with_alpha(age_alpha)
        } else {
            dark::TEXT_DIM.with_alpha(age_alpha)
        };
        draw_text_block(
            &entry.message,
            rect.x + 14.0,
            rect.y + 56.0 + index as f32 * 21.0,
            rect.w - 28.0,
            18.0,
            if index == 0 { 13.0 } else { 12.0 },
            0.0,
            text_color,
        );
    }
}

fn alert_tone(severity: AlertSeverity) -> ButtonTone {
    match severity {
        AlertSeverity::Critical => ButtonTone::Danger,
        AlertSeverity::Warning => ButtonTone::Warning,
        AlertSeverity::Info => ButtonTone::Secondary,
    }
}

fn alert_action(ctx: &UiContext<'_>, target: Selection) -> UiAction {
    match target {
        Selection::Worker(index) => UiAction::SelectWorker(index),
        Selection::Necromancer => UiAction::SelectNecromancer,
        Selection::Ground(tile) => UiAction::SelectTile(tile),
        Selection::Grave(index) => ctx.session.world.plots.get(index).map_or(
            UiAction::SelectTile(crate::state::WorldState::stockpile_position()),
            |plot| UiAction::SelectTile(plot.position),
        ),
        Selection::Building(index) => ctx.session.world.buildings.get(index).map_or(
            UiAction::SelectTile(crate::state::WorldState::stockpile_position()),
            |building| UiAction::SelectTile(building.position),
        ),
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
    if ctx.domain_overlays.zones {
        for zone in &ctx.session.world.zones {
            let color = match zone.kind {
                crate::state::ZoneKind::Work => Color::new(0.24, 0.72, 0.58, 0.75),
                crate::state::ZoneKind::Storage => Color::new(0.88, 0.62, 0.24, 0.82),
                crate::state::ZoneKind::Patrol => Color::new(0.42, 0.58, 0.92, 0.82),
            };
            for tile in &zone.tiles {
                let marker = minimap_tile_rect(rect, sx, sy, *tile);
                draw_rectangle(marker.x, marker.y, marker.w, marker.h, color);
            }
        }
    }
    for building in &ctx.session.world.buildings {
        let marker = minimap_tile_rect(rect, sx, sy, building.position);
        draw_rectangle(
            marker.x,
            marker.y,
            sx * building.width as f32,
            sy * building.height as f32,
            if building.complete {
                Color::new(0.74, 0.72, 0.62, 1.0)
            } else {
                Color::new(0.74, 0.48, 0.28, 1.0)
            },
        );
    }
    for worker in &ctx.session.workforce.workers {
        let marker = minimap_tile_rect(rect, sx, sy, worker.position);
        draw_circle(
            marker.center().x,
            marker.center().y,
            marker.w * 0.22,
            Color::new(0.92, 0.92, 0.78, 1.0),
        );
    }
    let necromancer = minimap_tile_rect(rect, sx, sy, ctx.session.world.necromancer_position);
    draw_rectangle(
        necromancer.x + necromancer.w * 0.28,
        necromancer.y + necromancer.h * 0.28,
        necromancer.w * 0.44,
        necromancer.h * 0.44,
        Color::new(0.78, 0.48, 0.92, 1.0),
    );
    if let Some(tile) = minimap_selection_tile(ctx) {
        let marker = minimap_tile_rect(rect, sx, sy, tile).inset(1.0);
        draw_rectangle_lines(
            marker.x,
            marker.y,
            marker.w,
            marker.h,
            1.0,
            Color::new(0.95, 0.92, 0.62, 1.0),
        );
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

fn minimap_tile_rect(rect: Rect, sx: f32, sy: f32, tile: macroquad_toolkit::grid::TilePos) -> Rect {
    Rect::new(
        rect.x + 10.0 + tile.x as f32 * sx,
        rect.y + 18.0 + tile.y as f32 * sy,
        sx,
        sy,
    )
}

fn minimap_selection_tile(ctx: &UiContext<'_>) -> Option<macroquad_toolkit::grid::TilePos> {
    match ctx.session.world.selected {
        Some(crate::state::Selection::Ground(tile)) => Some(tile),
        Some(crate::state::Selection::Necromancer) => Some(ctx.session.world.necromancer_position),
        Some(crate::state::Selection::Worker(index)) => ctx
            .session
            .workforce
            .workers
            .get(index)
            .map(|worker| worker.position),
        Some(crate::state::Selection::Building(index)) => ctx
            .session
            .world
            .buildings
            .get(index)
            .map(|building| building.position),
        Some(crate::state::Selection::Grave(index)) => {
            ctx.session.world.plots.get(index).map(|plot| plot.position)
        }
        None => None,
    }
}

pub(super) fn draw_panel(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    match ctx.panel {
        Panel::Build => draw_build_panel(ctx, pointer, actions),
        Panel::Orders => draw_orders_panel(ctx, pointer, actions),
        Panel::Undead => draw_undead_panel(ctx, pointer, actions),
        Panel::Research => draw_research_panel(ctx, pointer, actions),
        Panel::Zones => draw_zones_panel(ctx, pointer, actions),
        Panel::Domain => draw_domain_panel(ctx, pointer, actions),
        Panel::Feed => draw_feed_history_panel(ctx, pointer, actions),
        Panel::None => {}
    }
}

fn draw_feed_history_panel(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let rect = Rect::new(238.0, 106.0, 680.0, 490.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.045, 0.055, 0.055, 0.98))
            .with_border(1.0, Color::new(0.58, 0.70, 0.62, 0.75)),
    );
    draw_text_block(
        "FIELD NOTES ARCHIVE",
        rect.x + 24.0,
        rect.y + 20.0,
        300.0,
        22.0,
        18.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        &format!(
            "{} notes · {} road events logged · {} district effects tracked.",
            ctx.session.pressure.feed.len(),
            ctx.session.pressure.event_history.len(),
            ctx.session.progress.district_ledger.recent_activity.len()
        ),
        rect.x + 24.0,
        rect.y + 50.0,
        500.0,
        20.0,
        14.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_text_block(
        &districts::ledger_summary(ctx.session),
        rect.x + 24.0,
        rect.y + 72.0,
        rect.w - 48.0,
        16.0,
        11.0,
        0.0,
        dark::ACCENT,
    );
    draw_text_block(
        &districts::activity_summary(ctx.session),
        rect.x + 24.0,
        rect.y + 90.0,
        rect.w - 48.0,
        16.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    if virtual_button(
        Rect::new(rect.right() - 96.0, rect.y + 14.0, 72.0, 44.0),
        "Close",
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::TogglePanel(Panel::None));
    }
    for (index, entry) in ctx.session.pressure.feed.iter().take(7).enumerate() {
        let row = Rect::new(
            rect.x + 24.0,
            rect.y + 114.0 + index as f32 * 52.0,
            632.0,
            44.0,
        );
        draw_surface(
            row,
            &SurfaceStyle::new(if index == 0 {
                Color::new(0.10, 0.14, 0.12, 1.0)
            } else {
                Color::new(0.075, 0.085, 0.085, 1.0)
            })
            .with_border(1.0, Color::new(0.40, 0.47, 0.43, 0.45)),
        );
        draw_text_block(
            &entry.message,
            row.x + 14.0,
            row.y + 7.0,
            492.0,
            32.0,
            13.0,
            3.0,
            if index == 0 {
                dark::TEXT_BRIGHT
            } else {
                dark::TEXT
            },
        );
        let age_label = if index == 0 {
            format!("LATEST · {:.0}s", entry.age_seconds)
        } else {
            format!("{:.0}s ago", entry.age_seconds)
        };
        draw_text_block(
            &age_label,
            row.x + 520.0,
            row.y + 14.0,
            96.0,
            18.0,
            11.0,
            0.0,
            dark::TEXT_DIM,
        );
    }
    if ctx.session.pressure.feed.is_empty() {
        draw_text_block(
            "No field notes recorded yet.",
            rect.x + 24.0,
            rect.y + 114.0,
            rect.w - 48.0,
            22.0,
            16.0,
            0.0,
            dark::TEXT_DIM,
        );
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
    if let Some(def) = ctx.data.buildings.get(BuildingKind::OssuaryKiln.id()) {
        let unlocked = ctx
            .session
            .research
            .is_unlocked(Technology::OssuaryLogistics);
        let label = if unlocked {
            format!("Ossuary kiln · B{} W{}", def.bones_cost, def.wood_cost)
        } else {
            "Ossuary kiln · Logistics locked".to_owned()
        };
        if virtual_button(
            Rect::new(rect.x + 282.0, rect.y + 102.0, 280.0, 44.0),
            &label,
            unlocked
                && !ctx.session.has_building(BuildingKind::OssuaryKiln)
                && !ctx.session.building_in_progress(BuildingKind::OssuaryKiln)
                && e.bones >= def.bones_cost
                && e.wood >= def.wood_cost,
            ButtonTone::Secondary,
            pointer,
        ) {
            actions.push(UiAction::BeginPlacement(BuildingKind::OssuaryKiln));
        }
    }
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
