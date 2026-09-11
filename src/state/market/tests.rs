use super::*;
use crate::state::{GameSession, MarketContract};

#[test]
fn broker_standing_advances_at_persistent_favor_thresholds() {
    let mut market = MarketState::default();
    assert_eq!(market.standing_label(), "Whisper");
    assert_eq!(market.next_standing_target(), Some(3));

    market.favor = 3;
    assert_eq!(market.standing_label(), "Acquainted");
    assert_eq!(market.next_standing_target(), Some(8));

    market.favor = 8;
    assert_eq!(market.standing_label(), "Trusted");
    assert_eq!(market.next_standing_target(), None);
    assert_eq!(market.standing_progress(), 1.0);

    market.favor = 2;
    assert!((market.standing_progress() - (2.0 / 3.0)).abs() < 0.001);
    assert!((market.refresh_interval() - 30.0).abs() < 0.001);

    market.favor = 3;
    assert!((market.refresh_interval() - 27.0).abs() < 0.001);
    market.favor = 8;
    assert!((market.refresh_interval() - 24.0).abs() < 0.001);
}

#[test]
fn older_market_saves_infer_favor_from_completed_trades() {
    let mut market = MarketState {
        completed_trades: 5,
        ..MarketState::default()
    };
    market.normalize();
    assert_eq!(market.favor, 5);
    assert_eq!(market.standing_label(), "Acquainted");
}

#[test]
fn older_active_requests_restore_their_contract_defaults() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.progress.market.contract = Some(MarketContract {
        offer_index: 1,
        remaining_seconds: MARKET_CONTRACT_SECONDS,
        bonus_favor: MARKET_CONTRACT_BONUS_FAVOR,
    });
    let mut value = serde_json::to_value(session.to_save(&data.config.version)).unwrap();
    let contract = value
        .get_mut("progress")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|progress| progress.get_mut("market"))
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|market| market.get_mut("contract"))
        .and_then(serde_json::Value::as_object_mut)
        .expect("active contract object");
    contract.remove("remaining_seconds");
    contract.remove("bonus_favor");

    let restored = GameSession::from_save(serde_json::from_value(value).unwrap());
    let restored_contract = restored
        .progress
        .market
        .contract
        .expect("contract restored");
    assert!((restored_contract.remaining_seconds - MARKET_CONTRACT_SECONDS).abs() < 0.001);
    assert_eq!(restored_contract.bonus_favor, MARKET_CONTRACT_BONUS_FAVOR);
}
