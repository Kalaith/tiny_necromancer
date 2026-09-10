use super::*;

#[test]
fn suspicion_crossing_threshold_opens_the_matching_event() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    adjust(&mut session, 26.0, "test noise");
    update_stage(&mut session, &data);
    assert_eq!(session.pressure.stage, SuspicionStage::Rumour);
    assert_eq!(session.pressure.active_event.as_deref(), Some("rumour"));
}

#[test]
fn resolving_a_ward_choice_reduces_suspicion_and_closes_event() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.pressure.suspicion = 52.0;
    session.pressure.stage = SuspicionStage::Questioning;
    session.pressure.active_event = Some("questioning".to_owned());
    session.economy.mana = 20;
    resolve_event(&mut session, &data, "ward").unwrap();
    assert!(session.pressure.suspicion < 52.0);
    assert!(session.pressure.active_event.is_none());
    assert_eq!(session.economy.mana, 14);
}
