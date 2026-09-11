use super::super::{destination_for_worker, simulate};
use crate::state::{GameSession, JobKind, RoutePolicy, Technology, WorldState};
use macroquad_toolkit::grid::TilePos;

#[test]
fn hauler_commits_the_source_and_marked_storage_drop_when_loading() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let source = session.world.plots[0].position;
    let drop = TilePos::new(5, 1);
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![drop],
    });
    session.economy.loose_bones = 8;
    session.economy.loose_bones_source = Some(source);
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].position = source;

    simulate(&mut session, &data, 0.0);

    let plan = session.workforce.workers[0]
        .haul_plan
        .expect("loading should commit a haul route");
    assert_eq!(plan.resource, crate::state::ResourceKind::Bones);
    assert_eq!(plan.source, source);
    assert_eq!(plan.destination, drop);
    assert_eq!(plan.storage_policy, RoutePolicy::MarkedFirst);
}

#[test]
fn committed_drop_stays_stable_when_another_marked_tile_is_added() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let first_drop = TilePos::new(5, 1);
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![first_drop],
    });
    session.economy.loose_bones = 8;
    session.economy.loose_bones_source = Some(session.world.plots[0].position);
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].position = session.world.plots[0].position;

    simulate(&mut session, &data, 0.0);
    let committed = session.workforce.workers[0]
        .haul_plan
        .expect("loading should commit a haul route")
        .destination;

    session.world.zones[0]
        .tiles
        .push(WorldState::stockpile_position());

    assert_eq!(
        destination_for_worker(&session, &session.workforce.workers[0]),
        Some(committed)
    );
}

#[test]
fn changed_storage_policy_replans_a_carried_bundle_without_dropping_it() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let marked_drop = TilePos::new(5, 1);
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![marked_drop],
    });
    session.economy.loose_bones = 8;
    session.economy.loose_bones_source = Some(session.world.plots[0].position);
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].position = session.world.plots[0].position;

    simulate(&mut session, &data, 0.0);
    assert_eq!(
        session.workforce.workers[0]
            .haul_plan
            .expect("loading should commit a haul route")
            .destination,
        marked_drop
    );

    session.world.route_policies.storage = RoutePolicy::Nearest;
    simulate(&mut session, &data, 0.0);

    let worker = &session.workforce.workers[0];
    let plan = worker
        .haul_plan
        .expect("replanning should keep the bundle planned");
    assert_eq!(worker.carrying, 8);
    assert_eq!(plan.destination, WorldState::stockpile_position());
    assert_eq!(plan.storage_policy, RoutePolicy::Nearest);
}

#[test]
fn storage_policy_replan_emits_one_delivery_notice() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![TilePos::new(5, 1)],
    });
    session.economy.loose_bones = 8;
    session.economy.loose_bones_source = Some(session.world.plots[0].position);
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].position = session.world.plots[0].position;

    simulate(&mut session, &data, 0.0);
    session.world.route_policies.storage = RoutePolicy::Nearest;
    let messages = simulate(&mut session, &data, 0.0);

    assert_eq!(
        messages,
        vec!["Rattlebones replanned its bones bundle for Nearest Storage.".to_owned()]
    );
    assert!(simulate(&mut session, &data, 0.0).is_empty());
}
