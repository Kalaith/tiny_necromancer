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

#[test]
fn hauler_commits_a_drop_before_reaching_the_source() {
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
    session.workforce.workers[0].position = WorldState::stockpile_position();

    simulate(&mut session, &data, 0.0);

    let worker = &session.workforce.workers[0];
    let plan = worker
        .haul_plan
        .expect("the source walk should carry a committed plan");
    assert_eq!(worker.carrying, 0);
    assert_eq!(plan.source, source);
    assert_eq!(plan.destination, drop);
    assert_eq!(destination_for_worker(&session, worker), Some(source));
}

#[test]
fn pending_haul_plans_reserve_distinct_marked_storage_drops() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let first_drop = TilePos::new(5, 1);
    let second_drop = TilePos::new(6, 5);
    let source = session.world.plots[0].position;
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![first_drop, second_drop],
    });
    session.economy.loose_bones = 16;
    session.economy.loose_bones_source = Some(source);
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].position = WorldState::stockpile_position();
    let mut second_worker = session.workforce.workers[0].clone();
    second_worker.id = session.workforce.next_worker_id;
    second_worker.position = source;
    session.workforce.next_worker_id += 1;
    session.workforce.workers.push(second_worker);

    simulate(&mut session, &data, 0.0);
    simulate(&mut session, &data, 0.0);

    assert_eq!(
        session.workforce.workers[0]
            .haul_plan
            .expect("first worker should reserve a drop")
            .destination,
        first_drop
    );
    assert_eq!(
        session.workforce.workers[1]
            .haul_plan
            .expect("second worker should reserve another drop")
            .destination,
        second_drop
    );
}

#[test]
fn removed_storage_drop_replans_a_walking_and_carried_bundle() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let first_drop = TilePos::new(5, 1);
    let second_drop = TilePos::new(6, 5);
    let source = session.world.plots[0].position;
    session.research.completed = vec![Technology::DomainStewardship];
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![first_drop, second_drop],
    });
    session.economy.loose_bones = 16;
    session.economy.loose_bones_source = Some(source);
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].position = WorldState::stockpile_position();

    simulate(&mut session, &data, 0.0);
    assert_eq!(
        session.workforce.workers[0]
            .haul_plan
            .expect("walking worker should have a plan")
            .destination,
        first_drop
    );
    session.world.zones[0]
        .tiles
        .retain(|tile| *tile != first_drop);
    simulate(&mut session, &data, 0.0);
    assert_eq!(
        session.workforce.workers[0]
            .haul_plan
            .expect("walking worker should replan")
            .destination,
        second_drop
    );

    session.workforce.workers[0].position = source;
    simulate(&mut session, &data, 0.0);
    assert!(session.workforce.workers[0].carrying > 0);
    session.world.zones[0].tiles.push(first_drop);
    session.world.zones[0]
        .tiles
        .retain(|tile| *tile != second_drop);
    simulate(&mut session, &data, 0.0);

    let worker = &session.workforce.workers[0];
    assert!(worker.carrying > 0);
    assert_eq!(
        worker
            .haul_plan
            .expect("carried worker should replan")
            .destination,
        first_drop
    );
}

#[test]
fn hauler_selects_the_nearest_of_multiple_resource_piles() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let near_source = session.world.plots[0].position;
    let far_source = session.world.forest_tiles[0];
    session
        .economy
        .add_loose(crate::state::ResourceKind::Bones, near_source, 8);
    session
        .economy
        .add_loose(crate::state::ResourceKind::Bones, far_source, 8);
    session.workforce.workers[0].assignment = JobKind::Haul;
    session.workforce.workers[0].position = near_source;

    simulate(&mut session, &data, 0.0);

    let worker = &session.workforce.workers[0];
    assert_eq!(
        worker
            .haul_plan
            .expect("haul should choose a source")
            .source,
        near_source
    );
    assert_eq!(
        session
            .economy
            .loose_amount_at(crate::state::ResourceKind::Bones, near_source),
        0
    );
    assert_eq!(
        session
            .economy
            .loose_amount_at(crate::state::ResourceKind::Bones, far_source),
        8
    );
}
