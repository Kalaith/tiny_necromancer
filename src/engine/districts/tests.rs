use super::*;
use crate::state::WorldState;

#[test]
fn district_rules_wait_for_domain_stewardship() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let work_tile = session.world.plots[0].position;
    let patrol_tile = WorldState::guard_position(session.world.road_x);
    session.world.zones = vec![
        crate::state::Zone {
            kind: ZoneKind::Work,
            tiles: vec![work_tile],
        },
        crate::state::Zone {
            kind: ZoneKind::Storage,
            tiles: vec![WorldState::stockpile_position()],
        },
        crate::state::Zone {
            kind: ZoneKind::Patrol,
            tiles: vec![patrol_tile],
        },
    ];

    assert_eq!(
        work_speed_multiplier(&session, &data.config.district_rules, work_tile),
        1.0
    );
    assert_eq!(
        haul_capacity_bonus(
            &session,
            &data.config.district_rules,
            WorldState::stockpile_position(),
        ),
        0
    );
    assert_eq!(
        guard_mitigation_multiplier(&session, &data.config.district_rules, patrol_tile),
        1.0
    );

    session.research.completed = vec![Technology::DomainStewardship];

    assert_eq!(
        work_speed_multiplier(&session, &data.config.district_rules, work_tile),
        data.config.district_rules.work_speed_multiplier
    );
    assert_eq!(
        haul_capacity_bonus(
            &session,
            &data.config.district_rules,
            WorldState::stockpile_position(),
        ),
        data.config.district_rules.storage_capacity_bonus
    );
    assert_eq!(
        guard_mitigation_multiplier(&session, &data.config.district_rules, patrol_tile),
        data.config.district_rules.patrol_mitigation_multiplier
    );
    let unmarked_tile = TilePos::new(4, 4);
    assert_eq!(
        haul_capacity_bonus(&session, &data.config.district_rules, unmarked_tile),
        0
    );
    assert_eq!(
        guard_mitigation_multiplier(&session, &data.config.district_rules, unmarked_tile),
        1.0
    );
}

#[test]
fn rule_summary_names_only_marked_districts() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: ZoneKind::Storage,
        tiles: vec![WorldState::stockpile_position()],
    });

    assert_eq!(
        rule_summary(&session, &data.config.district_rules),
        "Rules: marked tiles · Storage +4"
    );
}

#[test]
fn empty_districts_keep_original_job_values() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    let tile = session.world.plots[0].position;

    assert_eq!(
        work_speed_multiplier(&session, &data.config.district_rules, tile),
        1.0
    );
    assert_eq!(
        haul_capacity_bonus(&session, &data.config.district_rules, tile),
        0
    );
    assert_eq!(
        guard_mitigation_multiplier(&session, &data.config.district_rules, tile),
        1.0
    );
    assert!(rule_summary(&session, &data.config.district_rules).starts_with("No district rules"));
}

#[test]
fn district_ledger_records_effects_and_first_use_notes() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);

    assert_eq!(
        ledger_summary(&session),
        "Ledger: no marked district effect recorded yet."
    );
    record_work_cycle(&mut session);
    record_work_cycle(&mut session);
    record_storage_bonus(&mut session, 4);
    record_patrol_quieting(&mut session, 0.6);
    record_patrol_quieting(&mut session, 0.2);

    assert_eq!(session.progress.district_ledger.work_cycles, 2);
    assert_eq!(session.progress.district_ledger.storage_bonus_items, 4);
    assert!((session.progress.district_ledger.patrol_quieting - 0.8).abs() < 0.001);
    assert_eq!(
        ledger_summary(&session),
        "Ledger: Work 2 cycles · Storage +4 haul · Patrol 0.8 quieted"
    );
    assert_eq!(session.pressure.feed.len(), 4);
    assert!(session
        .pressure
        .feed
        .iter()
        .any(|entry| entry.message.contains("marked Work")));
}
