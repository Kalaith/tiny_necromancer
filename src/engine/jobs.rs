//! Worker assignment and fixed-timestep job progression.

use crate::data::GameData;
use crate::engine::{corpses, progression, suspicion};
use crate::state::{GameSession, JobKind, PlotStatus, WorkerStatus};

pub fn assign_job(session: &mut GameSession, job: JobKind) -> Result<(), String> {
    let index = session.workforce.selected_worker;
    if session.workforce.workers.get(index).is_none() {
        return Err("No worker is selected.".to_owned());
    }
    if session.workforce.workers[index].assignment == job {
        return Ok(());
    }
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
                let worker = &mut session.workforce.workers[index];
                worker.status = WorkerStatus::Hiding;
                worker.position = crate::state::WorldState::guard_position(session.world.road_x);
                worker.progress = 0.0;
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
                session.workforce.workers[index].status = WorkerStatus::Working;
                session.workforce.workers[index].position = session.world.mana_source;
                if let Some(message) =
                    progression::advance_construction(session, data, dt * worker_speed)
                {
                    messages.push(message);
                    session.workforce.workers[index].progress = 0.0;
                }
            }
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
            -(mitigation * guards as f32 * dt),
            "guards keep the road quiet",
        );
    }
    messages
}

fn choose_priority(session: &GameSession, data: &GameData) -> JobKind {
    for priority in &session.workforce.priorities {
        let available = match priority {
            JobKind::Guard => session.pressure.suspicion >= data.config.suspicion_thresholds[1],
            JobKind::Haul => session.economy.loose_bones > 0 || session.economy.loose_wood > 0,
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
        };
        if available {
            return *priority;
        }
    }
    JobKind::Guard
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
        let claimed: Vec<usize> = session
            .workforce
            .workers
            .iter()
            .filter_map(|worker| worker.target_plot)
            .collect();
        let selected = session.world.selected_plot;
        let selected_target = session
            .world
            .plots
            .iter()
            .find(|plot| {
                plot.status == PlotStatus::Ready
                    && !claimed.contains(&plot.id)
                    && selected == Some(plot.id)
            })
            .map(|plot| plot.id);
        let target_id = selected_target.or_else(|| {
            session
                .world
                .plots
                .iter()
                .find(|plot| plot.status == PlotStatus::Ready && !claimed.contains(&plot.id))
                .map(|plot| plot.id)
        });
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
    let worker = &mut session.workforce.workers[index];
    worker.status = WorkerStatus::Working;
    worker.position = plot_position;
    worker.progress += dt * speed * job.base_speed;
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

fn simulate_haul(
    session: &mut GameSession,
    data: &GameData,
    index: usize,
    dt: f32,
    speed: f32,
    messages: &mut Vec<String>,
) {
    let job = data.jobs.get("haul").expect("validated haul job");
    let has_material = session.economy.loose_bones > 0 || session.economy.loose_wood > 0;
    if !has_material {
        session.workforce.workers[index].status = WorkerStatus::Idle;
        return;
    }
    let worker = &mut session.workforce.workers[index];
    worker.status = WorkerStatus::Carrying;
    worker.position = session.world.storage_position();
    worker.progress += dt * speed * job.base_speed;
    if worker.progress < job.work_seconds {
        return;
    }
    worker.progress = 0.0;
    let capacity = data
        .undead
        .get(worker.kind.id())
        .expect("validated undead type")
        .haul_capacity;
    let bones = session.economy.loose_bones.min(capacity);
    if bones > 0 {
        session.economy.loose_bones -= bones;
        session.economy.bones += bones;
        messages.push(format!("Hauled {bones} bones into the stockpile."));
    } else {
        let wood = session.economy.loose_wood.min(capacity);
        session.economy.loose_wood -= wood;
        session.economy.wood += wood;
        messages.push(format!("Hauled {wood} wood into the stockpile."));
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
    let worker = &mut session.workforce.workers[index];
    worker.status = WorkerStatus::Working;
    worker.position = session.world.forest_tiles[0];
    worker.progress += dt * speed * job.base_speed;
    if worker.progress >= job.work_seconds {
        worker.progress = 0.0;
        session.economy.loose_wood += job.output_amount;
        suspicion::adjust(
            session,
            job.suspicion_per_cycle,
            "axes work the forest edge",
        );
        messages.push(format!("Gathered {} loose wood.", job.output_amount));
    }
}

#[cfg(test)]
mod tests;
