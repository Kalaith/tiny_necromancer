use super::*;

#[test]
fn digging_completes_and_produces_loose_bones() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.begin();
    simulate(&mut session, &data, 8.0);
    assert_eq!(session.world.plots[0].status, PlotStatus::Dug);
    assert_eq!(session.economy.loose_bones, 8);
    assert!(session.pressure.suspicion > 0.0);
}

#[test]
fn hauling_respects_worker_capacity() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.begin();
    session.economy.loose_bones = 20;
    session.workforce.workers[0].assignment = JobKind::Haul;
    simulate(&mut session, &data, 4.0);
    assert_eq!(session.economy.loose_bones, 12);
    assert_eq!(session.economy.bones, data.config.starting_bones + 8);
}

#[test]
fn same_seed_produces_same_work_output() {
    let data = crate::data::GameData::load().unwrap();
    let mut first = GameSession::new(&data.config);
    let mut second = GameSession::new(&data.config);
    first.begin();
    second.begin();
    simulate(&mut first, &data, 8.0);
    simulate(&mut second, &data, 8.0);
    assert_eq!(first.economy.loose_bones, second.economy.loose_bones);
    assert_eq!(first.economy.corpses.len(), second.economy.corpses.len());
    assert_eq!(first.rng.state(), second.rng.state());
}

#[test]
fn auto_mode_chooses_haul_before_more_digging() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.begin();
    session.economy.loose_bones = 8;
    toggle_automation(&mut session).unwrap();
    simulate(&mut session, &data, 4.0);
    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
    assert_eq!(session.economy.loose_bones, 0);
    assert_eq!(session.economy.bones, data.config.starting_bones + 8);
}

#[test]
fn guarding_mitigates_suspicion_without_flooding_the_feed() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.pressure.suspicion = 10.0;
    session.workforce.workers[0].assignment = JobKind::Guard;
    let feed_len = session.pressure.feed.len();
    simulate(&mut session, &data, 1.0);
    assert!(session.pressure.suspicion < 10.0);
    assert_eq!(session.pressure.feed.len(), feed_len);
}
