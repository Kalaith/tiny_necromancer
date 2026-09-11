use super::super::simulate;
use crate::engine::progression::{production_input_need, start_production};
use crate::state::{
    Building, BuildingKind, GameSession, HaulDestination, JobKind, Technology, WorkerStatus,
    WorldState,
};

fn session_with_empty_kiln(data: &crate::data::GameData) -> GameSession {
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::Gravecraft, Technology::OssuaryLogistics];
    let position = crate::state::default_building_position_for_kind(BuildingKind::OssuaryKiln);
    session.world.buildings.push(Building {
        kind: BuildingKind::OssuaryKiln,
        progress: 14.0,
        complete: true,
        position,
        width: 2,
        height: 1,
    });
    session.economy.bones = 0;
    session.economy.wood = 0;
    session
}

#[test]
fn kiln_supply_haul_delivers_stockpiled_inputs_before_refining() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = session_with_empty_kiln(&data);
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    assert_eq!(
        production_input_need(&session, crate::state::ResourceKind::Bones),
        12
    );
    assert_eq!(
        production_input_need(&session, crate::state::ResourceKind::Wood),
        6
    );

    session.economy.bones = 12;
    session.economy.wood = 6;
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].position = WorldState::stockpile_position();

    for _ in 0..100 {
        simulate(&mut session, &data, 1.0);
        if production_input_need(&session, crate::state::ResourceKind::Bones) == 0
            && production_input_need(&session, crate::state::ResourceKind::Wood) == 0
        {
            break;
        }
    }

    assert_eq!(
        production_input_need(&session, crate::state::ResourceKind::Bones),
        0
    );
    assert_eq!(
        production_input_need(&session, crate::state::ResourceKind::Wood),
        0
    );
    assert_eq!(session.economy.bones, 0);
    assert_eq!(session.economy.wood, 0);
    assert_eq!(session.economy.ward_charges, 0);
}

#[test]
fn kiln_supply_plan_names_the_kiln_as_its_destination() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = session_with_empty_kiln(&data);
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    session.economy.bones = 4;
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].position = WorldState::stockpile_position();

    simulate(&mut session, &data, 0.0);

    let plan = session.workforce.workers[0]
        .haul_plan
        .expect("missing kiln input should create a haul plan");
    assert_eq!(plan.destination_kind, HaulDestination::Kiln);
    assert_eq!(plan.source, WorldState::stockpile_position());
    assert_eq!(
        plan.destination,
        crate::engine::progression::production_destination(&session).unwrap()
    );
}

#[test]
fn refiner_waits_without_advancing_an_unloaded_cycle() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = session_with_empty_kiln(&data);
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    let work_position = crate::engine::progression::production_destination(&session).unwrap();
    session.workforce.workers[0].assignment = JobKind::Refine;
    session.workforce.workers[0].position = work_position;

    simulate(&mut session, &data, 1.0);

    assert_eq!(
        session
            .progress
            .production
            .as_ref()
            .expect("cycle should still be waiting")
            .progress,
        0.0
    );
    assert_eq!(session.workforce.workers[0].status, WorkerStatus::Idle);
}
