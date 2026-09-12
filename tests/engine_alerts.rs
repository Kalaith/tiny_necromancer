use macroquad_toolkit::grid::TilePos;
use tiny_necromancer::engine::alerts::*;
use tiny_necromancer::state::*;
use tiny_necromancer::state::{
    Building, BuildingKind, GameSession, ProductionOrder, ProductionRecipeKind,
};
#[test]
fn loaded_production_without_a_refiner_becomes_actionable() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.progress.production = Some(ProductionOrder {
        building: BuildingKind::OssuaryKiln,
        progress: 1.0,
        recipe: ProductionRecipeKind::WardCharge,
        bones_remaining: 0,
        wood_remaining: 0,
    });
    session.world.buildings.push(Building {
        kind: BuildingKind::OssuaryKiln,
        progress: 14.0,
        complete: true,
        position: TilePos::new(6, 4),
        width: 2,
        height: 1,
    });
    let alerts = collect(&session, &data);
    let alert = alerts
        .iter()
        .find(|alert| alert.title == "Kiln unattended")
        .expect("production blocker should be reported");
    assert_eq!(alert.target, Some(Selection::Building(0)));
}
#[test]
fn loose_material_without_a_hauler_points_to_its_source() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let source = TilePos::new(4, 3);
    session.economy.loose_bones = 8;
    session.economy.loose_bones_source = Some(source);
    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Materials waiting")
        .expect("loose material should be reported");
    assert_eq!(alert.target, Some(Selection::Ground(source)));
}
#[test]
fn construction_without_a_builder_points_to_the_scaffold() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.world.buildings.push(Building {
        kind: BuildingKind::WorkShed,
        progress: 2.0,
        complete: false,
        position: TilePos::new(6, 2),
        width: 2,
        height: 2,
    });
    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Construction stalled")
        .expect("unfinished structure should be reported");
    assert_eq!(alert.target, Some(Selection::Building(0)));
}
#[test]
fn blocked_worker_route_points_to_the_worker() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.workforce.workers[0].assignment = JobKind::Guard;
    session.workforce.workers[0].position = TilePos::new(2, 2);
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

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Route blocked")
        .expect("sealed worker route should be reported");
    assert_eq!(alert.target, Some(Selection::Worker(0)));
    assert!(alert.detail.contains("obstructions seal the way"));
}
#[test]
fn marked_only_priority_waiting_becomes_an_actionable_alert() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![tiny_necromancer::state::Technology::DomainStewardship];
    session.world.route_policies.work = tiny_necromancer::state::RoutePolicy::MarkedOnly;
    session.economy.wood = 0;
    session.workforce.workers[0].assignment = tiny_necromancer::state::JobKind::Wood;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.priorities = vec![
        tiny_necromancer::state::JobKind::Wood,
        tiny_necromancer::state::JobKind::Haul,
        tiny_necromancer::state::JobKind::Guard,
        tiny_necromancer::state::JobKind::Dig,
        tiny_necromancer::state::JobKind::Build,
        tiny_necromancer::state::JobKind::Refine,
    ];

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Route policy waiting")
        .expect("a strict route policy should explain its missing mark");

    assert_eq!(
        alert.target,
        Some(tiny_necromancer::state::Selection::Worker(0))
    );
    assert!(alert.detail.contains("marked Work route for Wood"));
}
