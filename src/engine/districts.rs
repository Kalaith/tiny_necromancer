//! Domain Stewardship rules applied by marked district tiles.

use crate::state::{GameSession, Technology, ZoneKind};
use macroquad_toolkit::grid::TilePos;

pub const WORK_SPEED_MULTIPLIER: f32 = 1.15;
pub const STORAGE_CAPACITY_BONUS: i32 = 4;
pub const PATROL_MITIGATION_MULTIPLIER: f32 = 1.25;

pub fn work_speed_multiplier(session: &GameSession, tile: TilePos) -> f32 {
    if rule_active(session, ZoneKind::Work) && session.world.zone_contains(ZoneKind::Work, tile) {
        WORK_SPEED_MULTIPLIER
    } else {
        1.0
    }
}

pub fn haul_capacity_bonus(session: &GameSession) -> i32 {
    if rule_active(session, ZoneKind::Storage) {
        STORAGE_CAPACITY_BONUS
    } else {
        0
    }
}

pub fn guard_mitigation_multiplier(session: &GameSession) -> f32 {
    if rule_active(session, ZoneKind::Patrol) {
        PATROL_MITIGATION_MULTIPLIER
    } else {
        1.0
    }
}

pub fn rule_summary(session: &GameSession) -> String {
    let mut rules = Vec::new();
    if rule_active(session, ZoneKind::Work) {
        rules.push("Work +15% speed");
    }
    if rule_active(session, ZoneKind::Storage) {
        rules.push("Storage +4 haul");
    }
    if rule_active(session, ZoneKind::Patrol) {
        rules.push("Patrol +25% quieting");
    }
    if rules.is_empty() {
        "No district rules active · mark a Work, Storage, or Patrol tile.".to_owned()
    } else {
        format!("Rules: {}", rules.join(" · "))
    }
}

fn rule_active(session: &GameSession, kind: ZoneKind) -> bool {
    session.research.is_unlocked(Technology::DomainStewardship)
        && session
            .world
            .zones
            .iter()
            .any(|zone| zone.kind == kind && !zone.tiles.is_empty())
}

#[cfg(test)]
mod tests;
