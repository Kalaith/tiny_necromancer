//! Route-aware destinations for workers and marked district coverage.

use super::PatrolCoverage;
use crate::engine::navigation;
use crate::state::{
    Building, BuildingKind, GameSession, JobKind, PlotStatus, RoutePolicy, Technology, Worker,
    WorldState, ZoneKind,
};
use macroquad_toolkit::grid::TilePos;

pub fn destination_for_worker(session: &GameSession, worker: &Worker) -> Option<TilePos> {
    match worker.assignment {
        JobKind::Dig => dig_destination(session, worker),
        JobKind::Haul => {
            if worker.carrying > 0 {
                worker
                    .haul_plan
                    .map(|plan| plan.destination)
                    .or_else(|| storage_destination(session, worker.position, worker.id))
            } else if let Some(plan) = worker.haul_plan {
                let available = match plan.resource {
                    crate::state::ResourceKind::Bones => session.economy.loose_bones,
                    crate::state::ResourceKind::Wood => session.economy.loose_wood,
                };
                (available > 0).then_some(plan.source)
            } else if session.economy.loose_bones > 0 {
                let dug = session
                    .world
                    .plots
                    .iter()
                    .filter(|plot| plot.status == PlotStatus::Dug)
                    .map(|plot| plot.position)
                    .collect::<Vec<_>>();
                preferred_source_or_reachable_alternative(
                    session,
                    worker.position,
                    session.economy.loose_bones_source,
                    dug,
                )
                .or(Some(WorldState::stockpile_position()))
            } else if session.economy.loose_wood > 0 {
                preferred_source_or_reachable_alternative(
                    session,
                    worker.position,
                    session.economy.loose_wood_source,
                    session.world.forest_tiles.clone(),
                )
            } else {
                None
            }
        }
        JobKind::Guard => patrol_destination(session, worker.position, worker.id),
        JobKind::Wood => wood_destination(session, worker.position, worker.id),
        JobKind::Build => structure_destination(
            session,
            worker.position,
            session
                .world
                .buildings
                .iter()
                .filter(|building| !building.complete)
                .map(Building::work_position)
                .collect(),
        ),
        JobKind::Refine => structure_destination(
            session,
            worker.position,
            session
                .world
                .buildings
                .iter()
                .filter(|building| building.kind == BuildingKind::OssuaryKiln && building.complete)
                .map(Building::work_position)
                .collect(),
        ),
    }
}

pub fn patrol_coverage(session: &GameSession) -> PatrolCoverage {
    let posts = unique_tiles(zone_tiles(session, ZoneKind::Patrol));
    let mut covered = Vec::new();
    let mut guard_count = 0;
    for worker in &session.workforce.workers {
        if worker.assignment != JobKind::Guard {
            continue;
        }
        guard_count += 1;
        let Some(destination) = destination_for_worker(session, worker) else {
            continue;
        };
        if !posts.contains(&destination)
            || navigation::plan_route(session, worker.position, destination).is_err()
            || covered.contains(&destination)
        {
            continue;
        }
        covered.push(destination);
    }
    PatrolCoverage {
        total_posts: posts.len(),
        covered_posts: covered.len(),
        guard_count,
    }
}

pub fn patrol_post_number(session: &GameSession, worker: &Worker) -> Option<usize> {
    if worker.assignment != JobKind::Guard {
        return None;
    }
    let destination = destination_for_worker(session, worker)?;
    unique_tiles(zone_tiles(session, ZoneKind::Patrol))
        .iter()
        .position(|post| *post == destination)
        .map(|index| index + 1)
}

fn dig_destination(session: &GameSession, worker: &Worker) -> Option<TilePos> {
    if let Some(position) = worker
        .target_plot
        .and_then(|plot_id| session.world.plots.get(plot_id))
        .map(|plot| plot.position)
    {
        return Some(position);
    }
    let claimed = session
        .workforce
        .workers
        .iter()
        .filter_map(|other| other.target_plot)
        .collect::<Vec<_>>();
    let selected = session.world.selected_plot.and_then(|plot_id| {
        let plot = session.world.plots.iter().find(|plot| {
            plot.id == plot_id && plot.status == PlotStatus::Ready && !claimed.contains(&plot.id)
        })?;
        navigation::plan_route(session, worker.position, plot.position)
            .ok()
            .map(|_| plot.position)
    });
    let designated = session
        .world
        .plots
        .iter()
        .filter(|plot| {
            plot.status == PlotStatus::Ready
                && !claimed.contains(&plot.id)
                && session.world.zone_contains(ZoneKind::Work, plot.position)
        })
        .map(|plot| plot.position)
        .collect::<Vec<_>>();
    let available = session
        .world
        .plots
        .iter()
        .filter(|plot| plot.status == PlotStatus::Ready && !claimed.contains(&plot.id))
        .map(|plot| plot.position)
        .collect::<Vec<_>>();
    match route_policy(session, worker.id, ZoneKind::Work) {
        RoutePolicy::MarkedOnly => selected
            .filter(|position| session.world.zone_contains(ZoneKind::Work, *position))
            .or_else(|| navigation::nearest_reachable(session, worker.position, designated.clone()))
            .or_else(|| designated.first().copied()),
        RoutePolicy::Nearest => selected
            .or_else(|| navigation::nearest_reachable(session, worker.position, available.clone()))
            .or_else(|| available.first().copied()),
        RoutePolicy::MarkedFirst => selected
            .or_else(|| navigation::nearest_reachable(session, worker.position, designated))
            .or_else(|| navigation::nearest_reachable(session, worker.position, available.clone()))
            .or_else(|| {
                session
                    .world
                    .selected_plot
                    .and_then(|plot_id| {
                        session.world.plots.iter().find(|plot| {
                            plot.id == plot_id
                                && plot.status == PlotStatus::Ready
                                && !claimed.contains(&plot.id)
                        })
                    })
                    .map(|plot| plot.position)
            })
            .or_else(|| available.first().copied()),
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

fn preferred_source_or_reachable_alternative(
    session: &GameSession,
    origin: TilePos,
    preferred: Option<TilePos>,
    alternatives: Vec<TilePos>,
) -> Option<TilePos> {
    if let Some(preferred) = preferred {
        if navigation::plan_route(session, origin, preferred).is_ok() {
            return Some(preferred);
        }
    }
    nearest_reachable_or_nearest(session, origin, alternatives).or(preferred)
}

fn structure_destination(
    session: &GameSession,
    origin: TilePos,
    candidates: Vec<TilePos>,
) -> Option<TilePos> {
    nearest_reachable_or_nearest(session, origin, candidates)
}

fn storage_destination(session: &GameSession, origin: TilePos, worker_id: u32) -> Option<TilePos> {
    let policy = route_policy(session, worker_id, ZoneKind::Storage);
    let fallback = if policy == RoutePolicy::Nearest {
        WorldState::stockpile_position()
    } else {
        session.world.storage_position()
    };
    let marked = unique_tiles(zone_tiles(session, ZoneKind::Storage));
    let candidates = match policy {
        RoutePolicy::MarkedFirst => {
            if marked.is_empty() {
                vec![fallback]
            } else {
                marked
            }
        }
        RoutePolicy::Nearest => vec![fallback],
        RoutePolicy::MarkedOnly => {
            if marked.is_empty() {
                return None;
            }
            marked
        }
    };
    let current_slot = storage_slot(session, worker_id);
    let preferred = if current_slot == 0 {
        nearest_reachable_or_nearest(session, origin, candidates.clone())?
    } else {
        candidates[current_slot % candidates.len()]
    };
    let occupied = session
        .workforce
        .workers
        .iter()
        .filter(|worker| {
            worker.assignment == JobKind::Haul
                && worker.id != worker_id
                && (worker.carrying > 0 || worker.haul_plan.is_some())
                && storage_slot(session, worker.id) < current_slot
        })
        .filter_map(|worker| {
            worker
                .haul_plan
                .map(|plan| plan.destination)
                .or_else(|| storage_destination(session, worker.position, worker.id))
        })
        .collect::<Vec<_>>();
    if !occupied.contains(&preferred) && navigation::plan_route(session, origin, preferred).is_ok()
    {
        return Some(preferred);
    }
    let unoccupied = candidates
        .iter()
        .copied()
        .filter(|candidate| !occupied.contains(candidate))
        .collect::<Vec<_>>();
    nearest_reachable_or_nearest(session, origin, unoccupied)
        .or_else(|| nearest_reachable_or_nearest(session, origin, candidates.clone()))
        .or_else(|| (policy != RoutePolicy::MarkedOnly).then_some(fallback))
}

pub(super) fn storage_destination_for(
    session: &GameSession,
    origin: TilePos,
    worker_id: u32,
) -> Option<TilePos> {
    storage_destination(session, origin, worker_id)
}

pub(super) fn storage_route_policy(session: &GameSession, worker_id: u32) -> RoutePolicy {
    route_policy(session, worker_id, ZoneKind::Storage)
}

pub(super) fn storage_destination_is_current(
    session: &GameSession,
    worker_id: u32,
    destination: TilePos,
) -> bool {
    match storage_route_policy(session, worker_id) {
        RoutePolicy::Nearest => destination == WorldState::stockpile_position(),
        RoutePolicy::MarkedOnly => session.world.zone_contains(ZoneKind::Storage, destination),
        RoutePolicy::MarkedFirst => {
            let marked = unique_tiles(zone_tiles(session, ZoneKind::Storage));
            marked.is_empty() && destination == session.world.storage_position()
                || marked.contains(&destination)
                || destination == session.world.storage_position()
        }
    }
}

fn assignment_slot(session: &GameSession, job: JobKind, worker_id: u32) -> usize {
    roster_slot(session, job, worker_id)
}

fn storage_slot(session: &GameSession, worker_id: u32) -> usize {
    let Some(worker_index) = session
        .workforce
        .workers
        .iter()
        .position(|worker| worker.id == worker_id)
    else {
        return worker_id as usize;
    };
    session.workforce.workers[..=worker_index]
        .iter()
        .filter(|worker| {
            worker.id == worker_id
                || (worker.assignment == JobKind::Haul
                    && (worker.carrying > 0 || worker.haul_plan.is_some()))
        })
        .count()
        .saturating_sub(1)
}

fn patrol_destination(session: &GameSession, origin: TilePos, worker_id: u32) -> Option<TilePos> {
    let policy = route_policy(session, worker_id, ZoneKind::Patrol);
    let fallback = if policy == RoutePolicy::Nearest {
        WorldState::guard_position(session.world.road_x)
    } else {
        session.world.patrol_position()
    };
    let candidates = unique_tiles(zone_tiles(session, ZoneKind::Patrol));
    match policy {
        RoutePolicy::Nearest => return Some(fallback),
        RoutePolicy::MarkedOnly if candidates.is_empty() => return None,
        _ => {}
    }
    if candidates.is_empty() {
        return Some(fallback);
    }
    let current_slot = guard_slot(session, worker_id);
    let preferred = candidates[current_slot % candidates.len()];
    if navigation::plan_route(session, origin, preferred).is_ok() {
        return Some(preferred);
    }
    let occupied = session
        .workforce
        .workers
        .iter()
        .filter(|worker| {
            worker.assignment == JobKind::Guard
                && worker.id != worker_id
                && guard_slot(session, worker.id) < current_slot
        })
        .filter_map(|worker| patrol_destination(session, worker.position, worker.id))
        .collect::<Vec<_>>();
    let unoccupied = candidates
        .iter()
        .copied()
        .filter(|candidate| !occupied.contains(candidate))
        .collect::<Vec<_>>();
    nearest_reachable_or_nearest(session, origin, unoccupied)
        .or_else(|| nearest_reachable_or_nearest(session, origin, candidates.clone()))
        .or_else(|| {
            (policy != RoutePolicy::MarkedOnly).then(|| session.world.patrol_position_for(origin))
        })
}

fn guard_slot(session: &GameSession, worker_id: u32) -> usize {
    roster_slot(session, JobKind::Guard, worker_id)
}

fn roster_slot(session: &GameSession, job: JobKind, worker_id: u32) -> usize {
    let Some(worker_index) = session
        .workforce
        .workers
        .iter()
        .position(|worker| worker.id == worker_id)
    else {
        return worker_id as usize;
    };
    session.workforce.workers[..=worker_index]
        .iter()
        .filter(|worker| worker.id == worker_id || worker.assignment == job)
        .count()
        .saturating_sub(1)
}

fn zone_tiles(session: &GameSession, kind: ZoneKind) -> Vec<TilePos> {
    session
        .world
        .zones
        .iter()
        .filter(|zone| zone.kind == kind)
        .flat_map(|zone| zone.tiles.iter().copied())
        .collect()
}

fn unique_tiles(tiles: Vec<TilePos>) -> Vec<TilePos> {
    tiles.into_iter().fold(Vec::new(), |mut unique, tile| {
        if !unique.contains(&tile) {
            unique.push(tile);
        }
        unique
    })
}

fn wood_destination(session: &GameSession, origin: TilePos, worker_id: u32) -> Option<TilePos> {
    let marked = session
        .world
        .forest_tiles
        .iter()
        .filter(|tile| session.world.zone_contains(ZoneKind::Work, **tile))
        .copied()
        .collect::<Vec<_>>();
    let candidates = match route_policy(session, worker_id, ZoneKind::Work) {
        RoutePolicy::MarkedFirst => {
            if marked.is_empty() {
                session.world.forest_tiles.clone()
            } else {
                marked
            }
        }
        RoutePolicy::Nearest => session.world.forest_tiles.clone(),
        RoutePolicy::MarkedOnly => marked,
    };
    let candidates = unique_tiles(candidates);
    if candidates.is_empty() {
        return None;
    }
    let current_slot = assignment_slot(session, JobKind::Wood, worker_id);
    let preferred = if current_slot == 0 {
        nearest_reachable_or_nearest(session, origin, candidates.clone())?
    } else {
        candidates[current_slot % candidates.len()]
    };
    let occupied = session
        .workforce
        .workers
        .iter()
        .filter(|worker| {
            worker.assignment == JobKind::Wood
                && worker.id != worker_id
                && assignment_slot(session, JobKind::Wood, worker.id) < current_slot
        })
        .filter_map(|worker| wood_destination(session, worker.position, worker.id))
        .collect::<Vec<_>>();
    if !occupied.contains(&preferred) && navigation::plan_route(session, origin, preferred).is_ok()
    {
        return Some(preferred);
    }
    let unoccupied = candidates
        .iter()
        .copied()
        .filter(|candidate| !occupied.contains(candidate))
        .collect::<Vec<_>>();
    nearest_reachable_or_nearest(session, origin, unoccupied)
        .or_else(|| nearest_reachable_or_nearest(session, origin, candidates))
}

fn route_policy(session: &GameSession, worker_id: u32, kind: ZoneKind) -> RoutePolicy {
    let automated = session
        .workforce
        .workers
        .iter()
        .find(|worker| worker.id == worker_id)
        .is_some_and(|worker| worker.priority_mode);
    if automated && session.research.is_unlocked(Technology::DomainStewardship) {
        session.world.route_policies.for_kind(kind)
    } else {
        RoutePolicy::MarkedFirst
    }
}
