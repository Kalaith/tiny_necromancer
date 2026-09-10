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
            priority_mode: false,
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
