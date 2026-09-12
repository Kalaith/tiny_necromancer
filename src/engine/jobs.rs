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
mod simulation;
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
    let message = if job == JobKind::Wood {
        format!(
            "{} assigned to Gather Wood. Reassign it to Haul when timber is loose.",
            worker_name
        )
    } else {
        format!("{} assigned to {}.", worker_name, job.label())
    };
    session.add_feed(message);
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
    simulation::simulate(session, data, dt)
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
