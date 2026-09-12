//! Compact work, storage, and patrol zone controls.

use super::*;

pub(super) fn draw_compact_zones_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    compact_panel_title("WORK AREAS", sheet);
    draw_zone_instruction(ctx, sheet);
    let width = (sheet.w - 42.0) / 3.0;
    draw_zone_tools(ctx, pointer, actions, sheet, width);
    draw_zone_storage(ctx, sheet);
    if ctx
        .session
        .research
        .is_unlocked(Technology::DomainStewardship)
    {
        draw_zone_routes(ctx, pointer, actions, sheet, width);
    }
}

fn draw_zone_instruction(ctx: &UiContext<'_>, sheet: Rect) {
    let instruction = if ctx
        .session
        .research
        .is_unlocked(Technology::DomainStewardship)
    {
        "Mark a Work, Storage, or Patrol tile; bonuses stay local. Work alerts point to reachable work."
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
}

fn draw_zone_tools(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    width: f32,
) {
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

fn draw_zone_storage(ctx: &UiContext<'_>, sheet: Rect) {
    draw_text_block(
        &format!(
            "{} · +{} cap/tile",
            districts::storage_summary(ctx.session, &ctx.data.config.district_rules),
            ctx.data.config.district_rules.storage_volume_per_tile
        ),
        sheet.x + 16.0,
        sheet.y + 210.0,
        sheet.w - 32.0,
        18.0,
        11.0,
        0.0,
        if districts::storage_space(ctx.session, &ctx.data.config.district_rules) == 0 {
            dark::WARNING
        } else {
            dark::ACCENT
        },
    );
}

fn draw_zone_routes(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
    width: f32,
) {
    draw_text_block(
        "ROUTE POLICY · repeat workers",
        sheet.x + 16.0,
        sheet.y + 236.0,
        sheet.w - 32.0,
        18.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    for (index, kind) in [ZoneKind::Work, ZoneKind::Storage, ZoneKind::Patrol]
        .into_iter()
        .enumerate()
    {
        if compact_virtual_button(
            Rect::new(
                sheet.x + 16.0 + index as f32 * (width + 5.0),
                sheet.y + 256.0,
                width,
                44.0,
            ),
            ctx.session.world.route_policies.for_kind(kind).label(),
            ctx.session.phase == GamePhase::Playing,
            ButtonTone::Secondary,
            11.0,
            pointer,
        ) {
            actions.push(UiAction::CycleRoutePolicy(kind));
        }
    }
    draw_text_block(
        "Direct orders keep their existing route behavior.",
        sheet.x + 16.0,
        sheet.y + 306.0,
        sheet.w - 32.0,
        18.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
}
