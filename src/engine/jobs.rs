//! Worker assignment and fixed-timestep job progression.

use crate::data::GameData;
use crate::engine::{corpses, districts, movement, navigation, progression, suspicion};
use crate::state::{
    Building, BuildingKind, GameSession, JobKind, PlotStatus, ResourceKind, Worker, WorkerStatus,
    WorldState, ZoneKind,
};
use macroquad_toolkit::grid::TilePos;

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

pub fn destination_for_worker(session: &GameSession, worker: &Worker) -> Option<TilePos> {
    match worker.assignment {
        JobKind::Dig => worker
            .target_plot
            .and_then(|plot_id| session.world.plots.get(plot_id))
            .map(|plot| plot.position)
            .or_else(|| {
                session
                    .world
                    .plots
                    .iter()
                    .find(|plot| plot.status == PlotStatus::Ready)
                    .map(|plot| plot.position)
            }),
        JobKind::Haul => {
            if worker.carrying > 0 {
                Some(storage_destination(session, worker.position))
            } else if session.economy.loose_bones > 0 {
                session
                    .economy
                    .loose_bones_source
                    .or_else(|| {
                        let dug = session
                            .world
                            .plots
                            .iter()
                            .filter(|plot| plot.status == PlotStatus::Dug)
                            .map(|plot| plot.position)
                            .collect::<Vec<_>>();
                        nearest_reachable_or_nearest(session, worker.position, dug)
                    })
                    .or(Some(WorldState::stockpile_position()))
            } else if session.economy.loose_wood > 0 {
                session.economy.loose_wood_source.or_else(|| {
                    nearest_reachable_or_nearest(
                        session,
                        worker.position,
                        session.world.forest_tiles.clone(),
                    )
                })
            } else {
                None
            }
        }
        JobKind::Guard => Some(patrol_destination(session, worker.position)),
        JobKind::Wood => wood_destination(session, worker.position),
        JobKind::Build => session
            .world
            .buildings
            .iter()
            .find(|building| !building.complete)
            .map(Building::work_position),
        JobKind::Refine => session
            .world
            .buildings
            .iter()
            .find(|building| building.kind == BuildingKind::OssuaryKiln && building.complete)
            .map(Building::work_position),
    }
}

fn nearest_reachable_or_nearest(
    session: &GameSession,
    origin: TilePos,
    candidates: Vec<TilePos>,
) -> Option<TilePos> {
    navigation::nearest_reachable(session, origin, candidates.clone())
        .or_else(|| nearest_tile(origin, candidates))
}

fn nearest_tile(origin: TilePos, candidates: Vec<TilePos>) -> Option<TilePos> {
    candidates.into_iter().min_by_key(|tile| {
        (
            (origin.x - tile.x).abs() + (origin.y - tile.y).abs(),
            tile.y,
            tile.x,
        )
    })
}

fn storage_destination(session: &GameSession, origin: TilePos) -> TilePos {
    let fallback = session.world.storage_position();
    let candidates = session
        .world
        .zones
        .iter()
        .find(|zone| zone.kind == ZoneKind::Storage)
        .map_or_else(|| vec![fallback], |zone| zone.tiles.clone());
    nearest_reachable_or_nearest(session, origin, candidates).unwrap_or(fallback)
}

fn patrol_destination(session: &GameSession, origin: TilePos) -> TilePos {
    let fallback = session.world.patrol_position();
    let candidates = session
        .world
        .zones
        .iter()
        .find(|zone| zone.kind == ZoneKind::Patrol)
        .map_or_else(|| vec![fallback], |zone| zone.tiles.clone());
    nearest_reachable_or_nearest(session, origin, candidates)
        .unwrap_or_else(|| session.world.patrol_position_for(origin))
}

fn wood_destination(session: &GameSession, origin: TilePos) -> Option<TilePos> {
    let marked = session
        .world
        .forest_tiles
        .iter()
        .filter(|tile| session.world.zone_contains(ZoneKind::Work, **tile))
        .copied()
        .collect::<Vec<_>>();
    let candidates = if marked.is_empty() {
        session.world.forest_tiles.clone()
    } else {
        marked
    };
    nearest_reachable_or_nearest(session, origin, candidates)
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
    for index in 0..worker_count {
        let automatic = session.workforce.workers[index].priority_mode;
        let previous_job = session.workforce.workers[index].assignment;
        let job = if automatic {
            choose_priority(session, data)
        } else {
            previous_job
        };
        if automatic && job != previous_job {
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
                if move_worker_to(session, index, patrol) == WorkerMoveResult::Arrived {
                    let worker = &mut session.workforce.workers[index];
                    worker.status = WorkerStatus::Hiding;
                    worker.progress = 0.0;
                }
            }
            JobKind::Dig => simulate_dig(
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
        suspicion::adjust_quiet(
            session,
            -(mitigation
                * guards as f32
                * districts::guard_mitigation_multiplier(session, &data.config.district_rules)
                * dt),
            "guards keep the road quiet",
        );
    }
    messages
}

fn choose_priority(session: &GameSession, data: &GameData) -> JobKind {
    let mut available_jobs = Vec::new();
    for priority in &session.workforce.priorities {
        let available = match priority {
            JobKind::Guard => {
                let threshold =
                    if session.stewardship_policy == crate::state::StewardshipPolicy::Secure {
                        data.config.suspicion_thresholds[0]
                    } else {
                        data.config.suspicion_thresholds[1]
                    };
                session.pressure.suspicion >= threshold
            }
            JobKind::Haul => {
                session.economy.loose_bones > 0
                    || session.economy.loose_wood > 0
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
        };
        if available {
            available_jobs.push(*priority);
        }
    }
    available_jobs
        .into_iter()
        .min_by_key(|job| session.stewardship_policy.bias(*job))
        .unwrap_or(JobKind::Guard)
}

fn simulate_dig(
    session: &mut GameSession,
    data: &GameData,
    index: usize,
    dt: f32,
    speed: f32,
    lantern_bonus: f32,
    messages: &mut Vec<String>,
) {
    let job = data.jobs.get("dig").expect("validated dig job");
    let conspicuousness = data
        .undead
        .get(session.workforce.workers[index].kind.id())
        .expect("validated undead type")
        .conspicuousness;
    if session.workforce.workers[index].target_plot.is_none() {
        let worker_position = session.workforce.workers[index].position;
        let claimed: Vec<usize> = session
            .workforce
            .workers
            .iter()
            .filter_map(|worker| worker.target_plot)
            .collect();
        let selected = session.world.selected_plot;
        let target_id = select_reachable_dig_target(session, worker_position, &claimed, selected);
        if let Some(target_id) = target_id {
            if let Some(plot) = session.world.plots.get_mut(target_id) {
                plot.status = PlotStatus::Digging;
            }
            session.workforce.workers[index].target_plot = Some(target_id);
        }
    }
    let Some(plot_id) = session.workforce.workers[index].target_plot else {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        return;
    };
    let Some(plot_position) = session.world.plots.get(plot_id).map(|plot| plot.position) else {
        return;
    };
    match move_worker_to(session, index, plot_position) {
        WorkerMoveResult::Arrived => {}
        WorkerMoveResult::Walking => return,
        WorkerMoveResult::Blocked => {
            release_dig_target(session, index, plot_id);
            return;
        }
    }
    let district_speed =
        districts::work_speed_multiplier(session, &data.config.district_rules, plot_position);
    let worker = &mut session.workforce.workers[index];
    worker.status = WorkerStatus::Working;
    worker.progress += dt * speed * job.base_speed * district_speed;
    if worker.progress < job.work_seconds {
        return;
    }
    worker.progress = 0.0;
    worker.target_plot = None;
    if let Some(plot) = session.world.plots.get_mut(plot_id) {
        plot.status = PlotStatus::Dug;
        plot.progress = job.work_seconds;
    }
    session.economy.loose_bones += job.output_amount;
    if session.economy.loose_bones_source.is_none() {
        session.economy.loose_bones_source = Some(plot_position);
    }
    suspicion::adjust(
        session,
        job.suspicion_per_cycle * lantern_bonus * conspicuousness,
        "a grave was disturbed",
    );
    messages.push(format!(
        "{} bones are loose by plot {}.",
        job.output_amount,
        plot_id + 1
    ));
    if session.rng.chance(data.config.corpse_discovery_chance) {
        corpses::discover(session, data);
        suspicion::adjust(session, 1.0, "a corpse went missing from its grave");
    }
}

fn select_reachable_dig_target(
    session: &GameSession,
    origin: TilePos,
    claimed: &[usize],
    selected: Option<usize>,
) -> Option<usize> {
    if let Some(plot_id) = selected {
        let selected_candidate = session.world.plots.iter().find(|plot| {
            plot.id == plot_id && plot.status == PlotStatus::Ready && !claimed.contains(&plot.id)
        });
        if selected_candidate
            .and_then(|plot| navigation::plan_route(session, origin, plot.position).ok())
            .is_some()
        {
            return Some(plot_id);
        }
    }
    let designated = session
        .world
        .plots
        .iter()
        .filter(|plot| {
            plot.status == PlotStatus::Ready
                && !claimed.contains(&plot.id)
                && session.world.zone_contains(ZoneKind::Work, plot.position)
        })
        .map(|plot| (plot.id, plot.position))
        .collect::<Vec<_>>();
    reachable_plot_id(session, origin, &designated).or_else(|| {
        let available = session
            .world
            .plots
            .iter()
            .filter(|plot| plot.status == PlotStatus::Ready && !claimed.contains(&plot.id))
            .map(|plot| (plot.id, plot.position))
            .collect::<Vec<_>>();
        reachable_plot_id(session, origin, &available)
    })
}

fn reachable_plot_id(
    session: &GameSession,
    origin: TilePos,
    candidates: &[(usize, TilePos)],
) -> Option<usize> {
    let position = navigation::nearest_reachable(
        session,
        origin,
        candidates.iter().map(|(_, position)| *position),
    )?;
    candidates
        .iter()
        .find(|(_, candidate)| *candidate == position)
        .map(|(plot_id, _)| *plot_id)
}

fn release_dig_target(session: &mut GameSession, index: usize, plot_id: usize) {
    if let Some(plot) = session.world.plots.get_mut(plot_id) {
        if plot.status == PlotStatus::Digging {
            plot.status = PlotStatus::Ready;
            plot.progress = 0.0;
        }
    }
    let worker = &mut session.workforce.workers[index];
    worker.target_plot = None;
    worker.progress = 0.0;
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
    let capacity = data
        .undead
        .get(session.workforce.workers[index].kind.id())
        .expect("validated undead type")
        .haul_capacity
        + districts::haul_capacity_bonus(session, &data.config.district_rules);
    if session.workforce.workers[index].carrying <= 0 {
        let (resource, source) = if session.economy.loose_bones > 0 {
            (
                ResourceKind::Bones,
                destination_for_worker(session, &session.workforce.workers[index]),
            )
        } else if session.economy.loose_wood > 0 {
            (
                ResourceKind::Wood,
                destination_for_worker(session, &session.workforce.workers[index]),
            )
        } else {
            session.workforce.workers[index].status = WorkerStatus::Idle;
            session.workforce.workers[index].progress = 0.0;
            session.workforce.workers[index].carrying_resource = None;
            return;
        };
        let Some(source) = source else {
            session.workforce.workers[index].status = WorkerStatus::Idle;
            return;
        };
        if move_worker_to(session, index, source) != WorkerMoveResult::Arrived {
            return;
        }
        let amount = match resource {
            ResourceKind::Bones => session.economy.loose_bones.min(capacity),
            ResourceKind::Wood => session.economy.loose_wood.min(capacity),
        };
        if amount <= 0 {
            return;
        }
        match resource {
            ResourceKind::Bones => {
                session.economy.loose_bones -= amount;
                if session.economy.loose_bones == 0 {
                    session.economy.loose_bones_source = None;
                }
            }
            ResourceKind::Wood => {
                session.economy.loose_wood -= amount;
                if session.economy.loose_wood == 0 {
                    session.economy.loose_wood_source = None;
                }
            }
        }
        let worker = &mut session.workforce.workers[index];
        worker.carrying = amount;
        worker.carrying_resource = Some(resource);
        worker.status = WorkerStatus::Carrying;
        worker.progress = 0.0;
        return;
    }
    let storage_position = destination_for_worker(session, &session.workforce.workers[index])
        .unwrap_or_else(WorldState::stockpile_position);
    if move_worker_to(session, index, storage_position) != WorkerMoveResult::Arrived {
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
    worker.carrying = 0;
    worker.carrying_resource = None;
    worker.status = WorkerStatus::Idle;
    match resource {
        ResourceKind::Bones => {
            session.economy.bones += amount;
            messages.push(format!("Hauled {amount} bones into the stockpile."));
        }
        ResourceKind::Wood => {
            session.economy.wood += amount;
            messages.push(format!("Hauled {amount} wood into the stockpile."));
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
    let worker = &mut session.workforce.workers[index];
    worker.status = WorkerStatus::Working;
    worker.progress += dt * speed * job.base_speed * district_speed;
    if worker.progress >= job.work_seconds {
        worker.progress = 0.0;
        session.economy.loose_wood += job.output_amount;
        session.economy.loose_wood_source = Some(work_position);
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
