//! Source-to-drop planning for carried resources.

use super::{destination_for_worker, targets};
use crate::data::DistrictRules;
use crate::engine::{districts, navigation, progression};
use crate::state::{GameSession, HaulDestination, HaulPlan, ResourceKind};
use macroquad_toolkit::grid::TilePos;

pub(super) fn plan_haul(session: &GameSession, worker_index: usize) -> Option<HaulPlan> {
    let worker = session.workforce.workers.get(worker_index)?;
    if let Some(plan) = production_plan(session, worker) {
        return Some(plan);
    }
    let resource = if session.economy.loose_bones > 0 {
        ResourceKind::Bones
    } else if session.economy.loose_wood > 0 {
        ResourceKind::Wood
    } else {
        return None;
    };
    if let Some(plan) = worker.haul_plan {
        if plan.destination_kind == HaulDestination::Storage
            && plan.resource == resource
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
        destination_kind: HaulDestination::Storage,
    })
}

fn production_plan(session: &GameSession, worker: &crate::state::Worker) -> Option<HaulPlan> {
    let resource = if progression::production_input_need(session, ResourceKind::Bones) > 0
        && resource_available_for_supply(session, ResourceKind::Bones)
    {
        ResourceKind::Bones
    } else if progression::production_input_need(session, ResourceKind::Wood) > 0
        && resource_available_for_supply(session, ResourceKind::Wood)
    {
        ResourceKind::Wood
    } else {
        return None;
    };
    let source = if stockpiled_supply_available(session, resource) {
        crate::state::WorldState::stockpile_position()
    } else {
        targets::loose_source_for_resource(session, worker, resource)?
    };
    let destination = progression::production_destination(session)?;
    if let Some(plan) = worker.haul_plan {
        if plan.destination_kind == HaulDestination::Kiln
            && plan.resource == resource
            && plan.destination == destination
            && kiln_source_amount(session, plan) > 0
            && navigation::plan_route(session, worker.position, plan.source).is_ok()
            && navigation::plan_route(session, plan.source, destination).is_ok()
        {
            return Some(plan);
        }
    }
    (navigation::plan_route(session, worker.position, source).is_ok()
        && navigation::plan_route(session, source, destination).is_ok())
    .then_some(HaulPlan {
        resource,
        source,
        destination,
        storage_policy: crate::state::RoutePolicy::MarkedFirst,
        destination_kind: HaulDestination::Kiln,
    })
}

fn resource_available_for_supply(session: &GameSession, resource: ResourceKind) -> bool {
    stockpiled_supply_available(session, resource)
        || match resource {
            ResourceKind::Bones => session.economy.loose_bones > 0,
            ResourceKind::Wood => session.economy.loose_wood > 0,
        }
}

fn stockpiled_supply_available(session: &GameSession, resource: ResourceKind) -> bool {
    match resource {
        ResourceKind::Bones => session.economy.bones > 0,
        ResourceKind::Wood => session.economy.wood > 0,
    }
}

fn kiln_source_amount(session: &GameSession, plan: HaulPlan) -> i32 {
    if plan.source == crate::state::WorldState::stockpile_position() {
        match plan.resource {
            ResourceKind::Bones => session.economy.bones,
            ResourceKind::Wood => session.economy.wood,
        }
    } else {
        session.economy.loose_amount_at(plan.resource, plan.source)
    }
}

pub(super) fn destination_for_cargo(
    session: &mut GameSession,
    worker_index: usize,
    rules: &DistrictRules,
) -> Option<(TilePos, bool)> {
    let worker = session.workforce.workers.get(worker_index)?;
    let resource = worker.carrying_resource.unwrap_or(ResourceKind::Bones);
    let current_policy = targets::storage_route_policy(session, worker.id);
    if let Some(plan) = worker.haul_plan {
        let current = match plan.destination_kind {
            HaulDestination::Storage => {
                plan.resource == resource
                    && plan.storage_policy == current_policy
                    && targets::storage_destination_is_current(session, worker.id, plan.destination)
                    && districts::storage_space(session, rules) > 0
                    && navigation::plan_route(session, worker.position, plan.destination).is_ok()
            }
            HaulDestination::Kiln => {
                plan.resource == resource
                    && progression::production_input_need(session, resource) > 0
                    && progression::production_destination(session) == Some(plan.destination)
                    && navigation::plan_route(session, worker.position, plan.destination).is_ok()
            }
        };
        if current {
            return Some((plan.destination, false));
        }
    }

    let worker_id = worker.id;
    let worker_position = worker.position;
    let previous_plan = worker.haul_plan;
    let previous_source = previous_plan
        .map(|plan| plan.source)
        .unwrap_or(worker_position);
    let production_destination = progression::production_destination(session)
        .filter(|_| progression::production_input_need(session, resource) > 0);
    let (destination, destination_kind) = if let Some(destination) = production_destination {
        (destination, HaulDestination::Kiln)
    } else {
        if districts::storage_space(session, rules) <= 0 {
            session.workforce.workers[worker_index].haul_plan = None;
            return None;
        }
        let destination = targets::storage_destination_for(session, worker_position, worker_id);
        let Some(destination) = destination else {
            session.workforce.workers[worker_index].haul_plan = None;
            return None;
        };
        (destination, HaulDestination::Storage)
    };
    let source = if destination_kind == HaulDestination::Kiln {
        previous_plan
            .filter(|plan| plan.destination_kind == HaulDestination::Kiln)
            .map(|plan| plan.source)
            .unwrap_or_else(crate::state::WorldState::stockpile_position)
    } else {
        previous_source
    };
    session.workforce.workers[worker_index].haul_plan = Some(HaulPlan {
        resource,
        source,
        destination,
        storage_policy: current_policy,
        destination_kind,
    });
    let replanned = previous_plan.is_none_or(|plan| {
        plan.destination != destination
            || plan.storage_policy != current_policy
            || plan.destination_kind != destination_kind
    });
    Some((destination, replanned))
}
