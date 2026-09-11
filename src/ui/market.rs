//! Full and compact views for the rotating night market.

use super::components::{compact_virtual_button, virtual_button};
use super::{UiAction, UiContext};
use crate::engine::trade;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_market_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let rect = Rect::new(238.0, 106.0, 680.0, 490.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.075, 0.055, 0.09, 0.98))
            .with_border(1.0, Color::new(0.68, 0.56, 0.76, 0.80)),
    );
    draw_text_block(
        "NIGHT MARKET",
        rect.x + 24.0,
        rect.y + 20.0,
        250.0,
        24.0,
        19.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        "The grave lantern opens a door to quiet, temporary bargains.",
        rect.x + 24.0,
        rect.y + 52.0,
        500.0,
        20.0,
        14.0,
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
        actions.push(UiAction::TogglePanel(super::Panel::None));
    }
    draw_offer(ctx, rect);
    let offer = trade::current_offer(ctx.session);
    if virtual_button(
        Rect::new(rect.x + 24.0, rect.y + 284.0, 300.0, 52.0),
        &format!("Make exchange · {}", offer.cost),
        ctx.session.phase == crate::state::GamePhase::Playing,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::ExecuteTrade);
    }
    draw_text_block(
        &format!(
            "{} exchanges completed · +2 suspicion per bargain",
            ctx.session.progress.market.completed_trades
        ),
        rect.x + 24.0,
        rect.y + 360.0,
        rect.w - 48.0,
        20.0,
        13.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn draw_offer(ctx: &UiContext<'_>, rect: Rect) {
    let offer = trade::current_offer(ctx.session);
    let card = Rect::new(rect.x + 24.0, rect.y + 92.0, rect.w - 48.0, 168.0);
    draw_surface(
        card,
        &SurfaceStyle::new(Color::new(0.12, 0.085, 0.15, 1.0))
            .with_border(1.0, Color::new(0.56, 0.43, 0.68, 0.70)),
    );
    draw_text_block(
        offer.title,
        card.x + 18.0,
        card.y + 16.0,
        310.0,
        26.0,
        21.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        offer.detail,
        card.x + 18.0,
        card.y + 52.0,
        card.w - 36.0,
        22.0,
        14.0,
        0.0,
        dark::TEXT,
    );
    draw_text_block(
        &format!("Pay · {}", offer.cost),
        card.x + 18.0,
        card.y + 98.0,
        240.0,
        20.0,
        14.0,
        0.0,
        dark::WARNING,
    );
    draw_text_block(
        &format!("Receive · {}", offer.reward),
        card.x + 300.0,
        card.y + 98.0,
        260.0,
        20.0,
        14.0,
        0.0,
        dark::POSITIVE,
    );
    draw_text_block(
        &format!(
            "Next offer in {:.0}s",
            ctx.session.progress.market.refresh_seconds
        ),
        card.x + 18.0,
        card.y + 132.0,
        260.0,
        18.0,
        12.0,
        0.0,
        dark::ACCENT,
    );
}

pub(super) fn draw_compact_market_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    sheet: Rect,
) {
    draw_text_block(
        "NIGHT MARKET",
        sheet.x + 16.0,
        sheet.y + 84.0,
        sheet.w - 110.0,
        22.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    if compact_virtual_button(
        Rect::new(sheet.right() - 92.0, sheet.y + 80.0, 76.0, 40.0),
        "Close",
        true,
        ButtonTone::Secondary,
        11.0,
        pointer,
    ) {
        actions.push(UiAction::TogglePanel(super::Panel::None));
    }
    let offer = trade::current_offer(ctx.session);
    let card = Rect::new(sheet.x + 16.0, sheet.y + 122.0, sheet.w - 32.0, 132.0);
    draw_surface(
        card,
        &SurfaceStyle::new(Color::new(0.12, 0.085, 0.15, 1.0))
            .with_border(1.0, Color::new(0.56, 0.43, 0.68, 0.70)),
    );
    draw_text_block(
        offer.title,
        card.x + 12.0,
        card.y + 12.0,
        card.w - 24.0,
        20.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        offer.detail,
        card.x + 12.0,
        card.y + 40.0,
        card.w - 24.0,
        34.0,
        12.0,
        3.0,
        dark::TEXT,
    );
    draw_text_block(
        &format!("Pay {}  ·  receive {}", offer.cost, offer.reward),
        card.x + 12.0,
        card.y + 88.0,
        card.w - 24.0,
        18.0,
        12.0,
        0.0,
        dark::ACCENT,
    );
    if compact_virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 270.0, sheet.w - 32.0, 48.0),
        &format!("Make exchange · {}", offer.cost),
        ctx.session.phase == crate::state::GamePhase::Playing,
        ButtonTone::Positive,
        12.0,
        pointer,
    ) {
        actions.push(UiAction::ExecuteTrade);
    }
    draw_text_block(
        &format!(
            "Next in {:.0}s · {} completed",
            ctx.session.progress.market.refresh_seconds,
            ctx.session.progress.market.completed_trades
        ),
        sheet.x + 16.0,
        sheet.y + 332.0,
        sheet.w - 32.0,
        18.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
}
