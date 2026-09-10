//! Management panels presented inside the compact command sheet.

use super::super::components::{compact_virtual_button, virtual_button};
use super::super::{DomainOverlay, Panel, UiAction, UiContext};
use crate::engine::{alerts, districts};
use crate::state::{BuildingKind, GamePhase, Selection, Technology, UndeadKind, ZoneKind};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_compact_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    match ctx.panel {
        Panel::Build => draw_compact_build_panel(ctx, pointer, actions, sheet),
        Panel::Orders => draw_compact_orders_panel(ctx, pointer, actions, sheet),
        Panel::Undead => draw_compact_undead_panel(ctx, pointer, actions, sheet),
        Panel::Research => draw_compact_research_panel(ctx, pointer, actions, sheet),
        Panel::Zones => draw_compact_zones_panel(ctx, pointer, actions, sheet),
        Panel::Domain => draw_compact_domain_panel(ctx, pointer, actions, sheet),
        Panel::Feed => draw_compact_feed_panel(ctx, pointer, actions, sheet),
        Panel::None => {}
    }
}

fn compact_panel_title(title: &str, sheet: Rect) {
    draw_text_block(
        title,
        sheet.x + 16.0,
        sheet.y + 84.0,
        sheet.w - 32.0,
        22.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
}

fn draw_compact_build_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("BUILD PALETTE", sheet);
    let e = &ctx.session.economy;
    let half = (sheet.w - 42.0) / 2.0;
    for (index, kind) in [
        BuildingKind::WorkShed,
        BuildingKind::GraveLantern,
        BuildingKind::OssuaryKiln,
    ]
    .into_iter()
    .enumerate()
    {
        let Some(def) = ctx.data.buildings.get(kind.id()) else {
            continue;
        };
        let label = if sheet.w < 500.0 {
            let short_name = match kind {
                BuildingKind::WorkShed => "Shed",
                BuildingKind::GraveLantern => "Lantern",
                BuildingKind::OssuaryKiln => "Kiln",
            };
            format!("{} · B{} W{}", short_name, def.bones_cost, def.wood_cost)
        } else {
            format!("{} · B{} W{}", def.name, def.bones_cost, def.wood_cost)
        };
        let unlocked = kind != BuildingKind::OssuaryKiln
            || ctx
                .session
                .research
                .is_unlocked(Technology::OssuaryLogistics);
        let button = Rect::new(
            sheet.x + 16.0 + (index % 2) as f32 * (half + 10.0),
            sheet.y + 116.0 + (index / 2) as f32 * 52.0,
            half,
            44.0,
        );
        if virtual_button(
            button,
            &label,
            unlocked
                && ctx.session.phase == GamePhase::Playing
                && !ctx.session.has_building(kind)
                && !ctx.session.building_in_progress(kind)
                && e.bones >= def.bones_cost
                && e.wood >= def.wood_cost,
            ButtonTone::Primary,
            pointer,
        ) {
            actions.push(UiAction::BeginPlacement(kind));
        }
    }
    let cost =
        crate::state::next_plot_unlock_cost(&ctx.data.config, ctx.session.progress.unlocked_plots);
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 222.0, sheet.w - 32.0, 44.0),
        &format!("Open next grave · {} wood", cost),
        ctx.session.progress.unlocked_plots < ctx.data.config.victory_plots
            && e.wood >= cost
            && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::UnlockPlot);
    }
}

fn draw_compact_orders_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("WORKER ORDERS", sheet);
    for (index, job) in ctx.session.workforce.priorities.iter().copied().enumerate() {
        let y = sheet.y + 110.0 + index as f32 * 44.0;
        draw_text_block(
            &format!("{}. {}", index + 1, job.label()),
            sheet.x + 16.0,
            y + 13.0,
            sheet.w - 150.0,
            18.0,
            13.0,
            0.0,
            dark::TEXT_BRIGHT,
        );
        let up = Rect::new(sheet.right() - 126.0, y, 56.0, 40.0);
        let down = Rect::new(sheet.right() - 64.0, y, 56.0, 40.0);
        if compact_virtual_button(
            up,
            "Up",
            index > 0 && ctx.session.phase == GamePhase::Playing,
            ButtonTone::Secondary,
            11.0,
            pointer,
        ) {
            actions.push(UiAction::MovePriority(job, -1));
        }
        if compact_virtual_button(
            down,
            "Down",
            index + 1 < ctx.session.workforce.priorities.len()
                && ctx.session.phase == GamePhase::Playing,
            ButtonTone::Secondary,
            10.0,
            pointer,
        ) {
            actions.push(UiAction::MovePriority(job, 1));
        }
    }
}

fn draw_compact_undead_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("UNDEAD", sheet);
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
    let half = (sheet.w - 42.0) / 2.0;
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 122.0, half, 52.0),
        &format!(
            "Skeleton · B{} M{}",
            skeleton.bones_cost, skeleton.mana_cost
        ),
        ctx.session.economy.bones >= skeleton.bones_cost
            && ctx.session.economy.mana >= skeleton.mana_cost
            && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::Raise(UndeadKind::Skeleton));
    }
    if virtual_button(
        Rect::new(sheet.x + 26.0 + half, sheet.y + 122.0, half, 52.0),
        &format!("Brute · B{} M{}", brute.bones_cost, brute.mana_cost),
        ctx.session.economy.bones >= brute.bones_cost
            && ctx.session.economy.mana >= brute.mana_cost
            && ctx.session.phase == GamePhase::Playing
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
}

fn draw_compact_research_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("RESEARCH", sheet);
    let techs = [
        Technology::BindingRoutines,
        Technology::Gravecraft,
        Technology::OssuaryLogistics,
        Technology::DomainStewardship,
    ];
    for (index, technology) in techs.into_iter().enumerate() {
        let y = sheet.y + 110.0 + index as f32 * 54.0;
        let complete = ctx.session.research.is_unlocked(technology);
        draw_text_block(
            technology.label(),
            sheet.x + 16.0,
            y + 7.0,
            sheet.w - 124.0,
            18.0,
            13.0,
            0.0,
            dark::TEXT_BRIGHT,
        );
        draw_text_block(
            technology.description(),
            sheet.x + 16.0,
            y + 27.0,
            sheet.w - 124.0,
            16.0,
            10.0,
            0.0,
            dark::TEXT_DIM,
        );
        if compact_virtual_button(
            Rect::new(sheet.right() - 96.0, y, 80.0, 44.0),
            if complete {
                "Done"
            } else if ctx.session.research.current == Some(technology) {
                "Studying"
            } else {
                "Study"
            },
            !complete
                && ctx.session.research.can_start(technology)
                && ctx.session.phase == GamePhase::Playing,
            if complete {
                ButtonTone::Secondary
            } else {
                ButtonTone::Positive
            },
            10.0,
            pointer,
        ) {
            actions.push(UiAction::StartResearch(technology));
        }
    }
}

fn draw_compact_zones_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("WORK AREAS", sheet);
    let instruction = if ctx
        .session
        .research
        .is_unlocked(Technology::DomainStewardship)
    {
        "Mark a Work, Storage, or Patrol tile; Domain bonuses stay local."
    } else {
        "Choose a tool, then tap world tiles to mark or clear them."
    };
    draw_text_block(
        instruction,
        sheet.x + 16.0,
        sheet.y + 112.0,
        sheet.w - 32.0,
        30.0,
        13.0,
        3.0,
        dark::TEXT_DIM,
    );
    let width = (sheet.w - 42.0) / 3.0;
    for (index, kind) in [ZoneKind::Work, ZoneKind::Storage, ZoneKind::Patrol]
        .into_iter()
        .enumerate()
    {
        if compact_virtual_button(
            Rect::new(
                sheet.x + 16.0 + index as f32 * (width + 5.0),
                sheet.y + 156.0,
                width,
                48.0,
            ),
            kind.label(),
            ctx.session.phase == GamePhase::Playing,
            if ctx.zone_mode == Some(kind) {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            12.0,
            pointer,
        ) {
            actions.push(UiAction::ToggleZone(kind));
        }
    }
}

fn draw_compact_domain_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("DOMAIN", sheet);
    let width = (sheet.w - 42.0) / 3.0;
    for (index, (overlay, label, enabled)) in [
        (DomainOverlay::Zones, "Zones", ctx.domain_overlays.zones),
        (DomainOverlay::Routes, "Routes", ctx.domain_overlays.routes),
        (
            DomainOverlay::Pressure,
            "Pressure",
            ctx.domain_overlays.pressure,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if compact_virtual_button(
            Rect::new(
                sheet.x + 16.0 + index as f32 * (width + 5.0),
                sheet.y + 112.0,
                width,
                48.0,
            ),
            &format!("{} {}", label, if enabled { "ON" } else { "OFF" }),
            true,
            if enabled {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            11.0,
            pointer,
        ) {
            actions.push(UiAction::ToggleDomainOverlay(overlay));
        }
    }
    let rule_summary = districts::rule_summary(ctx.session, &ctx.data.config.district_rules);
    let rule_summary = if sheet.w >= 520.0 {
        format!(
            "{rule_summary} · {}",
            districts::ledger_summary(ctx.session)
        )
    } else {
        rule_summary
    };
    draw_text_block(
        &rule_summary,
        sheet.x + 16.0,
        sheet.y + 172.0,
        sheet.w - 32.0,
        20.0,
        11.0,
        0.0,
        dark::ACCENT,
    );
    let (clear_routes, total_routes) = super::super::world_feedback::route_counts(ctx);
    let patrol_coverage = crate::engine::jobs::patrol_coverage(ctx.session);
    let route_summary = if total_routes == 0 {
        format!(
            "Routes · no destinations · P{}/{}",
            patrol_coverage.covered_posts, patrol_coverage.total_posts
        )
    } else {
        format!(
            "Routes · {clear_routes}/{total_routes} clear · P{}/{}",
            patrol_coverage.covered_posts, patrol_coverage.total_posts
        )
    };
    draw_text_block(
        &route_summary,
        sheet.x + 16.0,
        sheet.y + 194.0,
        sheet.w - 32.0,
        16.0,
        11.0,
        0.0,
        if clear_routes == total_routes
            && patrol_coverage.covered_posts == patrol_coverage.total_posts
        {
            dark::POSITIVE
        } else {
            dark::WARNING
        },
    );
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 220.0, sheet.w - 32.0, 44.0),
        &format!("Policy · {}", ctx.session.stewardship_policy.label()),
        ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::CycleStewardshipPolicy);
    }
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 270.0, sheet.w - 32.0, 44.0),
        "Quiet ward · -8 suspicion",
        ctx.session.phase == GamePhase::Playing
            && ctx.session.economy.ward_charges > 0
            && ctx.session.pressure.suspicion > 0.0,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::UseWardCharge);
    }
    let coverage_alert = alerts::collect(ctx.session, ctx.data)
        .into_iter()
        .find(|alert| matches!(alert.title, "Route blocked" | "Patrol coverage"));
    let route_target = coverage_alert.as_ref().and_then(|alert| alert.target);
    if let Some(Selection::Worker(index)) = route_target {
        if virtual_button(
            Rect::new(sheet.x + 16.0, sheet.y + 320.0, sheet.w - 32.0, 44.0),
            if coverage_alert
                .as_ref()
                .is_some_and(|alert| alert.title == "Patrol coverage")
            {
                "Staff patrol"
            } else {
                "Inspect route"
            },
            true,
            ButtonTone::Warning,
            pointer,
        ) {
            actions.push(UiAction::SelectWorker(index));
        }
    }
}

fn draw_compact_feed_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("FIELD NOTES", sheet);
    let ledger = if sheet.w >= 520.0 {
        districts::ledger_summary(ctx.session)
    } else {
        let ledger = &ctx.session.progress.district_ledger;
        format!(
            "Ledger W{} · S+{} · P-{:.1}",
            ledger.work_cycles, ledger.storage_bonus_items, ledger.patrol_quieting
        )
    };
    draw_text_block(
        &ledger,
        sheet.x + 16.0,
        sheet.y + 104.0,
        sheet.w - 128.0,
        16.0,
        11.0,
        0.0,
        dark::ACCENT,
    );
    if virtual_button(
        Rect::new(sheet.right() - 92.0, sheet.y + 80.0, 76.0, 40.0),
        "Close",
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::TogglePanel(Panel::None));
    }
    for (index, entry) in ctx.session.pressure.feed.iter().take(4).enumerate() {
        let y = sheet.y + 132.0 + index as f32 * 52.0;
        draw_text_block(
            &entry.message,
            sheet.x + 16.0,
            y,
            sheet.w - 32.0,
            42.0,
            12.0,
            3.0,
            if index == 0 {
                dark::TEXT_BRIGHT
            } else {
                dark::TEXT_DIM
            },
        );
    }
}
