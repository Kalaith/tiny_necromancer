use super::super::simulate;
use crate::state::{GameSession, JobKind, StewardshipPolicy, Technology, WorldState};

#[test]
fn harvest_policy_prefers_material_work_over_guarding() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.pressure.suspicion = data.config.suspicion_thresholds[1] + 0.1;
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
}

#[test]
fn harvest_policy_fills_a_marked_work_gap_before_unmarked_haul() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
}

#[test]
fn harvest_policy_fills_a_marked_storage_gap_before_unmarked_digging() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Storage,
        tiles: vec![WorldState::stockpile_position()],
    });
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
}

#[test]
fn harvest_policy_spreads_automated_workers_across_marked_gaps() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.extend([
        crate::state::Zone {
            kind: crate::state::ZoneKind::Work,
            tiles: vec![session.world.plots[0].position],
        },
        crate::state::Zone {
            kind: crate::state::ZoneKind::Storage,
            tiles: vec![WorldState::stockpile_position()],
        },
    ]);
    session.economy.loose_bones = 8;
    let mut second_worker = session.workforce.workers[0].clone();
    second_worker.id = session.workforce.next_worker_id;
    second_worker.name = "Second Hand".to_owned();
    session.workforce.next_worker_id += 1;
    session.workforce.workers.push(second_worker);
    for worker in &mut session.workforce.workers {
        worker.priority_mode = true;
        worker.assignment = JobKind::Guard;
    }

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
    assert_eq!(session.workforce.workers[1].assignment, JobKind::Dig);
}

#[test]
fn harvest_policy_fills_a_marked_forest_gap_with_wood() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.forest_tiles[0]],
    });
    for plot in &mut session.world.plots {
        plot.status = crate::state::PlotStatus::Locked;
    }
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Wood);
}

#[test]
fn harvest_policy_respects_a_direct_work_operator() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });
    session.economy.loose_bones = 8;
    let mut automatic_worker = session.workforce.workers[0].clone();
    automatic_worker.id = session.workforce.next_worker_id;
    automatic_worker.name = "Second Hand".to_owned();
    automatic_worker.priority_mode = true;
    automatic_worker.assignment = JobKind::Guard;
    session.workforce.next_worker_id += 1;
    session.workforce.workers.push(automatic_worker);
    session.workforce.workers[0].assignment = JobKind::Dig;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
    assert_eq!(session.workforce.workers[1].assignment, JobKind::Haul);
}

#[test]
fn harvest_policy_fills_a_remaining_work_slot() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.research.completed = vec![Technology::DomainStewardship];
    session.stewardship_policy = StewardshipPolicy::Harvest;
    let first_plot = session.world.plots[0].position;
    let second_plot = session.world.plots[1].position;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![first_plot, second_plot],
    });
    session.world.plots[1].status = crate::state::PlotStatus::Ready;
    session.economy.loose_bones = 8;
    let mut automatic_worker = session.workforce.workers[0].clone();
    automatic_worker.id = session.workforce.next_worker_id;
    automatic_worker.name = "Second Hand".to_owned();
    automatic_worker.priority_mode = true;
    automatic_worker.assignment = JobKind::Guard;
    session.workforce.next_worker_id += 1;
    session.workforce.workers.push(automatic_worker);
    session.workforce.workers[0].assignment = JobKind::Dig;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Dig);
    assert_eq!(session.workforce.workers[1].assignment, JobKind::Dig);
}

#[test]
fn harvest_policy_keeps_pre_domain_priorities_unchanged() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.stewardship_policy = StewardshipPolicy::Harvest;
    session.world.zones.push(crate::state::Zone {
        kind: crate::state::ZoneKind::Work,
        tiles: vec![session.world.plots[0].position],
    });
    session.economy.loose_bones = 8;
    session.workforce.workers[0].priority_mode = true;
    session.workforce.workers[0].assignment = JobKind::Guard;

    simulate(&mut session, &data, 0.0);

    assert_eq!(session.workforce.workers[0].assignment, JobKind::Haul);
}
