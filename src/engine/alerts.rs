//! Derived operational alerts that point the player toward real blockers.

use crate::data::{GameData, SuspicionStage};
use crate::state::{GameSession, JobKind, PlotStatus, Selection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct OperationalAlert {
    pub severity: AlertSeverity,
    pub title: &'static str,
    pub detail: String,
    pub target: Option<Selection>,
}

impl OperationalAlert {
    fn new(
        severity: AlertSeverity,
        title: &'static str,
        detail: impl Into<String>,
        target: Option<Selection>,
    ) -> Self {
        Self {
            severity,
            title,
            detail: detail.into(),
            target,
        }
    }
}

pub fn collect(session: &GameSession, data: &GameData) -> Vec<OperationalAlert> {
    if session.pressure.active_event.is_some() {
        return Vec::new();
    }
    let mut alerts = Vec::new();
    collect_pressure_alert(session, data, &mut alerts);
    collect_construction_alert(session, &mut alerts);
    collect_production_alert(session, &mut alerts);
    collect_material_alert(session, &mut alerts);
    collect_grave_alert(session, &mut alerts);
    alerts
}

fn collect_pressure_alert(
    session: &GameSession,
    data: &GameData,
    alerts: &mut Vec<OperationalAlert>,
) {
    if session.pressure.suspicion < data.config.suspicion_thresholds[1] {
        return;
    }
    let stage = match session.pressure.stage {
        SuspicionStage::Calm => "Calm",
        SuspicionStage::Rumour => "Rumour",
        SuspicionStage::Questioning => "Questioning",
        SuspicionStage::Investigation => "Investigation",
    };
    let target = session
        .workforce
        .workers
        .iter()
        .position(|worker| worker.assignment != JobKind::Guard)
        .map(Selection::Worker);
    alerts.push(OperationalAlert::new(
        AlertSeverity::Critical,
        "Road pressure",
        format!(
            "{stage} at {:.0}% · assign Guards or spend a ward.",
            session.pressure.suspicion
        ),
        target,
    ));
}

fn collect_construction_alert(session: &GameSession, alerts: &mut Vec<OperationalAlert>) {
    let Some((index, _building)) = session
        .world
        .buildings
        .iter()
        .enumerate()
        .find(|(_, building)| !building.complete)
    else {
        return;
    };
    if session
        .workforce
        .workers
        .iter()
        .any(|worker| worker.assignment == JobKind::Build)
    {
        return;
    }
    alerts.push(OperationalAlert::new(
        AlertSeverity::Warning,
        "Construction stalled",
        "Assign a worker to Build.",
        Some(Selection::Building(index)),
    ));
}

fn collect_production_alert(session: &GameSession, alerts: &mut Vec<OperationalAlert>) {
    let Some(order) = session.progress.production.as_ref() else {
        return;
    };
    if session
        .workforce
        .workers
        .iter()
        .any(|worker| worker.assignment == JobKind::Refine)
    {
        return;
    }
    let target = session
        .world
        .buildings
        .iter()
        .position(|building| building.kind == order.building);
    alerts.push(OperationalAlert::new(
        AlertSeverity::Warning,
        "Kiln unattended",
        format!(
            "Ward cycle loaded · assign a Refine worker ({} queued).",
            session.progress.production_queue
        ),
        target.map(Selection::Building),
    ));
}

fn collect_material_alert(session: &GameSession, alerts: &mut Vec<OperationalAlert>) {
    if session.economy.loose_bones <= 0 && session.economy.loose_wood <= 0 {
        return;
    }
    if session
        .workforce
        .workers
        .iter()
        .any(|worker| worker.assignment == JobKind::Haul || worker.carrying > 0)
    {
        return;
    }
    let source = session
        .economy
        .loose_bones_source
        .or(session.economy.loose_wood_source)
        .unwrap_or_else(crate::state::WorldState::stockpile_position);
    let detail = match (session.economy.loose_bones, session.economy.loose_wood) {
        (bones, wood) if bones > 0 && wood > 0 => {
            format!("{bones} bones and {wood} wood need a Haul order.")
        }
        (bones, _) if bones > 0 => format!("{bones} loose bones need a Haul order."),
        (_, wood) => format!("{wood} loose wood need a Haul order."),
    };
    alerts.push(OperationalAlert::new(
        AlertSeverity::Warning,
        "Materials waiting",
        detail,
        Some(Selection::Ground(source)),
    ));
}

fn collect_grave_alert(session: &GameSession, alerts: &mut Vec<OperationalAlert>) {
    let Some(plot) = session
        .world
        .plots
        .iter()
        .find(|plot| plot.status == PlotStatus::Ready)
    else {
        return;
    };
    if session
        .workforce
        .workers
        .iter()
        .any(|worker| worker.assignment == JobKind::Dig)
    {
        return;
    }
    alerts.push(OperationalAlert::new(
        AlertSeverity::Info,
        "Grave work waiting",
        "An open plot needs a Dig order.",
        Some(Selection::Grave(plot.id)),
    ));
}

#[cfg(test)]
mod tests;
