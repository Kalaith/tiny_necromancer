//! Compact stewardship and district-domain controls.

use super::*;

pub(super) fn draw_compact_domain_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("DOMAIN", sheet);
    let width = (sheet.w - 42.0) / 3.0;
    draw_overlay_buttons(ctx, pointer, actions, sheet, width);
    draw_domain_summaries(ctx, sheet);
    draw_domain_actions(ctx, pointer, actions, sheet);
}

fn draw_overlay_buttons(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    width: f32,
) {
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
}

fn draw_domain_summaries(ctx: &UiContext<'_>, sheet: Rect) {
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
    let staffing_summary = if sheet.w < 520.0 {
        districts::compact_operations_summary(ctx.session)
    } else {
        districts::operations_summary(ctx.session)
    };
    draw_text_block(
        &staffing_summary,
        sheet.x + 16.0,
        sheet.y + 192.0,
        sheet.w - 32.0,
        18.0,
        11.0,
        0.0,
        if districts::staffing_needs_attention(ctx.session) {
            dark::WARNING
        } else {
            dark::TEXT_DIM
        },
    );
    let coverage_summary = if sheet.w < 520.0 {
        districts::compact_coverage_summary(ctx.session)
    } else {
        districts::coverage_summary(ctx.session)
    };
    draw_text_block(
        &coverage_summary,
        sheet.x + 16.0,
        sheet.y + 212.0,
        sheet.w - 32.0,
        18.0,
        11.0,
        0.0,
        if districts::route_coverage_needs_attention(ctx.session) {
            dark::WARNING
        } else {
            dark::TEXT_DIM
        },
    );
    draw_domain_route_summary(ctx, sheet);
}

fn draw_domain_route_summary(ctx: &UiContext<'_>, sheet: Rect) {
    let (clear_routes, total_routes) = super::super::super::world_feedback::route_counts(ctx);
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
        sheet.y + 232.0,
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
}

fn draw_domain_actions(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 252.0, sheet.w - 32.0, 44.0),
        super::super::compact_policy_label(ctx.session.stewardship_policy),
        ctx.session.phase == GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::CycleStewardshipPolicy);
    }
    if virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 300.0, sheet.w - 32.0, 44.0),
        "Quiet ward · -8 suspicion",
        ctx.session.phase == GamePhase::Playing
            && ctx.session.economy.ward_charges > 0
            && ctx.session.pressure.suspicion > 0.0,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::UseWardCharge);
    }
    let advisory_alert = alerts::collect(ctx.session, ctx.data)
        .into_iter()
        .find(|alert| {
            matches!(
                alert.title,
                "Route blocked"
                    | "Patrol coverage"
                    | "Work district idle"
                    | "Storage district idle"
                    | "District route gap"
                    | "Route policy waiting"
            )
        });
    if let Some(alert) = advisory_alert.as_ref() {
        let Some(target) = alert.target else { return };
        let action = compact_advisory_action(ctx, target);
        if virtual_button(
            Rect::new(sheet.x + 16.0, sheet.y + 348.0, sheet.w - 32.0, 44.0),
            compact_advisory_label(alert.title),
            true,
            ButtonTone::Warning,
            pointer,
        ) {
            actions.push(action);
        }
    }
}
