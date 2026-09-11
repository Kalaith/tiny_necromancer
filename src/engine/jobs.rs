//! Worker assignment and fixed-timestep job progression.

use crate::data::GameData;
use crate::engine::{districts, movement, navigation, progression, suspicion};
use crate::state::{
    BuildingKind, GameSession, HaulDestination, JobKind, PlotStatus, ResourceKind, RoutePolicy,
    WorkerStatus, WorldState, ZoneKind,
};
use macroquad_toolkit::grid::TilePos;

mod dig;
mod logistics;
mod targets;

pub use targets::{destination_for_worker, patrol_coverage, patrol_post_number};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatrolCoverage {
    pub total_posts: usize,
    pub covered_posts: usize,
    pub guard_count: usize,
}

pub fn assign_job(session: &mut GameSession, job: JobKind) -> Result<(), String> {
    if job == JobKind::Refine && !session.has_building(BuildingKind::OssuaryKiln) {
        return Err("Complete an Ossuary Kiln before assigning Refine Wards.".to_owned());
    }
    let index = session.workforce.selected_worker;
    if session.workforce.workers.get(index).is_none() {
        return Err("No worker is selected.".to_owned());
    }
    if session.workforce.workers[index].assignment == job {
        return Ok(());
    }
    movement::drop_worker_cargo(session, index);
    let plot_id = session.workforce.workers[index].target_plot.take();
    if let Some(plot_id) = plot_id {
        if let Some(plot) = session.world.plots.get_mut(plot_id) {
            if plot.status == PlotStatus::Digging {
                plot.status = PlotStatus::Ready;
                plot.progress = 0.0;
            }
        }
    }
    let worker_name = session.workforce.workers[index].name.clone();
    let worker = &mut session.workforce.workers[index];
    worker.assignment = job;
    worker.progress = 0.0;
    worker.status = WorkerStatus::Idle;
    worker.carrying = 0;
    session.add_feed(format!("{} assigned to {}.", worker_name, job.label()));
    Ok(())
}

pub fn select_worker(session: &mut GameSession, index: usize) {
    if index < session.workforce.workers.len() {
        session.workforce.selected_worker = index;
    }
}

pub fn toggle_automation(session: &mut GameSession) -> Result<(), String> {
    let index = session.workforce.selected_worker;
    if session.workforce.workers.get(index).is_none() {
        return Err("No worker is selected.".to_owned());
    }
    movement::drop_worker_cargo(session, index);
    let plot_id = session.workforce.workers[index].target_plot;
    if let Some(plot_id) = plot_id {
        if let Some(plot) = session.world.plots.get_mut(plot_id) {
            if plot.status == PlotStatus::Digging {
                plot.status = PlotStatus::Ready;
                plot.progress = 0.0;
            }
        }
    }
    let (worker_name, priority_mode) = {
        let worker = &mut session.workforce.workers[index];
        worker.priority_mode = !worker.priority_mode;
        worker.progress = 0.0;
        worker.target_plot = None;
        (worker.name.clone(), worker.priority_mode)
    };
    session.add_feed(format!(
        "{} is now {}.",
        worker_name,
        if priority_mode {
            "following priorities"
        } else {
            "under direct orders"
        }
    ));
    Ok(())
}

pub fn move_priority(
    session: &mut GameSession,
    job: JobKind,
    direction: i32,
) -> Result<(), String> {
    if !session
        .research
        .is_unlocked(crate::state::Technology::BindingRoutines)
    {
        return Err("Study Binding Routines before tuning worker priorities.".to_owned());
    }
    if direction != -1 && direction != 1 {
        return Err("Priority movement must be one step at a time.".to_owned());
    }
    session.workforce.normalize_priorities();
    let index = session
        .workforce
        .priorities
        .iter()
        .position(|priority| *priority == job)
        .ok_or_else(|| "That job is not in the priority list.".to_owned())?;
    let destination = if direction < 0 {
        index
            .checked_sub(1)
            .ok_or_else(|| format!("{} is already the highest priority.", job.label()))?
    } else {
        let destination = index + 1;
        if destination >= session.workforce.priorities.len() {
            return Err(format!("{} is already the lowest priority.", job.label()));
        }
        destination
    };
    session.workforce.priorities.swap(index, destination);
    session.add_feed(format!(
        "{} moved in the worker priority list.",
        job.label()
    ));
    Ok(())
}

pub fn simulate(session: &mut GameSession, data: &GameData, dt: f32) -> Vec<String> {
    let mut messages = Vec::new();
    let worker_count = session.workforce.workers.len();
    let shed_bonus = if session.has_building(crate::state::BuildingKind::WorkShed) {
        data.buildings
            .get(crate::state::BuildingKind::WorkShed.id())
            .expect("validated work shed")
            .speed_multiplier
    } else {
        1.0
    };
    let lantern_bonus = if session.has_building(crate::state::BuildingKind::GraveLantern) {
        data.buildings
            .get(crate::state::BuildingKind::GraveLantern.id())
            .expect("validated grave lantern")
            .suspicion_multiplier
    } else {
        1.0
    };
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
            JobKind::Build => {
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
            JobKind::Refine => simulate_refine(session, data, index, dt, &mut messages),
        }
    }
    if guards > 0 {
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
    messages
}

pub fn priority_route_skip(
    session: &GameSession,
    data: &GameData,
    worker_index: usize,
) -> Option<JobKind> {
    session.workforce.priorities.iter().copied().find(|job| {
        priority_available(session, data, *job)
            && !has_reachable_destination(session, worker_index, *job)
    })
}

pub fn marked_only_priority_wait(
    session: &GameSession,
    data: &GameData,
    worker_index: usize,
) -> Option<(JobKind, ZoneKind)> {
    if !session
        .research
        .is_unlocked(crate::state::Technology::DomainStewardship)
    {
        return None;
    }
    let worker = session.workforce.workers.get(worker_index)?;
    if !worker.priority_mode {
        return None;
    }
    let job = priority_route_skip(session, data, worker_index)?;
    let kind = match job {
        JobKind::Dig | JobKind::Wood => ZoneKind::Work,
        JobKind::Haul => ZoneKind::Storage,
        JobKind::Guard => ZoneKind::Patrol,
        JobKind::Build | JobKind::Refine => return None,
    };
    if session.world.route_policies.for_kind(kind) != RoutePolicy::MarkedOnly {
        return None;
    }
    let mut hypothetical = worker.clone();
    hypothetical.assignment = job;
    hypothetical.target_plot = None;
    destination_for_worker(session, &hypothetical)
        .is_none()
        .then_some((job, kind))
}

fn choose_priority(session: &GameSession, data: &GameData, worker_index: usize) -> JobKind {
    let mut available_jobs = Vec::new();
    for priority in &session.workforce.priorities {
        if priority_available(session, data, *priority)
            && has_reachable_destination(session, worker_index, *priority)
        {
            available_jobs.push(*priority);
        }
    }
    available_jobs
        .into_iter()
        .min_by_key(|job| districts::policy_bias(session, *job))
        .unwrap_or(session.workforce.workers[worker_index].assignment)
}

fn priority_available(session: &GameSession, data: &GameData, priority: JobKind) -> bool {
    let patrol_gap = {
        let coverage = patrol_coverage(session);
        coverage.total_posts > coverage.covered_posts
    };
    match priority {
        JobKind::Guard => {
            let threshold = if session.stewardship_policy == crate::state::StewardshipPolicy::Secure
            {
                data.config.suspicion_thresholds[0]
            } else {
                data.config.suspicion_thresholds[1]
            };
            session.pressure.suspicion >= threshold
                || (session.stewardship_policy == crate::state::StewardshipPolicy::Secure
                    && patrol_gap)
        }
        JobKind::Haul => {
            session.economy.loose_bones > 0
                || session.economy.loose_wood > 0
                || progression::production_input_need(session, ResourceKind::Bones) > 0
                    && session.economy.bones > 0
                || progression::production_input_need(session, ResourceKind::Wood) > 0
                    && session.economy.wood > 0
                || session
                    .workforce
                    .workers
                    .iter()
                    .any(|worker| worker.carrying > 0)
        }
        JobKind::Dig => session
            .world
            .plots
            .iter()
            .any(|plot| plot.status == PlotStatus::Ready),
        JobKind::Wood => {
            session.economy.wood < data.config.worker_wood_reserve
                || session.progress.unlocked_plots < data.config.victory_plots
        }
        JobKind::Build => session
            .world
            .buildings
            .iter()
            .any(|building| !building.complete),
        JobKind::Refine => session.progress.production.is_some(),
    }
}

fn has_reachable_destination(session: &GameSession, worker_index: usize, job: JobKind) -> bool {
    let Some(current_worker) = session.workforce.workers.get(worker_index) else {
        return false;
    };
    let mut worker = current_worker.clone();
    worker.assignment = job;
    worker.target_plot = None;
    let Some(destination) = destination_for_worker(session, &worker) else {
        return false;
    };
    navigation::plan_route(session, worker.position, destination).is_ok()
}

fn simulate_haul(
    session: &mut GameSession,
    data: &GameData,
    index: usize,
    dt: f32,
    speed: f32,
    messages: &mut Vec<String>,
) {
    let job = data.jobs.get("haul").expect("validated haul job");
    let base_capacity = data
        .undead
        .get(session.workforce.workers[index].kind.id())
        .expect("validated undead type")
        .haul_capacity;
    if session.workforce.workers[index].carrying <= 0 {
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
        let storage_bonus = if plan.destination_kind == HaulDestination::Storage {
            districts::haul_capacity_bonus(session, &data.config.district_rules, plan.destination)
        } else {
            0
        };
        let capacity = base_capacity + storage_bonus;
        let storage_space = if plan.destination_kind == HaulDestination::Storage {
            districts::storage_space(session, &data.config.district_rules)
        } else {
            i32::MAX
        };
        let source_amount = match plan.destination_kind {
            HaulDestination::Storage => session.economy.loose_amount_at(plan.resource, plan.source),
            HaulDestination::Kiln => match plan.resource {
                ResourceKind::Bones if plan.source == WorldState::stockpile_position() => {
                    session.economy.bones
                }
                ResourceKind::Wood if plan.source == WorldState::stockpile_position() => {
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
            .min(capacity)
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
                ResourceKind::Bones if plan.source == WorldState::stockpile_position() => {
                    session.economy.bones -= amount
                }
                ResourceKind::Wood if plan.source == WorldState::stockpile_position() => {
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
        return;
    }
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
                HaulDestination::Storage => {
                    format!("{} Storage", plan.storage_policy.label())
                }
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
    if session.progress.production.is_none() {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        session.workforce.workers[index].progress = 0.0;
        return;
    }
    if progression::production_input_need(session, ResourceKind::Bones) > 0
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerMoveResult {
    Arrived,
    Walking,
    Blocked,
}

fn move_worker_to(session: &mut GameSession, index: usize, target: TilePos) -> WorkerMoveResult {
    let current = session.workforce.workers[index].position;
    if current == target {
        return WorkerMoveResult::Arrived;
    }
    if let Some(next) = navigation::next_step(session, current, target) {
        let worker = &mut session.workforce.workers[index];
        worker.position = next;
        worker.status = if worker.carrying > 0 {
            WorkerStatus::Carrying
        } else {
            WorkerStatus::Walking
        };
        worker.progress = 0.0;
        return WorkerMoveResult::Walking;
    }
    let worker = &mut session.workforce.workers[index];
    worker.status = WorkerStatus::Idle;
    worker.progress = 0.0;
    WorkerMoveResult::Blocked
}

#[cfg(test)]
mod tests;
