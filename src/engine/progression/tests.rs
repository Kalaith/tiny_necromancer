use super::*;
use crate::state::{JobKind, UndeadKind, Worker, WorkerStatus};
use macroquad_toolkit::grid::TilePos;

#[test]
fn plot_expansion_spends_wood_and_increases_capacity() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.wood = 50;
    unlock_plot(&mut session, &data).unwrap();
    assert_eq!(session.progress.unlocked_plots, 2);
    assert_eq!(session.economy.wood, 42);
    assert_eq!(session.world.plots[1].status, PlotStatus::Ready);
}

#[test]
fn victory_needs_every_vertical_slice_milestone() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.progress.unlocked_plots = 6;
    session.world.buildings = vec![
        Building {
            kind: BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position: crate::state::default_building_position_for_kind(BuildingKind::WorkShed),
            width: 2,
            height: 2,
        },
        Building {
            kind: BuildingKind::GraveLantern,
            progress: 12.0,
            complete: true,
            position: crate::state::default_building_position_for_kind(BuildingKind::GraveLantern),
            width: 1,
            height: 1,
        },
    ];
    for id in 1..=5 {
        session.workforce.workers.push(Worker {
            id,
            name: id.to_string(),
            kind: if id == 5 {
                UndeadKind::BruteSkeleton
            } else {
                UndeadKind::Skeleton
            },
            assignment: JobKind::Guard,
            position: TilePos::new(1, 1),
            status: WorkerStatus::Idle,
            progress: 0.0,
            target_plot: None,
            carrying: 0,
            carrying_resource: None,
            priority_mode: false,
            haul_plan: None,
        });
    }
    check_victory(&mut session, &data);
    assert_eq!(session.phase, GamePhase::Victory);
}

#[test]
fn shed_construction_adds_the_improved_shovel() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    queue_building(&mut session, &data, BuildingKind::WorkShed).unwrap();
    let before = session.economy.shovels;
    assert!(advance_construction(&mut session, &data, 10.0).is_some());
    assert!(session.has_building(BuildingKind::WorkShed));
    assert_eq!(session.economy.shovels, before + 1);
}

#[test]
fn work_shed_upgrade_spends_materials_and_improves_throughput() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    queue_building(&mut session, &data, BuildingKind::WorkShed).unwrap();
    advance_construction(&mut session, &data, 10.0);

    upgrade_building(&mut session, &data, BuildingKind::WorkShed).unwrap();

    assert_eq!(building_level(&session, BuildingKind::WorkShed), 2);
    assert_eq!(session.economy.bones, 82);
    assert_eq!(session.economy.wood, 58);
    assert!(
        (building_speed_multiplier(&session, &data, BuildingKind::WorkShed) - 1.38).abs() < 0.001
    );
}

#[test]
fn lantern_upgrade_makes_guarding_quieter() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.mana = 100;
    session.economy.wood = 100;
    queue_building(&mut session, &data, BuildingKind::GraveLantern).unwrap();
    advance_construction(&mut session, &data, 12.0);
    let before = building_suspicion_multiplier(&session, &data, BuildingKind::GraveLantern);

    upgrade_building(&mut session, &data, BuildingKind::GraveLantern).unwrap();

    assert!(building_suspicion_multiplier(&session, &data, BuildingKind::GraveLantern) < before);
}

#[test]
fn kiln_upgrade_finishes_a_cycle_faster() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    session.research.completed = vec![Technology::Gravecraft, Technology::OssuaryLogistics];
    queue_building(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    advance_construction(&mut session, &data, 14.0);
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    upgrade_building(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();

    advance_production(&mut session, &data, 6.5);

    assert_eq!(session.economy.ward_charges, 1);
    assert_eq!(building_level(&session, BuildingKind::OssuaryKiln), 2);
}

#[test]
fn kiln_is_gated_by_logistics_and_loads_its_recipe() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    assert!(queue_building(&mut session, &data, BuildingKind::OssuaryKiln).is_err());
    session.research.completed = vec![Technology::Gravecraft, Technology::OssuaryLogistics];
    queue_building(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    advance_construction(&mut session, &data, 14.0);
    assert!(session.has_building(BuildingKind::OssuaryKiln));
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    assert_eq!(session.economy.bones, 70);
    assert_eq!(session.economy.wood, 70);
    assert!(session.progress.production.is_some());
}

#[test]
fn ward_charge_can_be_spent_to_quiet_suspicion() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.ward_charges = 2;
    session.pressure.suspicion = 20.0;
    use_ward_charge(&mut session).unwrap();
    assert_eq!(session.economy.ward_charges, 1);
    assert_eq!(session.pressure.suspicion, 12.0);
}

#[test]
fn buildings_cannot_cover_graves_or_the_forest_edge() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    assert!(queue_building_at(
        &mut session,
        &data,
        BuildingKind::WorkShed,
        TilePos::new(2, 2),
    )
    .is_err());
    assert!(queue_building_at(
        &mut session,
        &data,
        BuildingKind::WorkShed,
        TilePos::new(0, 0),
    )
    .is_err());
}

#[test]
fn kiln_queue_rolls_into_the_next_reserved_cycle() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    session.research.completed = vec![Technology::Gravecraft, Technology::OssuaryLogistics];
    queue_building(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    advance_construction(&mut session, &data, 14.0);
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    assert_eq!(session.progress.production_queue, 1);
    advance_production(&mut session, &data, 8.0);
    assert_eq!(session.economy.ward_charges, 1);
    assert!(session.progress.production.is_some());
    advance_production(&mut session, &data, 8.0);
    assert_eq!(session.economy.ward_charges, 2);
    assert!(session.progress.production.is_none());
}

#[test]
fn cancelling_a_reserved_cycle_refunds_materials_but_keeps_working_cycle() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    session.research.completed = vec![Technology::Gravecraft, Technology::OssuaryLogistics];
    queue_building(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    advance_construction(&mut session, &data, 14.0);
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    session.progress.production.as_mut().unwrap().progress = 2.0;
    let bones_after_two_loads = session.economy.bones;

    cancel_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();

    assert_eq!(session.progress.production_queue, 0);
    assert_eq!(session.economy.bones, bones_after_two_loads + 12);
    assert_eq!(session.economy.wood, 70);
    assert_eq!(session.progress.production.unwrap().progress, 2.0);
    assert!(session
        .pressure
        .feed
        .first()
        .is_some_and(|entry| entry.message.contains("+12 bones and +6 wood")));
}

#[test]
fn cancelling_without_a_reserved_cycle_keeps_storage_untouched() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 42;
    session.economy.wood = 60;
    session.research.completed = vec![Technology::Gravecraft, Technology::OssuaryLogistics];
    queue_building(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    advance_construction(&mut session, &data, 14.0);
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    let storage_before = (session.economy.bones, session.economy.wood);

    let error = cancel_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap_err();

    assert_eq!(error, "There is no reserved ward cycle to cancel.");
    assert_eq!(
        (session.economy.bones, session.economy.wood),
        storage_before
    );
}

#[test]
fn full_kiln_queue_does_not_spend_more_materials() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    session.research.completed = vec![Technology::Gravecraft, Technology::OssuaryLogistics];
    queue_building(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    advance_construction(&mut session, &data, 14.0);
    for _ in 0..=MAX_PRODUCTION_QUEUE {
        start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    }
    let bones = session.economy.bones;
    let wood = session.economy.wood;
    assert!(start_production(&mut session, &data, BuildingKind::OssuaryKiln).is_err());
    assert_eq!(session.economy.bones, bones);
    assert_eq!(session.economy.wood, wood);
}
