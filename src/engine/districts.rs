//! Domain Stewardship rules applied by marked district tiles.

use crate::data::DistrictRules;
use crate::state::{GameSession, Technology, ZoneKind};
use macroquad_toolkit::grid::TilePos;

pub fn work_speed_multiplier(session: &GameSession, rules: &DistrictRules, tile: TilePos) -> f32 {
    if rule_active(session, ZoneKind::Work) && session.world.zone_contains(ZoneKind::Work, tile) {
        rules.work_speed_multiplier
    } else {
        1.0
    }
}

pub fn haul_capacity_bonus(session: &GameSession, rules: &DistrictRules, tile: TilePos) -> i32 {
    if rule_active(session, ZoneKind::Storage)
        && session.world.zone_contains(ZoneKind::Storage, tile)
    {
        rules.storage_capacity_bonus
    } else {
        0
    }
}

pub fn guard_mitigation_multiplier(
    session: &GameSession,
    rules: &DistrictRules,
    tile: TilePos,
) -> f32 {
    if rule_active(session, ZoneKind::Patrol) && session.world.zone_contains(ZoneKind::Patrol, tile)
    {
        rules.patrol_mitigation_multiplier
    } else {
        1.0
    }
}

pub fn rule_summary(session: &GameSession, config: &DistrictRules) -> String {
    let mut rules = Vec::new();
    if rule_active(session, ZoneKind::Work) {
        rules.push(format!(
            "Work +{:.0}% speed",
            (config.work_speed_multiplier - 1.0) * 100.0
        ));
    }
    if rule_active(session, ZoneKind::Storage) {
        rules.push(format!("Storage +{} haul", config.storage_capacity_bonus));
    }
    if rule_active(session, ZoneKind::Patrol) {
        rules.push(format!(
            "Patrol +{:.0}% quieting",
            (config.patrol_mitigation_multiplier - 1.0) * 100.0
        ));
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
