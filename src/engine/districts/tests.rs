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

    assert_eq!(work_speed_multiplier(&session, work_tile), 1.0);
    assert_eq!(haul_capacity_bonus(&session), 0);
    assert_eq!(guard_mitigation_multiplier(&session), 1.0);

    session.research.completed = vec![Technology::DomainStewardship];

    assert_eq!(
        work_speed_multiplier(&session, work_tile),
        WORK_SPEED_MULTIPLIER
    );
    assert_eq!(haul_capacity_bonus(&session), STORAGE_CAPACITY_BONUS);
    assert_eq!(
        guard_mitigation_multiplier(&session),
        PATROL_MITIGATION_MULTIPLIER
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

    assert_eq!(rule_summary(&session), "Rules: Storage +4 haul");
}
