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
            "Work +{:.0}%",
            (config.work_speed_multiplier - 1.0) * 100.0
        ));
    }
    if rule_active(session, ZoneKind::Storage) {
        rules.push(format!("Storage +{}", config.storage_capacity_bonus));
    }
    if rule_active(session, ZoneKind::Patrol) {
        rules.push(format!(
            "Patrol +{:.0}%",
            (config.patrol_mitigation_multiplier - 1.0) * 100.0
        ));
    }
    if rules.is_empty() {
        "No district rules active · mark a Work, Storage, or Patrol tile.".to_owned()
    } else {
        format!("Rules: marked tiles · {}", rules.join(" · "))
    }
}

pub fn ledger_summary(session: &GameSession) -> String {
    let ledger = &session.progress.district_ledger;
    if ledger.work_cycles == 0
        && ledger.storage_bonus_items == 0
        && ledger.patrol_quieting <= f32::EPSILON
    {
        return "Ledger: no marked district effect recorded yet.".to_owned();
    }
    format!(
        "Ledger: Work {} cycle{} · Storage +{} haul · Patrol {:.1} quieted",
        ledger.work_cycles,
        if ledger.work_cycles == 1 { "" } else { "s" },
        ledger.storage_bonus_items,
        ledger.patrol_quieting
    )
}

pub fn record_work_cycle(session: &mut GameSession) {
    let first = {
        let ledger = &mut session.progress.district_ledger;
        let first = ledger.work_cycles == 0;
        ledger.work_cycles = ledger.work_cycles.saturating_add(1);
        first
    };
    if first {
        session.add_feed("A marked Work tile completes its first accelerated cycle.");
    }
}

pub fn record_storage_bonus(session: &mut GameSession, amount: i32) {
    if amount <= 0 {
        return;
    }
    let first = {
        let ledger = &mut session.progress.district_ledger;
        let first = ledger.storage_bonus_items == 0;
        ledger.storage_bonus_items = ledger.storage_bonus_items.saturating_add(amount);
        first
    };
    if first {
        session.add_feed("Marked Storage gives a hauler extra room on its first drop.");
    }
}

pub fn record_patrol_quieting(session: &mut GameSession, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    let first = {
        let ledger = &mut session.progress.district_ledger;
        let first = ledger.patrol_quieting <= f32::EPSILON;
        ledger.patrol_quieting += amount;
        first
    };
    if first {
        session.add_feed("A marked Patrol post quiets the road more effectively.");
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
