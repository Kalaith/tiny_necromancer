//! Domain stewardship readout and map-overlay controls.

use super::components::virtual_button;
use super::{DomainOverlay, UiAction, UiContext};
use crate::data::SuspicionStage;
use crate::engine::{alerts, districts, jobs};
use crate::state::{GamePhase, JobKind, Selection, Technology};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_domain_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let rect = Rect::new(238.0, 106.0, 680.0, 490.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.045, 0.055, 0.055, 0.98))
            .with_border(1.0, Color::new(0.58, 0.70, 0.62, 0.75)),
    );
    draw_text_block(
        "DOMAIN STEWARDSHIP",
        rect.x + 24.0,
        rect.y + 20.0,
        300.0,
        22.0,
        18.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        "Read the clearing as districts, routes, and ward pressure.",
        rect.x + 24.0,
        rect.y + 50.0,
        560.0,
        20.0,
        14.0,
        0.0,
        dark::TEXT_DIM,
    );
    if !ctx
        .session
        .research
        .is_unlocked(Technology::DomainStewardship)
    {
        draw_text_block(
            "Study Domain Stewardship to open the settlement readout.",
            rect.x + 24.0,
            rect.y + 112.0,
            rect.w - 48.0,
            48.0,
            17.0,
            4.0,
            dark::TEXT,
        );
        return;
    }

    draw_summary_cards(ctx, rect);
    draw_overlay_controls(ctx, rect, pointer, actions);
    draw_stewardship_readout(ctx, rect, pointer, actions);
}

fn draw_summary_cards(ctx: &UiContext<'_>, rect: Rect) {
    let district_detail = if ctx
        .session
        .progress
        .district_ledger
        .recent_activity
        .is_empty()
    {
        "marked tiles".to_owned()
    } else {
        format!(
            "tiles · {} effects",
            ctx.session.progress.district_ledger.recent_activity.len()
        )
    };
    let cards = [
        (
            "UNDEAD",
            ctx.session.active_undead().to_string(),
            "bound workers".to_owned(),
        ),
        (
            "DISTRICT 01",
            zone_tile_count(ctx).to_string(),
            district_detail,
        ),
        (
            "WARD CHARGES",
            ctx.session.economy.ward_charges.to_string(),
            "ready to spend".to_owned(),
        ),
    ];
    for (index, (label, value, detail)) in cards.into_iter().enumerate() {
        let card = Rect::new(
            rect.x + 24.0 + index as f32 * 208.0,
            rect.y + 88.0,
            192.0,
            68.0,
        );
        draw_surface(
            card,
            &SurfaceStyle::new(Color::new(0.075, 0.095, 0.085, 1.0))
                .with_border(1.0, Color::new(0.36, 0.52, 0.43, 0.62)),
        );
        draw_text_block(
            label,
            card.x + 12.0,
            card.y + 9.0,
            card.w - 24.0,
            14.0,
            10.0,
            0.0,
            dark::TEXT_DIM,
        );
        draw_text_block(
            &value,
            card.x + 12.0,
            card.y + 27.0,
            70.0,
            24.0,
            20.0,
            0.0,
            dark::TEXT_BRIGHT,
        );
        draw_text_block(
            &detail,
            card.x + 78.0,
            card.y + 34.0,
            card.w - 90.0,
            18.0,
            11.0,
            0.0,
            dark::TEXT_DIM,
        );
    }
}

fn draw_overlay_controls(
    ctx: &UiContext<'_>,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    draw_text_block(
        "MAP OVERLAYS",
        rect.x + 24.0,
        rect.y + 178.0,
        160.0,
        16.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
    let controls = [
        (
            DomainOverlay::Zones,
            "Zones",
            "district marks",
            ctx.domain_overlays.zones,
        ),
        (
            DomainOverlay::Routes,
            "Routes",
            "full paths + blockers",
            ctx.domain_overlays.routes,
        ),
        (
            DomainOverlay::Pressure,
            "Pressure",
            "road watch",
            ctx.domain_overlays.pressure,
        ),
    ];
    for (index, (overlay, label, detail, enabled)) in controls.into_iter().enumerate() {
        let button = Rect::new(
            rect.x + 24.0 + index as f32 * 208.0,
            rect.y + 202.0,
            192.0,
            48.0,
        );
        if virtual_button(
            button,
            &format!("{} · {}", label, if enabled { "ON" } else { "OFF" }),
            true,
            if enabled {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            pointer,
        ) {
            actions.push(UiAction::ToggleDomainOverlay(overlay));
        }
        draw_text_block(
            detail,
            button.x + 4.0,
            button.bottom() + 5.0,
            button.w - 8.0,
            16.0,
            11.0,
            0.0,
            dark::TEXT_DIM,
        );
    }
    draw_text_block(
        &districts::ledger_summary(ctx.session),
        rect.x + 24.0,
        rect.y + 272.0,
        rect.w - 48.0,
        16.0,
        11.0,
        0.0,
        dark::ACCENT,
    );
}

fn draw_stewardship_readout(
    ctx: &UiContext<'_>,
    rect: Rect,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let (clear_routes, total_routes) = super::world_feedback::route_counts(ctx);
    let patrol_coverage = jobs::patrol_coverage(ctx.session);
    let route_summary = if total_routes == 0 {
        format!(
            "no destinations plotted · {}/{} patrol posts",
            patrol_coverage.covered_posts, patrol_coverage.total_posts
        )
    } else {
        format!(
            "{clear_routes}/{total_routes} routes clear · {}/{} patrol posts",
            patrol_coverage.covered_posts, patrol_coverage.total_posts
        )
    };
    draw_text_block(
        "STEWARDSHIP READOUT",
        rect.x + 24.0,
        rect.y + 286.0,
        220.0,
        16.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_readout_line(
        "ROAD PRESSURE",
        &format!(
            "{} · {:.0}%",
            stage_label(ctx.session.pressure.stage),
            ctx.session.pressure.suspicion
        ),
        rect.x + 24.0,
        rect.y + 314.0,
        pressure_color(ctx.session.pressure.stage),
    );
    draw_readout_line(
        "WORKFORCE",
        &format!(
            "{}/{} active · {} guarding · {route_summary}",
            active_workers(ctx),
            ctx.session.workforce.workers.len(),
            assigned_count(ctx, JobKind::Guard)
        ),
        rect.x + 24.0,
        rect.y + 344.0,
        if patrol_coverage.covered_posts == patrol_coverage.total_posts {
            dark::TEXT
        } else {
            dark::WARNING
        },
    );
    if virtual_button(
        Rect::new(rect.x + 24.0, rect.y + 366.0, 192.0, 44.0),
        &format!("Policy · {}", ctx.session.stewardship_policy.label()),
        ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::CycleStewardshipPolicy);
    }
    let policy_hint = if ctx.session.phase == GamePhase::Playing {
        format!(
            "Tap to cycle · {}",
            ctx.session.stewardship_policy.description()
        )
    } else {
        format!(
            "Resume play to change · {}",
            ctx.session.stewardship_policy.description()
        )
    };
    draw_text_block(
        &policy_hint,
        rect.x + 228.0,
        rect.y + 370.0,
        390.0,
        18.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_text_block(
        &districts::rule_summary(ctx.session, &ctx.data.config.district_rules),
        rect.x + 228.0,
        rect.y + 392.0,
        390.0,
        18.0,
        11.0,
        0.0,
        dark::ACCENT,
    );
    draw_text_block(
        &districts::operations_summary(ctx.session),
        rect.x + 24.0,
        rect.y + 414.0,
        440.0,
        18.0,
        11.0,
        0.0,
        if districts::staffing_needs_attention(ctx.session) {
            dark::WARNING
        } else {
            dark::TEXT_DIM
        },
    );
    draw_text_block(
        &districts::coverage_summary(ctx.session),
        rect.x + 24.0,
        rect.y + 434.0,
        440.0,
        18.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );

    let operational = alerts::collect(ctx.session, ctx.data);
    let detail = operational.first().map_or_else(
        || "No active blockers detected; the clearing is answering its orders.".to_owned(),
        |alert| format!("{} · {}", alert.title, alert.detail),
    );
    draw_text_block(
        &format!(
            "{} alert{} · {}",
            operational.len(),
            if operational.len() == 1 { "" } else { "s" },
            detail
        ),
        rect.x + 24.0,
        rect.y + 456.0,
        440.0,
        38.0,
        12.0,
        4.0,
        if operational.is_empty() {
            dark::POSITIVE
        } else {
            dark::WARNING
        },
    );
    let ward_label = if ctx.session.phase != GamePhase::Playing {
        "Paused"
    } else if ctx.session.economy.ward_charges == 0 {
        "No wards"
    } else if ctx.session.pressure.suspicion <= 0.0 {
        "No pressure"
    } else {
        "Quiet ward"
    };
    if virtual_button(
        Rect::new(rect.x + 492.0, rect.y + 406.0, 142.0, 48.0),
        ward_label,
        ctx.session.phase == GamePhase::Playing
            && ctx.session.economy.ward_charges > 0
            && ctx.session.pressure.suspicion > 0.0,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::UseWardCharge);
    }
    if let Some(alert) = operational.first().and_then(|alert| alert.target) {
        if virtual_button(
            Rect::new(rect.x + 492.0, rect.y + 478.0, 142.0, 32.0),
            alert_button_label(operational.first().map(|alert| alert.title)),
            true,
            ButtonTone::Warning,
            pointer,
        ) {
            actions.push(alert_action(ctx, alert));
        }
    }
}

fn alert_button_label(title: Option<&str>) -> &'static str {
    match title {
        Some("Route blocked") => "Inspect route",
        Some("Patrol coverage") => "Staff patrol",
        Some("Work district idle") => "Assign work",
        Some("Storage district idle") => "Staff storage",
        Some("District route gap") => "Clear district route",
        _ => "Locate blocker",
    }
}

fn draw_readout_line(label: &str, value: &str, x: f32, y: f32, color: Color) {
    draw_text_block(label, x, y, 136.0, 18.0, 11.0, 0.0, dark::TEXT_DIM);
    draw_text_block(value, x + 148.0, y, 470.0, 18.0, 13.0, 0.0, color);
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

fn zone_tile_count(ctx: &UiContext<'_>) -> usize {
    ctx.session
        .world
        .zones
        .iter()
        .map(|zone| zone.tiles.len())
        .sum()
}

fn active_workers(ctx: &UiContext<'_>) -> usize {
    ctx.session
        .workforce
        .workers
        .iter()
        .filter(|worker| !matches!(worker.status, crate::state::WorkerStatus::Idle))
        .count()
}

fn assigned_count(ctx: &UiContext<'_>, job: JobKind) -> usize {
    ctx.session
        .workforce
        .workers
        .iter()
        .filter(|worker| worker.assignment == job)
        .count()
}

fn stage_label(stage: SuspicionStage) -> &'static str {
    match stage {
        SuspicionStage::Calm => "Calm",
        SuspicionStage::Rumour => "Rumour",
        SuspicionStage::Questioning => "Questioning",
        SuspicionStage::Investigation => "Investigation",
    }
}

fn pressure_color(stage: SuspicionStage) -> Color {
    match stage {
        SuspicionStage::Calm => dark::POSITIVE,
        SuspicionStage::Rumour => dark::ACCENT,
        SuspicionStage::Questioning => dark::WARNING,
        SuspicionStage::Investigation => dark::NEGATIVE,
    }
}
