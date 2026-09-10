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
        haul_capacity_bonus(&session, &data.config.district_rules),
        0
    );
    assert_eq!(
        guard_mitigation_multiplier(&session, &data.config.district_rules),
        1.0
    );

    session.research.completed = vec![Technology::DomainStewardship];

    assert_eq!(
        work_speed_multiplier(&session, &data.config.district_rules, work_tile),
        data.config.district_rules.work_speed_multiplier
    );
    assert_eq!(
        haul_capacity_bonus(&session, &data.config.district_rules),
        data.config.district_rules.storage_capacity_bonus
    );
    assert_eq!(
        guard_mitigation_multiplier(&session, &data.config.district_rules),
        data.config.district_rules.patrol_mitigation_multiplier
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
        "Rules: Storage +4 haul"
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
        haul_capacity_bonus(&session, &data.config.district_rules),
        0
    );
    assert_eq!(
        guard_mitigation_multiplier(&session, &data.config.district_rules),
        1.0
    );
    assert!(rule_summary(&session, &data.config.district_rules).starts_with("No district rules"));
}
