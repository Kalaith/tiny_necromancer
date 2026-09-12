use macroquad_toolkit::grid::TilePos;
use tiny_necromancer::engine::jobs::*;
use tiny_necromancer::state::{
    Building, BuildingKind, GameSession, JobKind, RoutePolicy, StewardshipPolicy, Technology,
};
#[test]
fn harvest_policy_prefers_material_work_over_guarding() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.pressure.suspicion = data.config.suspicion_thresholds[1] + 0.1;
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
}
#[test]
fn harvest_policy_fills_a_marked_work_gap_before_unmarked_haul() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(tiny_necromancer::state::Zone {
        kind: tiny_necromancer::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
}
#[test]
fn harvest_policy_spreads_automated_workers_across_marked_gaps() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.extend([
        tiny_necromancer::state::Zone {
            kind: tiny_necromancer::state::ZoneKind::Work,
            tiles: vec![session.world.plots[0].position],
        },
        tiny_necromancer::state::Zone {
            kind: tiny_necromancer::state::ZoneKind::Storage,
            tiles: vec![session.world.stockpile_position()],
        },
    ]);
    session.economy.loose_bones = 8;
    let mut second_worker = session.workforce.workers[0].clone();
    second_worker.id = session.workforce.next_worker_id;
    second_worker.name = "Second Hand".to_owned();
    session.workforce.next_worker_id += 1;
    session.workforce.workers.push(second_worker);
    for worker in &mut session.workforce.workers {
        worker.priority_mode = true;
        worker.assignment = JobKind::Guard;
    }

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
    assert_eq!(session.workforce.workers[1].assignment, JobKind::Dig);
}
#[test]
fn automated_worker_falls_back_from_a_blocked_marked_forest() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Wood;
    session.workforce.priorities = vec![
        JobKind::Wood,
        JobKind::Haul,
        JobKind::Guard,
        JobKind::Dig,
        JobKind::Build,
        JobKind::Refine,
    ];
    session.world.zones.push(tiny_necromancer::state::Zone {
        kind: tiny_necromancer::state::ZoneKind::Work,
        tiles: vec![session.world.forest_tiles[0]],
    });
    session.economy.loose_bones = 8;
    for y in 0..session.world.height as i32 {
        session.world.buildings.push(Building {
            kind: BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position: TilePos::new(1, y),
            width: 1,
            height: 1,
        });
    }

    assert_eq!(priority_route_skip(&session, &data, 0), Some(JobKind::Wood));
    let messages = simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
    assert!(messages
        .iter()
        .any(|message| message == "Rattlebones skipped Wood: no route."));
}
#[test]
fn marked_only_work_policy_waits_without_marks_but_direct_orders_stay_open() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.route_policies.work = RoutePolicy::MarkedOnly;
    session.workforce.workers[0].assignment = JobKind::Wood;
    session.workforce.workers[0].priority_mode = true;

    assert_eq!(
        destination_for_worker(&session, &session.workforce.workers[0]),
        None
    );
    session.workforce.workers[0].priority_mode = false;
    assert!(destination_for_worker(&session, &session.workforce.workers[0]).is_some());
}
