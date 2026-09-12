//! Fixed-timestep worker simulation and resource delivery.

use super::*;

pub(super) fn simulate(session: &mut GameSession, data: &GameData, dt: f32) -> Vec<String> {
    let mut messages = Vec::new();
    let worker_count = session.workforce.workers.len();
    let shed_bonus =
        progression::building_speed_multiplier(session, data, crate::state::BuildingKind::WorkShed);
    let lantern_bonus = progression::building_suspicion_multiplier(
        session,
        data,
        crate::state::BuildingKind::GraveLantern,
    );
    let mut guards = 0;
    let mut guard_quieting = 0.0;
    for index in 0..worker_count {
        let automatic = session.workforce.workers[index].priority_mode;
        let previous_job = session.workforce.workers[index].assignment;
        let skipped_priority = automatic.then(|| priority_route_skip(session, data, index));
        let job = if automatic {
            choose_priority(session, data, index)
        } else {
            previous_job
        };
        if automatic && job != previous_job {
            if let Some(Some(skipped_priority)) = skipped_priority {
                messages.push(format!(
                    "{} skipped {}: no route.",
                    session.workforce.workers[index].name,
                    skipped_priority.label()
                ));
            }
            movement::drop_worker_cargo(session, index);
            if let Some(plot_id) = session.workforce.workers[index].target_plot {
                if let Some(plot) = session.world.plots.get_mut(plot_id) {
                    if plot.status == PlotStatus::Digging {
                        plot.status = PlotStatus::Ready;
                        plot.progress = 0.0;
                    }
                }
            }
            session.workforce.workers[index].target_plot = None;
            session.workforce.workers[index].progress = 0.0;
        }
        session.workforce.workers[index].assignment = job;
        let worker_speed = data
            .undead
            .get(session.workforce.workers[index].kind.id())
            .expect("validated undead type")
            .work_speed;
        match job {
            JobKind::Guard => {
                guards += 1;
                let patrol = destination_for_worker(session, &session.workforce.workers[index])
                    .unwrap_or_else(|| WorldState::guard_position(session.world.road_x));
                let arrived = move_worker_to(session, index, patrol) == WorkerMoveResult::Arrived;
                let position = session.workforce.workers[index].position;
                guard_quieting += districts::guard_mitigation_multiplier(
                    session,
                    &data.config.district_rules,
                    position,
                );
                if arrived {
                    let worker = &mut session.workforce.workers[index];
                    worker.status = WorkerStatus::Hiding;
                    worker.progress = 0.0;
                }
            }
            JobKind::Dig => dig::simulate(
                session,
                data,
                index,
                dt,
                worker_speed * shed_bonus,
                lantern_bonus,
                &mut messages,
            ),
            JobKind::Haul => simulate_haul(
                session,
                data,
                index,
                dt,
                worker_speed * shed_bonus,
                &mut messages,
            ),
            JobKind::Wood => simulate_wood(
                session,
                data,
                index,
                dt,
                worker_speed * shed_bonus,
                &mut messages,
            ),
            JobKind::Build => simulate_build(session, data, index, dt, worker_speed, &mut messages),
            JobKind::Refine => simulate_refine(session, data, index, dt, &mut messages),
        }
    }
    apply_guard_effect(session, data, dt, guards, guard_quieting);
    messages
}

fn apply_guard_effect(
    session: &mut GameSession,
    data: &GameData,
    dt: f32,
    guards: usize,
    guard_quieting: f32,
) {
    if guards == 0 {
        return;
    }
    let mitigation = data
        .jobs
        .get("guard")
        .expect("validated guard job")
        .guard_mitigation_per_second;
    let district_bonus = (guard_quieting - guards as f32).max(0.0);
    let suspicion_before = session.pressure.suspicion;
    suspicion::adjust_quiet(
        session,
        -(mitigation * guard_quieting * dt),
        "guards keep the road quiet",
    );
    let baseline_quieting = mitigation * guards as f32 * dt;
    let district_quieting = mitigation * district_bonus * dt;
    let room_after_baseline = (suspicion_before - baseline_quieting).max(0.0);
    districts::record_patrol_quieting(session, room_after_baseline.min(district_quieting));
}

fn simulate_build(
    session: &mut GameSession,
    data: &GameData,
    index: usize,
    dt: f32,
    worker_speed: f32,
    messages: &mut Vec<String>,
) {
    let construction_target = session
        .world
        .buildings
        .iter()
        .find(|building| !building.complete)
        .map(|building| building.work_position());
    if let Some(target) = construction_target {
        if move_worker_to(session, index, target) == WorkerMoveResult::Arrived {
            session.workforce.workers[index].status = WorkerStatus::Working;
            if let Some(message) =
                progression::advance_construction(session, data, dt * worker_speed)
            {
                messages.push(message);
                session.workforce.workers[index].progress = 0.0;
            }
        }
    } else {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        session.workforce.workers[index].progress = 0.0;
    }
}

fn simulate_haul(
    session: &mut GameSession,
    data: &GameData,
    index: usize,
    dt: f32,
    speed: f32,
    messages: &mut Vec<String>,
) {
    if session.workforce.workers[index].carrying <= 0 {
        load_haul(session, data, index);
        return;
    }
    let job = data.jobs.get("haul").expect("validated haul job");
    let Some((destination, replanned)) =
        logistics::destination_for_cargo(session, index, &data.config.district_rules)
    else {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        return;
    };
    if replanned {
        let worker = &session.workforce.workers[index];
        let resource = match worker.carrying_resource.unwrap_or(ResourceKind::Bones) {
            ResourceKind::Bones => "bones",
            ResourceKind::Wood => "wood",
        };
        let destination_detail = worker.haul_plan.map_or_else(
            || "current Storage".to_owned(),
            |plan| match plan.destination_kind {
                HaulDestination::Storage => format!("{} Storage", plan.storage_policy.label()),
                HaulDestination::Kiln => "the Ossuary Kiln".to_owned(),
            },
        );
        messages.push(format!(
            "{} replanned its {} bundle for {destination_detail}.",
            worker.name, resource
        ));
    }
    if move_worker_to(session, index, destination) != WorkerMoveResult::Arrived {
        return;
    }
    let worker = &mut session.workforce.workers[index];
    worker.status = WorkerStatus::Carrying;
    worker.progress += dt * speed * job.base_speed;
    if worker.progress < job.work_seconds {
        return;
    }
    worker.progress = 0.0;
    let resource = worker.carrying_resource.unwrap_or(ResourceKind::Bones);
    let amount = worker.carrying;
    let destination_kind = worker
        .haul_plan
        .map(|plan| plan.destination_kind)
        .unwrap_or(HaulDestination::Storage);
    worker.carrying = 0;
    worker.carrying_resource = None;
    worker.haul_plan = None;
    worker.status = WorkerStatus::Idle;
    if destination_kind == HaulDestination::Kiln {
        let delivered = progression::deliver_production_input(session, resource, amount);
        if delivered > 0 {
            let label = match resource {
                ResourceKind::Bones => "bones",
                ResourceKind::Wood => "wood",
            };
            messages.push(format!(
                "Delivered {delivered} {label} to the Ossuary Kiln."
            ));
        }
    } else {
        let stored = session.economy.store_resource(
            resource,
            amount,
            districts::storage_capacity(session, &data.config.district_rules),
        );
        if stored > 0 {
            let label = match resource {
                ResourceKind::Bones => "bones",
                ResourceKind::Wood => "wood",
            };
            messages.push(format!("Hauled {stored} {label} into the stockpile."));
        }
        if stored < amount {
            let worker = &mut session.workforce.workers[index];
            worker.carrying = amount - stored;
            worker.carrying_resource = Some(resource);
            worker.status = WorkerStatus::Carrying;
            messages.push(format!(
                "Storage is full; {} {} remain carried.",
                amount - stored,
                match resource {
                    ResourceKind::Bones => "bones",
                    ResourceKind::Wood => "wood",
                }
            ));
        }
    }
}

fn load_haul(session: &mut GameSession, data: &GameData, index: usize) {
    let Some(plan) = logistics::plan_haul(session, index) else {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        session.workforce.workers[index].progress = 0.0;
        session.workforce.workers[index].carrying_resource = None;
        session.workforce.workers[index].haul_plan = None;
        return;
    };
    session.workforce.workers[index].haul_plan = Some(plan);
    if move_worker_to(session, index, plan.source) != WorkerMoveResult::Arrived {
        return;
    }
    let base_capacity = data
        .undead
        .get(session.workforce.workers[index].kind.id())
        .expect("validated undead type")
        .haul_capacity;
    let storage_bonus = if plan.destination_kind == HaulDestination::Storage {
        districts::haul_capacity_bonus(session, &data.config.district_rules, plan.destination)
    } else {
        0
    };
    let storage_space = if plan.destination_kind == HaulDestination::Storage {
        districts::storage_space(session, &data.config.district_rules)
    } else {
        i32::MAX
    };
    let source_amount = match plan.destination_kind {
        HaulDestination::Storage => session.economy.loose_amount_at(plan.resource, plan.source),
        HaulDestination::Kiln => match plan.resource {
            ResourceKind::Bones if plan.source == session.world.stockpile_position() => {
                session.economy.bones
            }
            ResourceKind::Wood if plan.source == session.world.stockpile_position() => {
                session.economy.wood
            }
            _ => session.economy.loose_amount_at(plan.resource, plan.source),
        },
    };
    let production_need = if plan.destination_kind == HaulDestination::Kiln {
        progression::production_input_need(session, plan.resource)
    } else {
        i32::MAX
    };
    let amount = source_amount
        .min(base_capacity + storage_bonus)
        .min(production_need)
        .min(storage_space);
    if amount <= 0 {
        session.workforce.workers[index].haul_plan = None;
        session.workforce.workers[index].status = WorkerStatus::Idle;
        session.workforce.workers[index].progress = 0.0;
        return;
    }
    districts::record_storage_bonus(session, (amount - base_capacity).max(0));
    match plan.destination_kind {
        HaulDestination::Storage => {
            session
                .economy
                .take_loose(plan.resource, plan.source, amount);
        }
        HaulDestination::Kiln => match plan.resource {
            ResourceKind::Bones if plan.source == session.world.stockpile_position() => {
                session.economy.bones -= amount
            }
            ResourceKind::Wood if plan.source == session.world.stockpile_position() => {
                session.economy.wood -= amount
            }
            _ => {
                session
                    .economy
                    .take_loose(plan.resource, plan.source, amount);
            }
        },
    }
    let worker = &mut session.workforce.workers[index];
    worker.carrying = amount;
    worker.carrying_resource = Some(plan.resource);
    worker.haul_plan = Some(plan);
    worker.status = WorkerStatus::Carrying;
    worker.progress = 0.0;
}

fn simulate_wood(
    session: &mut GameSession,
    data: &GameData,
    index: usize,
    dt: f32,
    speed: f32,
    messages: &mut Vec<String>,
) {
    let job = data.jobs.get("wood").expect("validated wood job");
    let Some(work_position) = destination_for_worker(session, &session.workforce.workers[index])
    else {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        session.workforce.workers[index].progress = 0.0;
        return;
    };
    if move_worker_to(session, index, work_position) != WorkerMoveResult::Arrived {
        return;
    }
    let district_speed =
        districts::work_speed_multiplier(session, &data.config.district_rules, work_position);
    let completed = {
        let worker = &mut session.workforce.workers[index];
        worker.status = WorkerStatus::Working;
        worker.progress += dt * speed * job.base_speed * district_speed;
        if worker.progress >= job.work_seconds {
            worker.progress = 0.0;
            true
        } else {
            false
        }
    };
    if completed {
        session
            .economy
            .add_loose(ResourceKind::Wood, work_position, job.output_amount);
        if district_speed > 1.0 {
            districts::record_work_cycle(session);
        }
        suspicion::adjust(
            session,
            job.suspicion_per_cycle,
            "axes work the forest edge",
        );
        messages.push(format!("Gathered {} loose wood.", job.output_amount));
    }
}

fn simulate_refine(
    session: &mut GameSession,
    data: &GameData,
    index: usize,
    dt: f32,
    messages: &mut Vec<String>,
) {
    let Some(building) = session
        .world
        .buildings
        .iter()
        .find(|building| building.kind == BuildingKind::OssuaryKiln && building.complete)
    else {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        session.workforce.workers[index].progress = 0.0;
        return;
    };
    if session.progress.production.is_none()
        || progression::production_input_need(session, ResourceKind::Bones) > 0
        || progression::production_input_need(session, ResourceKind::Wood) > 0
    {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        session.workforce.workers[index].progress = 0.0;
        return;
    }
    let building_position = building.work_position();
    if move_worker_to(session, index, building_position) != WorkerMoveResult::Arrived {
        return;
    }
    session.workforce.workers[index].status = WorkerStatus::Working;
    if let Some(message) = progression::advance_production(session, data, dt) {
        messages.push(message);
        session.workforce.workers[index].progress = 0.0;
    } else {
        session.workforce.workers[index].progress = session
            .progress
            .production
            .as_ref()
            .map_or(0.0, |order| order.progress);
    }
}
