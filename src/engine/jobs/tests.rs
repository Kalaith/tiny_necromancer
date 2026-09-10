use super::*;

fn simulate_for_seconds(session: &mut GameSession, data: &crate::data::GameData, seconds: f32) {
    let ticks = (seconds / data.config.tick_seconds).ceil() as usize;
    for _ in 0..ticks {
        simulate(session, data, data.config.tick_seconds);
    }
}

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
    session.workforce.workers[0].position = crate::state::WorldState::stockpile_position();
    simulate_for_seconds(&mut session, &data, 4.5);
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
    session.workforce.workers[0].position = crate::state::WorldState::stockpile_position();
    toggle_automation(&mut session).unwrap();
    simulate_for_seconds(&mut session, &data, 4.5);
    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
    assert_eq!(session.economy.loose_bones, 0);
    assert_eq!(session.economy.bones, data.config.starting_bones + 8);
}

#[test]
fn secure_policy_deploys_automated_workers_at_rumour() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.stewardship_policy = crate::state::StewardshipPolicy::Secure;
    session.pressure.suspicion = data.config.suspicion_thresholds[0] + 0.1;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Dig;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Guard);
}

#[test]
fn harvest_policy_prefers_material_work_over_guarding() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.stewardship_policy = crate::state::StewardshipPolicy::Harvest;
    session.pressure.suspicion = data.config.suspicion_thresholds[1] + 0.1;
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
}

#[test]
fn binding_routines_reorders_shared_priorities() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::BindingRoutines];
    assert_eq!(session.workforce.priorities[0], JobKind::Guard);
    move_priority(&mut session, JobKind::Dig, -1).unwrap();
    assert_eq!(session.workforce.priorities[1], JobKind::Dig);
    assert_eq!(session.workforce.priorities[2], JobKind::Haul);
    move_priority(&mut session, JobKind::Dig, -1).unwrap();
    assert_eq!(session.workforce.priorities[0], JobKind::Dig);
    assert_eq!(session.workforce.priorities[1], JobKind::Guard);
    assert!(move_priority(&mut session, JobKind::Dig, -1).is_err());
}

#[test]
fn automated_worker_can_choose_refine_from_the_priority_list() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![
        crate::state::Technology::Gravecraft,
        crate::state::Technology::OssuaryLogistics,
        crate::state::Technology::BindingRoutines,
    ];
    session.world.buildings.push(crate::state::Building {
        kind: crate::state::BuildingKind::OssuaryKiln,
        progress: 14.0,
        complete: true,
        position: macroquad_toolkit::grid::TilePos::new(6, 4),
        width: 2,
        height: 1,
    });
    session.economy.bones = 100;
    session.economy.wood = 100;
    crate::engine::progression::start_production(
        &mut session,
        &data,
        crate::state::BuildingKind::OssuaryKiln,
    )
    .unwrap();
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].position = macroquad_toolkit::grid::TilePos::new(5, 4);
    session.workforce.priorities = vec![JobKind::Refine, JobKind::Guard];
    simulate(&mut session, &data, 8.0);
    assert_eq!(session.workforce.workers[0].assignment, JobKind::Refine);
    assert_eq!(session.economy.ward_charges, 1);
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

#[test]
fn refine_worker_completes_a_loaded_kiln_cycle() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    session.research.completed = vec![
        crate::state::Technology::Gravecraft,
        crate::state::Technology::OssuaryLogistics,
    ];
    crate::engine::progression::queue_building(
        &mut session,
        &data,
        crate::state::BuildingKind::OssuaryKiln,
    )
    .unwrap();
    crate::engine::progression::advance_construction(&mut session, &data, 14.0);
    crate::engine::progression::start_production(
        &mut session,
        &data,
        crate::state::BuildingKind::OssuaryKiln,
    )
    .unwrap();
    session.workforce.workers[0].assignment = JobKind::Refine;
    session.workforce.workers[0].position = macroquad_toolkit::grid::TilePos::new(5, 4);
    simulate(&mut session, &data, 8.0);
    assert_eq!(session.economy.ward_charges, 1);
    assert!(session.progress.production.is_none());
}

#[test]
fn refine_order_requires_a_completed_kiln() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    assert!(assign_job(&mut session, JobKind::Refine).is_err());
}

#[test]
fn work_zone_guides_an_automatic_dig_target() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.progress.unlocked_plots = 2;
    session.world.plots[1].status = PlotStatus::Ready;
    session.world.selected_plot = None;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[1].position],
    });
    simulate(&mut session, &data, 0.0);
    assert_eq!(session.workforce.workers[0].target_plot, Some(1));
}

#[test]
fn patrol_zone_moves_guards_to_its_anchor() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let patrol = macroquad_toolkit::grid::TilePos::new(5, 1);
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Patrol,
        tiles: vec![patrol],
    });
    session.workforce.workers[0].assignment = JobKind::Guard;
    for _ in 0..8 {
        simulate(&mut session, &data, data.config.tick_seconds);
    }
    assert_eq!(session.workforce.workers[0].position, patrol);
    assert_eq!(session.workforce.workers[0].status, WorkerStatus::Hiding);
}

#[test]
fn domain_work_rule_accelerates_digging_on_marked_tiles() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let plot_position = session.world.plots[0].position;
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![plot_position],
    });
    session.workforce.workers[0].position = plot_position;
    session.workforce.workers[0].assignment = JobKind::Dig;

    simulate(&mut session, &data, 1.0);

    assert!(
        (session.workforce.workers[0].progress - districts::WORK_SPEED_MULTIPLIER).abs() < 0.001
    );
}

#[test]
fn domain_storage_rule_increases_haul_bundle() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let stockpile = crate::state::WorldState::stockpile_position();
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![stockpile],
    });
    session.economy.loose_bones = 20;
    session.workforce.workers[0].position = stockpile;
    session.workforce.workers[0].assignment = JobKind::Haul;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].carrying, 12);
    assert_eq!(session.economy.loose_bones, 8);
}

#[test]
fn domain_patrol_rule_strengthens_guard_mitigation() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let patrol = crate::state::WorldState::guard_position(session.world.road_x);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Patrol,
        tiles: vec![patrol],
    });
    session.pressure.suspicion = 20.0;
    session.workforce.workers[0].position = patrol;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 1.0);

    assert!((session.pressure.suspicion - 18.5).abs() < 0.001);
}

#[test]
fn hauler_walks_to_storage_before_transfer() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.loose_bones = 8;
    session.workforce.workers[0].assignment = JobKind::Haul;
    simulate(&mut session, &data, data.config.tick_seconds);
    assert_eq!(session.workforce.workers[0].status, WorkerStatus::Walking);
    assert_eq!(session.economy.loose_bones, 8);
    for _ in 0..20 {
        simulate(&mut session, &data, data.config.tick_seconds);
    }
    assert_eq!(session.economy.loose_bones, 0);
}
