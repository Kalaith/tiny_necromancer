use super::*;
use crate::state::{Building, BuildingKind, GamePhase};

fn lantern_session(data: &GameData) -> GameSession {
    let mut session = GameSession::new(&data.config);
    session.phase = GamePhase::Playing;
    session.world.buildings.push(Building {
        kind: BuildingKind::GraveLantern,
        progress: 12.0,
        complete: true,
        position: crate::state::default_building_position_for_kind(BuildingKind::GraveLantern),
        width: 1,
        height: 1,
    });
    session
}

#[test]
fn market_rotates_and_wraps_after_elapsed_time() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    let message = advance_market(&mut session, 30.0);
    assert_eq!(session.progress.market.offer_index, 1);
    assert!((session.progress.market.refresh_seconds - 30.0).abs() < 0.001);
    assert_eq!(
        message.as_deref(),
        Some("Night market offer changed: Lantern draught.")
    );
    advance_market(&mut session, 60.0);
    assert_eq!(session.progress.market.offer_index, 0);
    assert_eq!(offer_position(&session), (1, 3));
}

#[test]
fn trusted_broker_refreshes_the_next_offer_faster() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    session.progress.market.favor = 8;
    session.progress.market.refresh_seconds = 1.0;
    advance_market(&mut session, 1.0);
    assert!((session.progress.market.refresh_seconds - 24.0).abs() < 0.001);
}

#[test]
fn broker_request_locks_the_offer_and_cannot_stack() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    accept_contract(&mut session).unwrap();
    assert_eq!(current_offer(&session).kind, TradeOfferKind::CarvedTimber);
    assert_eq!(
        session
            .progress
            .market
            .contract
            .as_ref()
            .unwrap()
            .remaining_seconds,
        75.0
    );
    assert!(accept_contract(&mut session).is_err());
}

#[test]
fn ignored_broker_request_expires_and_clears() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    accept_contract(&mut session).unwrap();
    let message = advance_contract(&mut session, 75.0);
    assert!(session.progress.market.contract.is_none());
    assert_eq!(
        message.as_deref(),
        Some("Broker request expired: Carved timber was left unfulfilled.")
    );
}

#[test]
fn fulfilling_a_broker_request_grants_bonus_favor() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    session.economy.bones = 30;
    accept_contract(&mut session).unwrap();
    execute_trade(&mut session, &data).unwrap();
    assert!(session.progress.market.contract.is_none());
    assert_eq!(session.progress.market.completed_contracts, 1);
    assert_eq!(session.progress.market.favor, 3);
    assert!(session.pressure.feed[0]
        .message
        .contains("Broker request fulfilled: 18 bones for 12 wood"));
}

#[test]
fn market_requires_a_complete_lantern() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 40;
    assert!(execute_trade(&mut session, &data).is_err());
}

#[test]
fn carved_timber_exchange_respects_storage_and_pays_reward() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    session.economy.bones = 30;
    session.economy.wood = 80;
    let result = execute_trade(&mut session, &data);
    assert!(result.is_err());
    assert_eq!(session.economy.bones, 30);
    assert_eq!(session.economy.wood, 80);

    session.economy.wood = 20;
    execute_trade(&mut session, &data).unwrap();
    assert_eq!(session.economy.bones, 12);
    assert_eq!(session.economy.wood, 32);
    assert_eq!(session.progress.market.completed_trades, 1);
    assert!(session.pressure.feed[0]
        .message
        .contains("Night market exchange: 18 bones for 12 wood"));
    assert_eq!(session.progress.market.favor, 1);
}

#[test]
fn trusted_broker_reduces_exchange_suspicion() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    session.progress.market.favor = 8;
    session.economy.bones = 30;
    execute_trade(&mut session, &data).unwrap();
    assert!((session.pressure.suspicion - 1.0).abs() < 0.001);
    assert_eq!(session.progress.market.favor, 9);
}

#[test]
fn standing_milestone_adds_a_broker_note() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    session.progress.market.favor = 2;
    session.economy.bones = 30;
    execute_trade(&mut session, &data).unwrap();
    assert!(session.pressure.feed[1]
        .message
        .contains("broker now calls this cemetery Acquainted"));
}

#[test]
fn market_status_names_the_blocker_without_spending_materials() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = lantern_session(&data);
    session.economy.bones = 10;
    let status = trade_status(&session, &data).unwrap_err();
    assert!(status.contains("18 bones"));
    assert_eq!(session.economy.bones, 10);
}

#[test]
fn market_save_defaults_and_normalizes_invalid_rotation() {
    let data = crate::data::GameData::load().unwrap();
    let mut save = lantern_session(&data).to_save(&data.config.version);
    save.progress.market.offer_index = 99;
    save.progress.market.refresh_seconds = f32::NAN;
    let restored = GameSession::from_save(save);
    assert_eq!(restored.progress.market.offer_index, 0);
    assert!((restored.progress.market.refresh_seconds - 30.0).abs() < 0.001);
}
