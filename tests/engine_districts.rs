use macroquad_toolkit::grid::TilePos;
use tiny_necromancer::engine::districts::*;
use tiny_necromancer::state::*;
use tiny_necromancer::state::{Building, BuildingKind, WorldState};
#[test]
fn district_rules_wait_for_domain_stewardship() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let work_tile = session.world.plots[0].position;
    let patrol_tile = WorldState::guard_position(session.world.road_x);
    session.world.zones = vec![
        tiny_necromancer::state::Zone {
            kind: ZoneKind::Work,
            tiles: vec![work_tile],
        },
        tiny_necromancer::state::Zone {
            kind: ZoneKind::Storage,
            tiles: vec![session.world.stockpile_position()],
        },
        tiny_necromancer::state::Zone {
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
            session.world.stockpile_position(),
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
            session.world.stockpile_position(),
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
fn operations_summary_names_staffing_by_district() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let work_tile = session.world.plots[0].position;
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones = vec![
        tiny_necromancer::state::Zone {
            kind: ZoneKind::Work,
            tiles: vec![work_tile],
        },
        tiny_necromancer::state::Zone {
            kind: ZoneKind::Patrol,
            tiles: vec![WorldState::guard_position(session.world.road_x)],
        },
    ];

    assert_eq!(
        operations_summary(&session),
        "Staffing (workers/marks): Work 1/1 · Storage 0/0 · Patrol 0/1 post · Needs staff (1 slot)."
    );
    assert_eq!(
        compact_operations_summary(&session),
        "Staffing: W 1/1 · S 0/0 · P 0/1 posts · GAP 1"
    );
    assert!(staffing_needs_attention(&session));
    assert_eq!(total_staffing_gap(&session), 1);
    let fresh_session = GameSession::new(&data.config);
    assert!(!staffing_needs_attention(&fresh_session));
    assert_eq!(
        operations_summary(&fresh_session),
        "Staffing (workers/marks): Work 1/0 · Storage 0/0 · Patrol 0/0 posts · Ready."
    );
    assert_eq!(
        compact_operations_summary(&fresh_session),
        "Staffing: W 1/0 · S 0/0 · P 0/0 posts · OK"
    );

    let mut locked_session = GameSession::new(&data.config);
    locked_session
        .world
        .zones
        .push(tiny_necromancer::state::Zone {
            kind: ZoneKind::Work,
            tiles: vec![work_tile],
        });
    assert!(!staffing_needs_attention(&locked_session));
}
#[test]
fn service_coverage_separates_assignment_from_route_access() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(tiny_necromancer::state::Zone {
        kind: ZoneKind::Storage,
        tiles: vec![session.world.stockpile_position()],
    });
    session.workforce.workers[0].assignment = JobKind::Haul;

    assert_eq!(
        service_coverage(&session, ZoneKind::Storage),
        DistrictCoverage {
            marked: 1,
            assigned: 1,
            reachable: 1,
        }
    );

    for y in 0..session.world.height as i32 {
        session.world.buildings.push(Building {
            kind: BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position: TilePos::new(3, y),
            width: 1,
            height: 1,
        });
    }

    assert_eq!(
        service_coverage(&session, ZoneKind::Storage),
        DistrictCoverage {
            marked: 1,
            assigned: 1,
            reachable: 0,
        }
    );
    assert!(route_coverage_needs_attention(&session));
    assert_eq!(route_gap_district(&session, 0), Some(ZoneKind::Storage));
    assert_eq!(route_gap_district(&session, 99), None);
}
#[test]
fn marked_tile_summary_keeps_overlapping_district_marks_visible() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let tile = session.world.plots[0].position;
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones = vec![
        tiny_necromancer::state::Zone {
            kind: ZoneKind::Work,
            tiles: vec![tile],
        },
        tiny_necromancer::state::Zone {
            kind: ZoneKind::Storage,
            tiles: vec![tile],
        },
    ];

    let summary = tile_summary(&session, &data.config.district_rules, tile)
        .expect("overlapping district marks should be inspectable");

    assert!(summary.contains("Work district"));
    assert!(summary.contains("+15% Dig/Wood speed here."));
    assert!(summary.contains("Storage district"));
    assert!(summary.contains("+4 Haul · +24 storage here."));
    assert!(summary.contains("\n"));
}
#[test]
fn district_ledger_records_effects_and_first_use_notes() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);

    assert_eq!(
        ledger_summary(&session),
        "Ledger: no marked district effect recorded yet."
    );
    record_work_cycle(&mut session);
    record_work_cycle(&mut session);
    session.progress.elapsed_seconds = 12.0;
    record_storage_bonus(&mut session, 4);
    session.progress.elapsed_seconds = 18.0;
    record_patrol_quieting(&mut session, 0.6);
    record_patrol_quieting(&mut session, 0.2);

    assert_eq!(session.progress.district_ledger.work_cycles, 2);
    assert_eq!(session.progress.district_ledger.storage_bonus_items, 4);
    assert!((session.progress.district_ledger.patrol_quieting - 0.8).abs() < 0.001);
    assert_eq!(session.progress.district_ledger.recent_activity.len(), 5);
    assert_eq!(
        session.progress.district_ledger.recent_activity[0].kind,
        DistrictActivityKind::PatrolQuieting
    );
    assert!((session.progress.district_ledger.recent_activity[0].amount - 0.2).abs() < 0.001);
    assert_eq!(
        ledger_summary(&session),
        "Ledger: Work 2 cycles · Storage +4 haul · Patrol -0.8 suspicion"
    );
    assert_eq!(
        activity_summary(&session),
        "Activity: Patrol -0.2 suspicion @ 18s · Patrol -0.6 suspicion @ 18s"
    );
    assert_eq!(
        latest_activity_summary(&session),
        "Last: Patrol -0.2 suspicion @ 18s"
    );
    assert_eq!(session.pressure.feed.len(), 4);
    assert!(session
        .pressure
        .feed
        .iter()
        .any(|entry| entry.message.contains("marked Work")));
    assert!(session
        .pressure
        .feed
        .iter()
        .any(|entry| entry.message.contains("first pickup")));
}
