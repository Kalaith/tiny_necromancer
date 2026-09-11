//! Source-to-drop planning for carried resources.

use super::{destination_for_worker, targets};
use crate::engine::navigation;
use crate::state::{GameSession, HaulPlan, ResourceKind};
use macroquad_toolkit::grid::TilePos;

pub(super) fn plan_haul(session: &GameSession, worker_index: usize) -> Option<HaulPlan> {
    let worker = session.workforce.workers.get(worker_index)?;
    let resource = if session.economy.loose_bones > 0 {
        ResourceKind::Bones
    } else if session.economy.loose_wood > 0 {
        ResourceKind::Wood
    } else {
        return None;
    };
    if let Some(plan) = worker.haul_plan {
        if plan.resource == resource
            && plan.storage_policy == targets::storage_route_policy(session, worker.id)
            && targets::storage_destination_is_current(session, worker.id, plan.destination)
            && session.economy.loose_amount_at(plan.resource, plan.source) > 0
            && navigation::plan_route(session, worker.position, plan.source).is_ok()
            && navigation::plan_route(session, plan.source, plan.destination).is_ok()
        {
            return Some(plan);
        }
    }
    let source = destination_for_worker(session, worker)?;
    let destination = targets::storage_destination_for(session, source, worker.id)?;
    Some(HaulPlan {
        resource,
        source,
        destination,
        storage_policy: targets::storage_route_policy(session, worker.id),
    })
}

pub(super) fn destination_for_cargo(
    session: &mut GameSession,
    worker_index: usize,
) -> Option<(TilePos, bool)> {
    let worker = session.workforce.workers.get(worker_index)?;
    let resource = worker.carrying_resource.unwrap_or(ResourceKind::Bones);
    let current_policy = targets::storage_route_policy(session, worker.id);
    if let Some(plan) = worker.haul_plan {
        if plan.resource == resource
            && plan.storage_policy == current_policy
            && targets::storage_destination_is_current(session, worker.id, plan.destination)
            && navigation::plan_route(session, worker.position, plan.destination).is_ok()
        {
            return Some((plan.destination, false));
        }
    }

    let worker_id = worker.id;
    let worker_position = worker.position;
    let previous_plan = worker.haul_plan;
    let previous_source = previous_plan
        .map(|plan| plan.source)
        .unwrap_or(worker_position);
    let destination = targets::storage_destination_for(session, worker_position, worker_id);
    let Some(destination) = destination else {
        session.workforce.workers[worker_index].haul_plan = None;
        return None;
    };
    session.workforce.workers[worker_index].haul_plan = Some(HaulPlan {
        resource,
        source: previous_source,
        destination,
        storage_policy: current_policy,
    });
    let replanned = previous_plan.is_none_or(|plan| {
        plan.destination != destination || plan.storage_policy != current_policy
    });
    Some((destination, replanned))
}
