//! Lantern-gated exchanges with the rotating night market.

use crate::data::GameData;
use crate::engine::{districts, suspicion};
use crate::state::{BuildingKind, GameSession, MarketState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeOfferKind {
    CarvedTimber,
    LanternDraught,
    QuietBargain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TradeOffer {
    pub kind: TradeOfferKind,
    pub title: &'static str,
    pub detail: &'static str,
    pub cost: &'static str,
    pub reward: &'static str,
    pub bones_cost: i32,
    pub wood_cost: i32,
}

const OFFERS: [TradeOffer; 3] = [
    TradeOffer {
        kind: TradeOfferKind::CarvedTimber,
        title: "Carved timber",
        detail: "The broker turns bone into sturdy boards.",
        cost: "18 bones",
        reward: "12 wood",
        bones_cost: 18,
        wood_cost: 0,
    },
    TradeOffer {
        kind: TradeOfferKind::LanternDraught,
        title: "Lantern draught",
        detail: "A blue tonic feeds the grave lantern's reserve.",
        cost: "18 wood",
        reward: "8 mana",
        bones_cost: 0,
        wood_cost: 18,
    },
    TradeOffer {
        kind: TradeOfferKind::QuietBargain,
        title: "Quiet bargain",
        detail: "A whispered fee buys one sealed ward charge.",
        cost: "24 bones + 12 wood",
        reward: "1 ward charge",
        bones_cost: 24,
        wood_cost: 12,
    },
];

pub fn current_offer(session: &GameSession) -> TradeOffer {
    OFFERS[session.progress.market.offer_index % OFFERS.len()]
}

pub fn advance_market(session: &mut GameSession, dt: f32) {
    if !dt.is_finite() || dt <= 0.0 {
        return;
    }
    session.progress.market.refresh_seconds -= dt;
    while session.progress.market.refresh_seconds <= 0.0 {
        session.progress.market.offer_index =
            (session.progress.market.offer_index + 1) % OFFERS.len();
        session.progress.market.refresh_seconds += MarketState::default().refresh_seconds;
    }
}

pub fn execute_trade(session: &mut GameSession, data: &GameData) -> Result<(), String> {
    if !session.has_building(BuildingKind::GraveLantern) {
        return Err("Complete the grave lantern before meeting the night broker.".to_owned());
    }
    let offer = current_offer(session);
    if session.economy.bones < offer.bones_cost || session.economy.wood < offer.wood_cost {
        return Err(format!("Need {} for this exchange.", offer.cost));
    }
    match offer.kind {
        TradeOfferKind::CarvedTimber => {
            let capacity = districts::storage_capacity(session, &data.config.district_rules);
            let after_cost = session.economy.stored_materials() - offer.bones_cost;
            if after_cost + 12 > capacity {
                return Err("Clear storage room before accepting more carved timber.".to_owned());
            }
        }
        TradeOfferKind::LanternDraught => {
            if session.economy.mana + 8 > data.config.max_mana {
                return Err("The mana reserve cannot hold that draught yet.".to_owned());
            }
        }
        TradeOfferKind::QuietBargain => {}
    }
    session.economy.bones -= offer.bones_cost;
    session.economy.wood -= offer.wood_cost;
    match offer.kind {
        TradeOfferKind::CarvedTimber => session.economy.wood += 12,
        TradeOfferKind::LanternDraught => session.economy.mana += 8,
        TradeOfferKind::QuietBargain => session.economy.ward_charges += 1,
    }
    session.progress.market.completed_trades += 1;
    session.add_feed(format!(
        "Night market exchange: {} for {}.",
        offer.cost, offer.reward
    ));
    suspicion::adjust(session, 2.0, "a discreet night market exchange");
    Ok(())
}

#[cfg(test)]
mod tests;
