//! Grave selection and route-aware digging progression.

use super::WorkerMoveResult;
use crate::data::GameData;
use crate::engine::{corpses, districts, navigation, suspicion};
use crate::state::{GameSession, PlotStatus, WorkerStatus, ZoneKind};
use macroquad_toolkit::grid::TilePos;

pub(super) fn simulate(
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
        let target_id = select_reachable_target(session, worker_position, &claimed, selected);
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
    match super::move_worker_to(session, index, plot_position) {
        WorkerMoveResult::Arrived => {}
        WorkerMoveResult::Walking => return,
        WorkerMoveResult::Blocked => {
            release_target(session, index, plot_id);
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
    if district_speed > 1.0 {
        districts::record_work_cycle(session);
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

fn select_reachable_target(
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

fn release_target(session: &mut GameSession, index: usize, plot_id: usize) {
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
