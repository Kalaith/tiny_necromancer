//! Compact status cards and pause control.

use super::*;

pub(super) fn draw_compact_status(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let width = ctx.layout.logical_width;
    let pause = Rect::new(width - 88.0, 8.0, 80.0, 48.0);
    let values = [
        ("BONES", ctx.session.economy.bones, dark::TEXT_BRIGHT),
        (
            "MANA",
            ctx.session.economy.mana,
            Color::new(0.73, 0.79, 1.0, 1.0),
        ),
        (
            "WOOD",
            ctx.session.economy.wood,
            Color::new(0.86, 0.68, 0.40, 1.0),
        ),
    ];
    let show_undead_card = width >= 400.0;
    let card_count = if show_undead_card { 4.0 } else { 3.0 };
    let card_width = ((width - 104.0) / card_count).max(72.0);
    draw_compact_resource_cards(values, card_width);
    if show_undead_card {
        draw_compact_undead_card(ctx, card_width);
    }
    if virtual_button(
        pause,
        if ctx.session.phase == GamePhase::Paused {
            "Resume"
        } else {
            "Pause"
        },
        true,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::TogglePause);
    }
    let suspicion = compact_suspicion_text(ctx, show_undead_card);
    let storage_full =
        crate::engine::districts::storage_space(ctx.session, &ctx.data.config.district_rules) == 0;
    draw_text_block(
        &suspicion,
        12.0,
        64.0,
        width - 24.0,
        18.0,
        12.0,
        0.0,
        if ctx.session.pressure.suspicion >= 50.0 || storage_full {
            dark::WARNING
        } else {
            dark::TEXT_DIM
        },
    );
}

fn draw_compact_resource_cards(values: [(&str, i32, Color); 3], card_width: f32) {
    for (index, (label, value, color)) in values.into_iter().enumerate() {
        let rect = Rect::new(8.0 + index as f32 * card_width, 8.0, card_width - 4.0, 48.0);
        draw_surface(
            rect,
            &SurfaceStyle::new(Color::new(0.055, 0.07, 0.065, 0.92))
                .with_border(1.0, Color::new(0.42, 0.54, 0.46, 0.60)),
        );
        draw_text_block(
            label,
            rect.x + 8.0,
            rect.y + 7.0,
            rect.w - 16.0,
            14.0,
            9.0,
            0.0,
            dark::TEXT_DIM,
        );
        draw_text_block(
            &value.to_string(),
            rect.x + 8.0,
            rect.y + 23.0,
            rect.w - 16.0,
            20.0,
            17.0,
            0.0,
            color,
        );
    }
}

fn draw_compact_undead_card(ctx: &UiContext<'_>, card_width: f32) {
    let rect = Rect::new(8.0 + 3.0 * card_width, 8.0, card_width - 4.0, 48.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.055, 0.07, 0.065, 0.92))
            .with_border(1.0, Color::new(0.42, 0.54, 0.46, 0.60)),
    );
    draw_text_block(
        "UNDEAD",
        rect.x + 8.0,
        rect.y + 7.0,
        rect.w - 16.0,
        14.0,
        9.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_text_block(
        &ctx.session.active_undead().to_string(),
        rect.x + 8.0,
        rect.y + 23.0,
        rect.w - 16.0,
        20.0,
        17.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
}

fn compact_suspicion_text(ctx: &UiContext<'_>, show_undead_card: bool) -> String {
    let storage_capacity =
        crate::engine::districts::storage_capacity(ctx.session, &ctx.data.config.district_rules);
    if show_undead_card {
        format!(
            "SUSPICION · {} {:.0}% · STORE {}/{}",
            super::super::components::stage_label(ctx.session.pressure.stage),
            ctx.session.pressure.suspicion,
            ctx.session.economy.stored_materials(),
            storage_capacity
        )
    } else {
        format!(
            "SUSPICION · {} {:.0}% · STORE {}/{} · UNDEAD {}",
            super::super::components::stage_label(ctx.session.pressure.stage),
            ctx.session.pressure.suspicion,
            ctx.session.economy.stored_materials(),
            storage_capacity,
            ctx.session.active_undead()
        )
    }
}
