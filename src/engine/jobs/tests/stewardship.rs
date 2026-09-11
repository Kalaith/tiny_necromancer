use super::super::{destination_for_worker, simulate};
use crate::state::{
    Building, BuildingKind, GameSession, JobKind, RoutePolicy, StewardshipPolicy, Technology,
    WorldState,
};
use macroquad_toolkit::grid::TilePos;

#[test]
fn harvest_policy_prefers_material_work_over_guarding() {
    let data = crate::data::GameData::load().unwrap();
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
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
}

#[test]
fn harvest_policy_fills_a_marked_storage_gap_before_unmarked_digging() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![WorldState::stockpile_position()],
    });
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
}

#[test]
fn harvest_policy_spreads_automated_workers_across_marked_gaps() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.extend([
        crate::state::Zone {
            kind: crate::state::ZoneKind::Work,
            tiles: vec![session.world.plots[0].position],
        },
        crate::state::Zone {
            kind: crate::state::ZoneKind::Storage,
            tiles: vec![WorldState::stockpile_position()],
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
fn harvest_policy_fills_a_marked_forest_gap_with_wood() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.forest_tiles[0]],
    });
    for plot in &mut session.world.plots {
        plot.status = crate::state::PlotStatus::Locked;
    }
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Wood);
}

#[test]
fn harvest_policy_respects_a_direct_work_operator() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });
    session.economy.loose_bones = 8;
    let mut automatic_worker = session.workforce.workers[0].clone();
    automatic_worker.id = session.workforce.next_worker_id;
    automatic_worker.name = "Second Hand".to_owned();
    automatic_worker.priority_mode = true;
    automatic_worker.assignment = JobKind::Guard;
    session.workforce.next_worker_id += 1;
    session.workforce.workers.push(automatic_worker);
    session.workforce.workers[0].assignment = JobKind::Dig;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
    assert_eq!(session.workforce.workers[1].assignment, JobKind::Haul);
}

#[test]
fn harvest_policy_fills_a_remaining_work_slot() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    let first_plot = session.world.plots[0].position;
    let second_plot = session.world.plots[1].position;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![first_plot, second_plot],
    });
    session.world.plots[1].status = crate::state::PlotStatus::Ready;
    session.economy.loose_bones = 8;
    let mut automatic_worker = session.workforce.workers[0].clone();
    automatic_worker.id = session.workforce.next_worker_id;
    automatic_worker.name = "Second Hand".to_owned();
    automatic_worker.priority_mode = true;
    automatic_worker.assignment = JobKind::Guard;
    session.workforce.next_worker_id += 1;
    session.workforce.workers.push(automatic_worker);
    session.workforce.workers[0].assignment = JobKind::Dig;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
    assert_eq!(session.workforce.workers[1].assignment, JobKind::Dig);
}

#[test]
fn harvest_policy_keeps_pre_domain_priorities_unchanged() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
}

#[test]
fn automated_worker_falls_back_from_a_blocked_marked_forest() {
    let data = crate::data::GameData::load().unwrap();
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
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
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

    assert_eq!(
        super::super::priority_route_skip(&session, &data, 0),
        Some(JobKind::Wood)
    );
    let messages = simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
    assert!(messages
        .iter()
        .any(|message| message == "Rattlebones skipped Wood: no route."));
}

#[test]
fn automated_worker_keeps_its_duty_when_every_route_is_blocked() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.progress.unlocked_plots = data.config.victory_plots;
    for plot in &mut session.world.plots {
        plot.status = crate::state::PlotStatus::Locked;
    }
    session.economy.wood = data.config.worker_wood_reserve;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Wood;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.forest_tiles[0]],
    });
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

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Wood);
}

#[test]
fn automated_worker_keeps_priority_order_among_reachable_duties() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;
    session.workforce.priorities = vec![
        JobKind::Wood,
        JobKind::Haul,
        JobKind::Guard,
        JobKind::Dig,
        JobKind::Build,
        JobKind::Refine,
    ];
    session.economy.loose_bones = 8;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Wood);
}

#[test]
fn hypothetical_haul_route_uses_the_worker_roster_slot() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let first_storage = TilePos::new(5, 1);
    let second_storage = TilePos::new(6, 5);
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![first_storage, second_storage],
    });
    session.workforce.workers[0].assignment = JobKind::Wood;
    session.workforce.workers[0].carrying = 1;
    session.workforce.workers[0].position = first_storage;
    let mut hypothetical = session.workforce.workers[0].clone();
    hypothetical.assignment = JobKind::Haul;

    assert_eq!(
        destination_for_worker(&session, &hypothetical),
        Some(first_storage)
    );
}

#[test]
fn nearest_work_policy_ignores_marked_forest_preference_for_repeat_workers() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.forest_tiles[1]],
    });
    session.world.route_policies.work = RoutePolicy::Nearest;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Wood;
    session.workforce.workers[0].position = TilePos::new(1, 0);

    assert_eq!(
        destination_for_worker(&session, &session.workforce.workers[0]),
        Some(session.world.forest_tiles[0])
    );
}

#[test]
fn marked_only_work_policy_waits_without_marks_but_direct_orders_stay_open() {
    let data = crate::data::GameData::load().unwrap();
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

#[test]
fn nearest_storage_and_patrol_policies_use_normal_fallbacks() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    let marked_storage = TilePos::new(5, 1);
    let marked_patrol = TilePos::new(5, 3);
    session.world.zones.extend([
        crate::state::Zone {
            kind: crate::state::ZoneKind::Storage,
            tiles: vec![marked_storage],
        },
        crate::state::Zone {
            kind: crate::state::ZoneKind::Patrol,
            tiles: vec![marked_patrol],
        },
    ]);
    session.world.route_policies.storage = RoutePolicy::Nearest;
    session.world.route_policies.patrol = RoutePolicy::Nearest;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].position = WorldState::stockpile_position();
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].carrying = 1;
    assert_eq!(
        destination_for_worker(&session, &session.workforce.workers[0]),
        Some(WorldState::stockpile_position())
    );
    session.workforce.workers[0].assignment = JobKind::Guard;
    session.workforce.workers[0].carrying = 0;
    assert_eq!(
        destination_for_worker(&session, &session.workforce.workers[0]),
        Some(WorldState::guard_position(session.world.road_x))
    );
}
