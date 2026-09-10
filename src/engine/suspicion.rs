//! Suspicion accumulation, escalation stages, and non-combat event choices.

use crate::data::{GameData, SuspicionStage};
use crate::state::GameSession;

pub fn adjust(session: &mut GameSession, amount: f32, reason: &str) {
    adjust_inner(session, amount, reason, true);
}

pub fn adjust_quiet(session: &mut GameSession, amount: f32, reason: &str) {
    adjust_inner(session, amount, reason, false);
}

fn adjust_inner(session: &mut GameSession, amount: f32, reason: &str, announce: bool) {
    if amount.abs() < f32::EPSILON {
        return;
    }
    session.pressure.suspicion = (session.pressure.suspicion + amount).clamp(0.0, 100.0);
    session.pressure.last_reason = format!("{} ({:+.1})", reason, amount);
    if !announce {
        return;
    }
    if amount > 0.0 {
        session.add_feed(format!("Suspicion +{amount:.1}: {reason}."));
    } else {
        session.add_feed(format!("Suspicion {amount:.1}: {reason}."));
    }
}

pub fn update_stage(session: &mut GameSession, data: &GameData) {
    let previous = session.pressure.stage;
    let thresholds = data.config.suspicion_thresholds;
    let next = if session.pressure.suspicion >= thresholds[2] {
        SuspicionStage::Investigation
    } else if session.pressure.suspicion >= thresholds[1] {
        SuspicionStage::Questioning
    } else if session.pressure.suspicion >= thresholds[0] {
        SuspicionStage::Rumour
    } else {
        SuspicionStage::Calm
    };
    if next == previous {
        return;
    }
    session.pressure.stage = next;
    if stage_rank(next) <= stage_rank(previous) {
        return;
    }
    if let Some(event) = data.event_for_stage(next) {
        if !session
            .pressure
            .event_history
            .iter()
            .any(|id| id == &event.id)
        {
            session.pressure.active_event = Some(event.id.clone());
            session.add_feed(format!("Human pressure: {}", event.title));
        }
    }
}

fn stage_rank(stage: SuspicionStage) -> u8 {
    match stage {
        SuspicionStage::Calm => 0,
        SuspicionStage::Rumour => 1,
        SuspicionStage::Questioning => 2,
        SuspicionStage::Investigation => 3,
    }
}

pub fn resolve_event(
    session: &mut GameSession,
    data: &GameData,
    choice_id: &str,
) -> Result<(), String> {
    let Some(event_id) = session.pressure.active_event.clone() else {
        return Err("There is no event waiting for a choice.".to_owned());
    };
    let event = data
        .events
        .get(&event_id)
        .ok_or_else(|| "Event content is missing.".to_owned())?;
    let choice = event
        .choices
        .iter()
        .find(|choice| choice.id == choice_id)
        .ok_or_else(|| "That event choice is not available.".to_owned())?;
    let delta = choice.suspicion_delta;
    session.economy.bones = (session.economy.bones + choice.bones_delta).max(0);
    session.economy.mana = (session.economy.mana + choice.mana_delta).max(0);
    session.economy.wood = (session.economy.wood + choice.wood_delta).max(0);
    session.pressure.active_event = None;
    session.pressure.event_history.push(event_id.clone());
    if choice.pause_digging {
        for worker in &mut session.workforce.workers {
            if worker.assignment == crate::state::JobKind::Dig {
                worker.assignment = crate::state::JobKind::Guard;
                worker.progress = 0.0;
                worker.target_plot = None;
            }
        }
        session.add_feed("Digging workers are hiding until the road is quiet.");
    }
    adjust(session, delta, &format!("you chose {}", choice.label));
    update_stage(session, data);
    Ok(())
}

#[cfg(test)]
mod tests;
