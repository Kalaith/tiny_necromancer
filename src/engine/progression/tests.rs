use super::*;
use crate::state::ProductionRecipeKind;
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
fn hush_ash_can_be_selected_and_quiets_suspicion_on_completion() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.wood = 100;
    session.pressure.suspicion = 20.0;
    session.research.completed = vec![Technology::Gravecraft, Technology::OssuaryLogistics];
    queue_building(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    advance_construction(&mut session, &data, 14.0);

    select_production_recipe(
        &mut session,
        &data,
        BuildingKind::OssuaryKiln,
        ProductionRecipeKind::HushAsh,
    )
    .unwrap();
    start_production(&mut session, &data, BuildingKind::OssuaryKiln).unwrap();
    advance_production(&mut session, &data, 11.0);

    assert_eq!(session.economy.ward_charges, 1);
    assert_eq!(session.pressure.suspicion, 19.0);
    assert_eq!(session.progress.production_ledger.total_cycles, 1);
    assert_eq!(session.progress.production_ledger.hush_ash_cycles, 1);
    assert_eq!(session.progress.production_ledger.suspicion_quieted, 5.0);
    assert!(session.pressure.feed[0].message.contains("Hush Ash"));
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
