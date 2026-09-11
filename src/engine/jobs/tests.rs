use super::*;

mod logistics;
mod production_supply;
mod stewardship;

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
fn starting_worker_can_gather_wood_before_the_shed_exists() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.begin();

    assign_job(&mut session, JobKind::Wood).unwrap();
    simulate_for_seconds(&mut session, &data, 10.0);

    assert!(session.economy.loose_wood > 0);
    let shed_cost = data
        .buildings
        .get("work_shed")
        .expect("validated work shed")
        .wood_cost;
    assert!(session.economy.wood < shed_cost);
    assert_eq!(session.workforce.workers[0].assignment, JobKind::Wood);
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
        (session.workforce.workers[0].progress - data.config.district_rules.work_speed_multiplier)
            .abs()
            < 0.001
    );
}
