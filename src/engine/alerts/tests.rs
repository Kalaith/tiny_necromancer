use super::*;
use crate::state::{Building, BuildingKind, GameSession, ProductionOrder};
use macroquad_toolkit::grid::TilePos;

#[test]
fn loaded_production_without_a_refiner_becomes_actionable() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.progress.production = Some(ProductionOrder {
        building: BuildingKind::OssuaryKiln,
        progress: 1.0,
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
fn assigned_refiner_clears_the_kiln_alert() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.progress.production = Some(ProductionOrder {
        building: BuildingKind::OssuaryKiln,
        progress: 1.0,
    });
    session.workforce.workers[0].assignment = JobKind::Refine;
    assert!(!collect(&session, &data)
        .iter()
        .any(|alert| alert.title == "Kiln unattended"));
}

#[test]
fn loose_material_without_a_hauler_points_to_its_source() {
    let data = crate::data::GameData::load().unwrap();
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
    let data = crate::data::GameData::load().unwrap();
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
fn road_pressure_alert_waits_until_the_questioning_threshold() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.pressure.suspicion = data.config.suspicion_thresholds[1];
    session.pressure.stage = crate::data::SuspicionStage::Questioning;
    assert!(collect(&session, &data)
        .iter()
        .any(|alert| alert.severity == AlertSeverity::Critical));
}

#[test]
fn blocked_worker_route_points_to_the_worker() {
    let data = crate::data::GameData::load().unwrap();
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
fn route_alert_counts_additional_blocked_workers() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.workforce.workers[0].assignment = JobKind::Guard;
    session.workforce.workers[0].position = TilePos::new(2, 2);
    let mut second = session.workforce.workers[0].clone();
    second.id = 2;
    session.workforce.workers.push(second);
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
        .expect("blocked workers should share one route alert");
    assert!(alert.detail.contains("(+1 more blocked)"));
}

#[test]
fn patrol_coverage_alert_points_to_a_worker_for_the_next_post() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Patrol,
        tiles: vec![TilePos::new(5, 1), TilePos::new(5, 3)],
    });
    session.workforce.workers[0].assignment = JobKind::Guard;
    let mut second = session.workforce.workers[0].clone();
    second.id = 9;
    second.assignment = JobKind::Dig;
    session.workforce.workers.push(second);

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Patrol coverage")
        .expect("uncovered patrol post should be reported");

    assert!(alert.detail.contains("1 of 2 marked posts covered"));
    assert_eq!(alert.target, Some(Selection::Worker(1)));
}

#[test]
fn idle_work_district_points_to_a_marked_grave() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Work district idle")
        .expect("idle work district should be reported");

    assert_eq!(alert.severity, AlertSeverity::Info);
    assert_eq!(
        alert.detail,
        "0/1 marked Work tiles staffed · Marked Work has a ready grave · assign Dig."
    );
    assert_eq!(alert.target, Some(Selection::Grave(0)));
}

#[test]
fn idle_work_district_points_to_a_marked_forest_tile() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.workforce.workers[0].assignment = JobKind::Haul;
    for plot in &mut session.world.plots {
        plot.status = crate::state::PlotStatus::Locked;
    }
    let forest_tile = session.world.forest_tiles[2];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![forest_tile],
    });

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Work district idle")
        .expect("idle forest work district should be reported");

    assert_eq!(
        alert.detail,
        "0/1 marked Work tiles staffed · Marked Work reaches the forest edge · assign Wood."
    );
    assert_eq!(alert.target, Some(Selection::Ground(forest_tile)));
}

#[test]
fn assigned_dig_operator_clears_the_work_district_alert() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.workforce.workers[0].assignment = JobKind::Dig;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });

    assert!(!collect(&session, &data)
        .iter()
        .any(|alert| alert.title == "Work district idle"));
}

#[test]
fn understaffed_storage_district_points_to_a_marked_drop_point() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    let storage_tile = crate::state::WorldState::stockpile_position();
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![storage_tile],
    });
    session.economy.loose_bones = 8;

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Storage district idle")
        .expect("understaffed storage should be reported");

    assert_eq!(
        alert.detail,
        "0/1 marked Storage tiles staffed · loose material is waiting · assign Haul."
    );
    assert_eq!(alert.target, Some(Selection::Ground(storage_tile)));
}

#[test]
fn assigned_hauler_clears_the_storage_district_alert() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![crate::state::WorldState::stockpile_position()],
    });
    session.economy.loose_bones = 8;
    session.workforce.workers[0].assignment = JobKind::Haul;

    assert!(!collect(&session, &data)
        .iter()
        .any(|alert| alert.title == "Storage district idle"));
}

#[test]
fn storage_staffing_alert_replaces_generic_material_waiting() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![crate::state::WorldState::stockpile_position()],
    });
    session.economy.loose_bones = 8;

    let alerts = collect(&session, &data);

    assert!(alerts
        .iter()
        .any(|alert| alert.title == "Storage district idle"));
    assert!(!alerts
        .iter()
        .any(|alert| alert.title == "Materials waiting"));
}

#[test]
fn work_district_skips_an_unreachable_grave_for_the_next_reachable_target() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].position = TilePos::new(4, 2);
    session.world.plots[2].status = crate::state::PlotStatus::Ready;
    for position in [
        TilePos::new(1, 2),
        TilePos::new(2, 1),
        TilePos::new(3, 2),
        TilePos::new(2, 3),
    ] {
        session.world.buildings.push(Building {
            kind: BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position,
            width: 1,
            height: 1,
        });
    }
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![
            session.world.plots[0].position,
            session.world.plots[2].position,
        ],
    });

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Work district idle")
        .expect("reachable work should still be reported");

    assert_eq!(alert.target, Some(Selection::Grave(2)));
}

#[test]
fn dig_operator_does_not_hide_a_forest_only_work_advisory() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.workforce.workers[0].assignment = JobKind::Dig;
    for plot in &mut session.world.plots {
        plot.status = crate::state::PlotStatus::Locked;
    }
    let forest_tile = session.world.forest_tiles[2];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![forest_tile],
    });

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Work district idle")
        .expect("forest work should still need a Wood operator");

    assert_eq!(alert.target, Some(Selection::Ground(forest_tile)));
}

#[test]
fn wood_operator_does_not_hide_a_grave_only_work_advisory() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![crate::state::Technology::DomainStewardship];
    session.workforce.workers[0].assignment = JobKind::Wood;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });

    let alert = collect(&session, &data)
        .into_iter()
        .find(|alert| alert.title == "Work district idle")
        .expect("grave work should still need a Dig operator");

    assert_eq!(alert.target, Some(Selection::Grave(0)));
}
