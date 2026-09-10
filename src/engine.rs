//! Deterministic simulation services used by the game session.

pub mod alerts;
pub mod corpses;
pub mod districts;
pub mod jobs;
pub mod movement;
pub mod navigation;
pub mod progression;
pub mod suspicion;

use crate::data::GameData;
use crate::state::GameSession;

#[derive(Debug, Default)]
pub struct TickReport {
    pub messages: Vec<String>,
    pub became_victorious: bool,
}

pub fn simulate_tick(session: &mut GameSession, data: &GameData, dt: f32) -> TickReport {
    let was_victorious = session.phase == crate::state::GamePhase::Victory;
    session.progress.elapsed_seconds += dt;
    session.tick_feed(dt);
    session.economy.mana_fraction += data.config.mana_regen_per_second * dt;
    while session.economy.mana_fraction >= 1.0 && session.economy.mana < data.config.max_mana {
        session.economy.mana_fraction -= 1.0;
        session.economy.mana += 1;
    }
    let mut report = TickReport::default();
    if let Some(message) = movement::simulate_necromancer(session) {
        report.messages.push(message);
    }
    report.messages.extend(jobs::simulate(session, data, dt));
    if let Some(message) = progression::advance_research(session, data, dt) {
        report.messages.push(message);
    }
    suspicion::update_stage(session, data);
    progression::check_victory(session, data);
    report.became_victorious = !was_victorious && session.phase == crate::state::GamePhase::Victory;
    report
}
