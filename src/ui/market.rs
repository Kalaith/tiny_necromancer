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
    draw_text_block(
        &standing_summary(ctx),
        rect.x + 24.0,
        rect.y + 76.0,
        420.0,
        18.0,
        12.0,
        0.0,
        dark::ACCENT,
    );
    progress_bar(
        rect.x + 486.0,
        rect.y + 82.0,
        166.0,
        7.0,
        ctx.session.progress.market.standing_progress(),
        1.0,
        dark::ACCENT,
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
    let trade_ready = ctx.session.phase == crate::state::GamePhase::Playing
        && trade::trade_status(ctx.session, ctx.data).is_ok();
    let trade_label = if trade_ready {
        format!("Make exchange · {}", offer.cost)
    } else {
        "Exchange unavailable".to_owned()
    };
    if virtual_button(
        Rect::new(rect.x + 24.0, rect.y + 284.0, 300.0, 52.0),
        &trade_label,
        trade_ready,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::ExecuteTrade);
    }
    let contract_active = ctx.session.progress.market.contract.is_some();
    let contract_label = ctx.session.progress.market.contract.as_ref().map_or_else(
        || {
            format!(
                "Take request · +{} favor",
                crate::state::MARKET_CONTRACT_BONUS_FAVOR
            )
        },
        |contract| format!("Request active · {:.0}s", contract.remaining_seconds),
    );
    if virtual_button(
        Rect::new(rect.x + 340.0, rect.y + 284.0, 300.0, 52.0),
        &contract_label,
        !contract_active && ctx.session.phase == crate::state::GamePhase::Playing,
        ButtonTone::Secondary,
        pointer,
    ) {
        actions.push(UiAction::AcceptMarketContract);
    }
    draw_text_block(
        &trade_status_label(ctx, trade_ready),
        rect.x + 24.0,
        rect.y + 350.0,
        rect.w - 48.0,
        28.0,
        13.0,
        3.0,
        if trade_ready {
            dark::POSITIVE
        } else {
            dark::WARNING
        },
    );
    draw_text_block(
        &format!(
            "{} exchanges · {} requests fulfilled · +{:.1} suspicion at this standing",
            ctx.session.progress.market.completed_trades,
            ctx.session.progress.market.completed_contracts,
            trade::exchange_suspicion(ctx.session)
        ),
        rect.x + 24.0,
        rect.y + 392.0,
        rect.w - 48.0,
        20.0,
        13.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn draw_offer(ctx: &UiContext<'_>, rect: Rect) {
    let offer = trade::current_offer(ctx.session);
    let (offer_number, offer_count) = trade::offer_position(ctx.session);
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
        &format!("OFFER {offer_number}/{offer_count}"),
        card.right() - 112.0,
        card.y + 20.0,
        94.0,
        18.0,
        11.0,
        0.0,
        dark::ACCENT,
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
    draw_text_block(
        &standing_summary(ctx),
        sheet.x + 16.0,
        sheet.y + 104.0,
        sheet.w - 32.0,
        18.0,
        11.0,
        0.0,
        dark::ACCENT,
    );
    progress_bar(
        sheet.x + 16.0,
        sheet.y + 122.0,
        sheet.w - 32.0,
        6.0,
        ctx.session.progress.market.standing_progress(),
        1.0,
        dark::ACCENT,
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
    let (offer_number, offer_count) = trade::offer_position(ctx.session);
    let trade_ready = ctx.session.phase == crate::state::GamePhase::Playing
        && trade::trade_status(ctx.session, ctx.data).is_ok();
    let trade_label = if trade_ready {
        format!("Make exchange · {}", offer.cost)
    } else {
        "Exchange unavailable".to_owned()
    };
    let card = Rect::new(sheet.x + 16.0, sheet.y + 132.0, sheet.w - 32.0, 132.0);
    draw_surface(
        card,
        &SurfaceStyle::new(Color::new(0.12, 0.085, 0.15, 1.0))
            .with_border(1.0, Color::new(0.56, 0.43, 0.68, 0.70)),
    );
    draw_text_block(
        offer.title,
        card.x + 12.0,
        card.y + 12.0,
        card.w - 120.0,
        20.0,
        16.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        &format!("{offer_number}/{offer_count}"),
        card.right() - 82.0,
        card.y + 14.0,
        70.0,
        18.0,
        11.0,
        0.0,
        dark::ACCENT,
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
        card.y + 70.0,
        card.w - 24.0,
        18.0,
        12.0,
        0.0,
        dark::ACCENT,
    );
    let contract_active = ctx.session.progress.market.contract.is_some();
    let contract_label = ctx.session.progress.market.contract.as_ref().map_or_else(
        || {
            format!(
                "Take request · +{} favor",
                crate::state::MARKET_CONTRACT_BONUS_FAVOR
            )
        },
        |contract| format!("Request active · {:.0}s", contract.remaining_seconds),
    );
    if compact_virtual_button(
        Rect::new(card.x + 12.0, card.y + 92.0, card.w - 24.0, 34.0),
        &contract_label,
        !contract_active && ctx.session.phase == crate::state::GamePhase::Playing,
        ButtonTone::Secondary,
        10.0,
        pointer,
    ) {
        actions.push(UiAction::AcceptMarketContract);
    }
    if compact_virtual_button(
        Rect::new(sheet.x + 16.0, sheet.y + 280.0, sheet.w - 32.0, 48.0),
        &trade_label,
        trade_ready,
        ButtonTone::Positive,
        12.0,
        pointer,
    ) {
        actions.push(UiAction::ExecuteTrade);
    }
    draw_text_block(
        &trade_status_label(ctx, trade_ready),
        sheet.x + 16.0,
        sheet.y + 338.0,
        sheet.w - 32.0,
        22.0,
        11.0,
        2.0,
        if trade_ready {
            dark::POSITIVE
        } else {
            dark::WARNING
        },
    );
    draw_text_block(
        &format!(
            "Next in {:.0}s · {} completed",
            ctx.session.progress.market.refresh_seconds,
            ctx.session.progress.market.completed_trades
        ),
        sheet.x + 16.0,
        sheet.y + 368.0,
        sheet.w - 32.0,
        18.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn trade_status_label(ctx: &UiContext<'_>, ready: bool) -> String {
    if ready {
        "Offer ready · the broker leaves when the timer turns.".to_owned()
    } else if ctx.session.phase != crate::state::GamePhase::Playing {
        "Resume the cemetery before making an exchange.".to_owned()
    } else {
        format!(
            "Waiting · {}",
            trade::trade_status(ctx.session, ctx.data)
                .expect_err("unavailable market offer should explain its blocker")
        )
    }
}

fn standing_summary(ctx: &UiContext<'_>) -> String {
    let market = &ctx.session.progress.market;
    match market.next_standing_target() {
        Some(target) => format!(
            "Broker standing · {} · favor {}/{} · cycle {:.0}s",
            market.standing_label(),
            market.favor,
            target,
            market.refresh_interval()
        ),
        None => format!(
            "Broker standing · {} · favor {} · cycle {:.0}s",
            market.standing_label(),
            market.favor,
            market.refresh_interval()
        ),
    }
}
