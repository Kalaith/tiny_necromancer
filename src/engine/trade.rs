//! Lantern-gated exchanges with the rotating night market.

use crate::data::GameData;
use crate::engine::{districts, suspicion};
use crate::state::{
    BuildingKind, GameSession, MarketContract, MARKET_CONTRACT_BONUS_FAVOR, MARKET_CONTRACT_SECONDS,
};

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
    session.progress.market.contract.as_ref().map_or_else(
        || offer_at(session.progress.market.offer_index),
        |contract| offer_at(contract.offer_index),
    )
}

pub fn offer_position(session: &GameSession) -> (usize, usize) {
    let index = session
        .progress
        .market
        .contract
        .as_ref()
        .map_or(session.progress.market.offer_index, |contract| {
            contract.offer_index
        });
    (index % OFFERS.len() + 1, OFFERS.len())
}

fn offer_at(index: usize) -> TradeOffer {
    OFFERS[index % OFFERS.len()]
}

pub fn exchange_suspicion(session: &GameSession) -> f32 {
    match session.progress.market.standing_tier() {
        0 => 2.0,
        1 => 1.5,
        _ => 1.0,
    }
}

pub fn advance_market(session: &mut GameSession, dt: f32) -> Option<String> {
    if !dt.is_finite() || dt <= 0.0 {
        return None;
    }
    let mut rotated = false;
    session.progress.market.refresh_seconds -= dt;
    while session.progress.market.refresh_seconds <= 0.0 {
        session.progress.market.offer_index =
            (session.progress.market.offer_index + 1) % OFFERS.len();
        session.progress.market.refresh_seconds += session.progress.market.refresh_interval();
        rotated = true;
    }
    rotated.then(|| {
        format!(
            "Night market offer changed: {}.",
            offer_at(session.progress.market.offer_index).title
        )
    })
}

pub fn advance_contract(session: &mut GameSession, dt: f32) -> Option<String> {
    if !dt.is_finite() || dt <= 0.0 {
        return None;
    }
    let contract = session.progress.market.contract.as_mut()?;
    contract.remaining_seconds -= dt;
    if contract.remaining_seconds > 0.0 {
        return None;
    }
    let offer = offer_at(contract.offer_index);
    session.progress.market.contract = None;
    Some(format!(
        "Broker request expired: {} was left unfulfilled.",
        offer.title
    ))
}

pub fn accept_contract(session: &mut GameSession) -> Result<(), String> {
    if !session.has_building(BuildingKind::GraveLantern) {
        return Err("Complete the grave lantern before accepting a broker request.".to_owned());
    }
    if session.progress.market.contract.is_some() {
        return Err("A broker request is already active.".to_owned());
    }
    session.progress.market.contract = Some(MarketContract {
        offer_index: session.progress.market.offer_index,
        remaining_seconds: MARKET_CONTRACT_SECONDS,
        bonus_favor: MARKET_CONTRACT_BONUS_FAVOR,
    });
    let offer = current_offer(session);
    session.add_feed(format!(
        "Broker request accepted: fulfill {} for +{} favor.",
        offer.title, MARKET_CONTRACT_BONUS_FAVOR
    ));
    Ok(())
}

pub fn execute_trade(session: &mut GameSession, data: &GameData) -> Result<(), String> {
    trade_status(session, data)?;
    let offer = current_offer(session);
    let contract = session.progress.market.contract.take();
    session.economy.bones -= offer.bones_cost;
    session.economy.wood -= offer.wood_cost;
    match offer.kind {
        TradeOfferKind::CarvedTimber => session.economy.wood += 12,
        TradeOfferKind::LanternDraught => session.economy.mana += 8,
        TradeOfferKind::QuietBargain => session.economy.ward_charges += 1,
    }
    session.progress.market.completed_trades += 1;
    let previous_tier = session.progress.market.standing_tier();
    let favor_gain = 1 + contract.as_ref().map_or(0, |active| active.bonus_favor);
    session.progress.market.favor = session.progress.market.favor.saturating_add(favor_gain);
    let suspicion_cost = exchange_suspicion(session);
    suspicion::adjust(session, suspicion_cost, "a discreet night market exchange");
    if session.progress.market.standing_tier() > previous_tier {
        session.add_feed(format!(
            "The broker now calls this cemetery {}.",
            session.progress.market.standing_label()
        ));
    }
    if contract.is_some() {
        session.progress.market.completed_contracts += 1;
        session.add_feed(format!(
            "Broker request fulfilled: {} for {} · +{} favor.",
            offer.cost, offer.reward, favor_gain
        ));
    } else {
        session.add_feed(format!(
            "Night market exchange: {} for {}.",
            offer.cost, offer.reward
        ));
    }
    Ok(())
}

pub fn trade_status(session: &GameSession, data: &GameData) -> Result<(), String> {
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
    Ok(())
}

#[cfg(test)]
mod tests;
