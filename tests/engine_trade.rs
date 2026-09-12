use tiny_necromancer::data::*;
use tiny_necromancer::engine::trade::*;
use tiny_necromancer::state::*;
use tiny_necromancer::state::{Building, BuildingKind, GamePhase};

fn lantern_session(data: &GameData) -> GameSession {
    let mut session = GameSession::new(&data.config);
    session.phase = GamePhase::Playing;
    session.world.buildings.push(Building {
        kind: BuildingKind::GraveLantern,
        progress: 12.0,
        complete: true,
        position: data
            .config
            .world_layout
            .building_position(BuildingKind::GraveLantern.id())
            .expect("validated position"),
        width: 1,
        height: 1,
    });
    session
}
#[test]
fn market_rotates_and_wraps_after_elapsed_time() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
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
fn broker_request_locks_the_offer_and_cannot_stack() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
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
    assert_eq!(contract_progress(&session), Some(1.0));
    session
        .progress
        .market
        .contract
        .as_mut()
        .unwrap()
        .remaining_seconds = 37.5;
    assert!((contract_progress(&session).unwrap() - 0.5).abs() < 0.001);
    assert!(accept_contract(&mut session).is_err());
}
#[test]
fn fulfilling_a_broker_request_grants_bonus_favor() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
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
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 40;
    assert!(execute_trade(&mut session, &data).is_err());
}
#[test]
fn market_save_defaults_and_normalizes_invalid_rotation() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut save = lantern_session(&data).to_save(&data.config.version);
    save.progress.market.offer_index = 99;
    save.progress.market.refresh_seconds = f32::NAN;
    let restored = GameSession::from_save(save, &data.config);
    assert_eq!(restored.progress.market.offer_index, 0);
    assert!((restored.progress.market.refresh_seconds - 30.0).abs() < 0.001);
}
