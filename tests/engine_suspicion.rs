use tiny_necromancer::data::*;
use tiny_necromancer::engine::suspicion::*;
use tiny_necromancer::state::*;

#[test]
fn suspicion_crossing_threshold_opens_the_matching_event() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    adjust(&mut session, 26.0, "test noise");
    update_stage(&mut session, &data);
    assert_eq!(session.pressure.stage, SuspicionStage::Rumour);
    assert_eq!(session.pressure.active_event.as_deref(), Some("rumour"));
}

#[test]
fn resolving_a_ward_choice_reduces_suspicion_and_closes_event() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
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

#[test]
fn pausing_digging_releases_the_interrupted_plot() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.world.plots[0].status = tiny_necromancer::state::PlotStatus::Digging;
    session.world.plots[0].progress = 3.0;
    session.workforce.workers[0].target_plot = Some(0);
    session.pressure.active_event = Some("rumour".to_owned());
    resolve_event(&mut session, &data, "pause").unwrap();
    assert_eq!(
        session.world.plots[0].status,
        tiny_necromancer::state::PlotStatus::Ready
    );
    assert_eq!(session.world.plots[0].progress, 0.0);
    assert_eq!(
        session.workforce.workers[0].assignment,
        tiny_necromancer::state::JobKind::Guard
    );
    assert_eq!(session.workforce.workers[0].target_plot, None);
}
