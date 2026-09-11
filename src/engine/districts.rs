//! Domain Stewardship rules applied by marked district tiles.

use crate::data::DistrictRules;
use crate::engine::navigation;
use crate::state::{
    DistrictActivity, DistrictActivityKind, GameSession, JobKind, PlotStatus, Technology, ZoneKind,
};
use macroquad_toolkit::grid::TilePos;

const RECENT_ACTIVITY_LIMIT: usize = 8;

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

pub fn storage_capacity(session: &GameSession, rules: &DistrictRules) -> i32 {
    let marked_tiles = if rule_active(session, ZoneKind::Storage) {
        marked_tile_count(session, ZoneKind::Storage) as i32
    } else {
        0
    };
    session
        .economy
        .storage_capacity
        .saturating_add(marked_tiles.saturating_mul(rules.storage_volume_per_tile))
}

pub fn storage_space(session: &GameSession, rules: &DistrictRules) -> i32 {
    session
        .economy
        .storage_space(storage_capacity(session, rules))
}

pub fn storage_summary(session: &GameSession, rules: &DistrictRules) -> String {
    let capacity = storage_capacity(session, rules);
    format!(
        "Storage: {}/{} used · {} room",
        session.economy.stored_materials(),
        capacity,
        storage_space(session, rules)
    )
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
        rules.push(format!(
            "Storage +{} haul · +{} capacity/tile",
            config.storage_capacity_bonus, config.storage_volume_per_tile
        ));
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

pub fn operations_summary(session: &GameSession) -> String {
    let work_tiles = marked_tile_count(session, ZoneKind::Work);
    let storage_tiles = marked_tile_count(session, ZoneKind::Storage);
    let patrol_tiles = marked_tile_count(session, ZoneKind::Patrol);
    let work_operators = operator_count(session, ZoneKind::Work);
    let storage_operators = operator_count(session, ZoneKind::Storage);
    let patrol_operators = operator_count(session, ZoneKind::Patrol);

    let total_gap = total_staffing_gap(session);
    let suffix = if total_gap > 0 {
        format!(
            " · Needs staff ({total_gap} slot{}).",
            if total_gap == 1 { "" } else { "s" }
        )
    } else {
        " · Ready.".to_owned()
    };
    format!(
        "Staffing (workers/marks): Work {}/{} · Storage {}/{} · Patrol {}/{} post{}{}",
        work_operators,
        work_tiles,
        storage_operators,
        storage_tiles,
        patrol_operators,
        patrol_tiles,
        if patrol_tiles == 1 { "" } else { "s" },
        suffix
    )
}

pub fn compact_operations_summary(session: &GameSession) -> String {
    let total_gap = total_staffing_gap(session);
    let suffix = if total_gap > 0 {
        format!(" · GAP {total_gap}")
    } else {
        " · OK".to_owned()
    };
    format!(
        "Staffing: W {}/{} · S {}/{} · P {}/{} posts{}",
        operator_count(session, ZoneKind::Work),
        marked_tile_count(session, ZoneKind::Work),
        operator_count(session, ZoneKind::Storage),
        marked_tile_count(session, ZoneKind::Storage),
        operator_count(session, ZoneKind::Patrol),
        marked_tile_count(session, ZoneKind::Patrol),
        suffix
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistrictCoverage {
    pub marked: usize,
    pub assigned: usize,
    pub reachable: usize,
}

impl DistrictCoverage {
    pub fn needed(self) -> usize {
        self.marked.min(self.assigned)
    }
}

pub fn service_coverage(session: &GameSession, kind: ZoneKind) -> DistrictCoverage {
    let targets = district_tiles(session, kind);
    if targets.is_empty() {
        return DistrictCoverage {
            marked: 0,
            assigned: 0,
            reachable: 0,
        };
    }
    let mut matched_workers = vec![None; targets.len()];
    for worker_index in 0..session.workforce.workers.len() {
        let mut visited = vec![false; targets.len()];
        let _ = match_reachable_worker(
            session,
            kind,
            worker_index,
            &targets,
            &mut matched_workers,
            &mut visited,
        );
    }
    let reachable = matched_workers
        .iter()
        .filter(|worker| worker.is_some())
        .count();
    DistrictCoverage {
        marked: targets.len(),
        assigned: operator_count(session, kind),
        reachable,
    }
}

pub fn coverage_summary(session: &GameSession) -> String {
    let work = service_coverage(session, ZoneKind::Work);
    let storage = service_coverage(session, ZoneKind::Storage);
    let patrol = service_coverage(session, ZoneKind::Patrol);
    format!(
        "Route coverage (reachable/needed): Work {}/{} · Storage {}/{} · Patrol {}/{}",
        work.reachable,
        work.needed(),
        storage.reachable,
        storage.needed(),
        patrol.reachable,
        patrol.needed()
    )
}

pub fn compact_coverage_summary(session: &GameSession) -> String {
    let work = service_coverage(session, ZoneKind::Work);
    let storage = service_coverage(session, ZoneKind::Storage);
    let patrol = service_coverage(session, ZoneKind::Patrol);
    format!(
        "Routes: W {}/{} · S {}/{} · P {}/{}",
        work.reachable,
        work.needed(),
        storage.reachable,
        storage.needed(),
        patrol.reachable,
        patrol.needed()
    )
}

pub fn first_route_gap_worker(session: &GameSession, kind: ZoneKind) -> Option<usize> {
    let coverage = service_coverage(session, kind);
    if coverage.reachable >= coverage.needed() {
        return None;
    }
    let targets = district_tiles(session, kind);
    let eligible_workers = session
        .workforce
        .workers
        .iter()
        .enumerate()
        .filter(|(_, worker)| {
            let serves_target = targets
                .iter()
                .copied()
                .any(|target| worker_serves_tile(session, kind, worker.assignment, target));
            serves_target
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    eligible_workers
        .iter()
        .copied()
        .find(|worker_index| {
            !targets.iter().copied().any(|target| {
                let job = session.workforce.workers[*worker_index].assignment;
                worker_serves_tile(session, kind, job, target)
                    && navigation::plan_route(
                        session,
                        session.workforce.workers[*worker_index].position,
                        target,
                    )
                    .is_ok()
            })
        })
        .or_else(|| eligible_workers.first().copied())
}

pub fn route_coverage_needs_attention(session: &GameSession) -> bool {
    [ZoneKind::Work, ZoneKind::Storage, ZoneKind::Patrol]
        .into_iter()
        .any(|kind| {
            let coverage = service_coverage(session, kind);
            coverage.reachable < coverage.needed()
        })
}

pub fn route_gap_district(session: &GameSession, worker_index: usize) -> Option<ZoneKind> {
    if worker_index >= session.workforce.workers.len() {
        return None;
    }
    [ZoneKind::Work, ZoneKind::Storage, ZoneKind::Patrol]
        .into_iter()
        .find(|kind| first_route_gap_worker(session, *kind) == Some(worker_index))
}

pub fn staffing_needs_attention(session: &GameSession) -> bool {
    if !session.research.is_unlocked(Technology::DomainStewardship) {
        return false;
    }
    [ZoneKind::Work, ZoneKind::Storage, ZoneKind::Patrol]
        .into_iter()
        .any(|kind| staffing_gap(session, kind) > 0)
}

pub fn policy_bias(session: &GameSession, job: JobKind) -> usize {
    let baseline_bias = session.stewardship_policy.bias(job);
    if session.stewardship_policy != crate::state::StewardshipPolicy::Harvest
        || !session.research.is_unlocked(Technology::DomainStewardship)
    {
        return baseline_bias;
    }

    let work_staffing = work_staffing(session);
    let work_gap = staffing_gap(session, ZoneKind::Work) > 0;
    let storage_gap = staffing_gap(session, ZoneKind::Storage) > 0;
    if work_staffing.dig_gap > 0 && job == JobKind::Dig {
        return 0;
    }
    if work_staffing.wood_gap > 0 && job == JobKind::Wood {
        return 0;
    }
    if work_staffing.flexible_gap > 0 && matches!(job, JobKind::Dig | JobKind::Wood) {
        return 0;
    }
    if storage_gap && job == JobKind::Haul {
        return 0;
    }
    if work_gap || storage_gap {
        baseline_bias.saturating_add(1)
    } else {
        baseline_bias
    }
}

pub fn tile_summary(
    session: &GameSession,
    config: &DistrictRules,
    tile: TilePos,
) -> Option<String> {
    let kinds = [ZoneKind::Work, ZoneKind::Storage, ZoneKind::Patrol]
        .into_iter()
        .filter(|kind| session.world.zone_contains(*kind, tile))
        .collect::<Vec<_>>();
    if kinds.is_empty() {
        return None;
    }

    let domain_unlocked = session.research.is_unlocked(Technology::DomainStewardship);
    if !domain_unlocked && kinds.len() > 1 {
        let labels = kinds
            .iter()
            .map(|kind| format!("{} district", kind.label()))
            .collect::<Vec<_>>();
        return Some(format!(
            "{} · Domain Stewardship will activate their local rules.",
            labels.join(" + ")
        ));
    }
    let summaries = kinds
        .into_iter()
        .map(|kind| {
            let tile_count = session
                .world
                .zones
                .iter()
                .filter(|zone| zone.kind == kind)
                .map(|zone| zone.tiles.len())
                .sum::<usize>();
            let detail = if !domain_unlocked {
                "Domain Stewardship will activate its local rule.".to_owned()
            } else {
                match kind {
                    ZoneKind::Work => format!(
                        "+{:.0}% Dig/Wood speed here.",
                        (config.work_speed_multiplier - 1.0) * 100.0
                    ),
                    ZoneKind::Storage => {
                        format!(
                            "+{} Haul · +{} storage here.",
                            config.storage_capacity_bonus, config.storage_volume_per_tile
                        )
                    }
                    ZoneKind::Patrol => format!(
                        "+{:.0}% Guard mitigation here.",
                        (config.patrol_mitigation_multiplier - 1.0) * 100.0
                    ),
                }
            };
            format!(
                "{} district · {} marked tile{} · {}",
                kind.label(),
                tile_count,
                if tile_count == 1 { "" } else { "s" },
                detail
            )
        })
        .collect::<Vec<_>>();

    let separator = if summaries.len() == 1 { "" } else { "\n" };
    Some(summaries.join(separator))
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
        "Ledger: Work {} cycle{} · Storage +{} haul · Patrol -{:.1} suspicion",
        ledger.work_cycles,
        if ledger.work_cycles == 1 { "" } else { "s" },
        ledger.storage_bonus_items,
        ledger.patrol_quieting
    )
}

pub fn activity_summary(session: &GameSession) -> String {
    let entries = session
        .progress
        .district_ledger
        .recent_activity
        .iter()
        .take(2)
        .map(format_activity)
        .collect::<Vec<_>>();
    if entries.is_empty() {
        "Activity: no marked district effect recorded yet.".to_owned()
    } else {
        format!("Activity: {}", entries.join(" · "))
    }
}

pub fn latest_activity_summary(session: &GameSession) -> String {
    session
        .progress
        .district_ledger
        .recent_activity
        .first()
        .map_or_else(
            || "Last district effect: none recorded.".to_owned(),
            |entry| {
                format!(
                    "Last: {} @ {:.0}s",
                    activity_label(entry),
                    entry.elapsed_seconds
                )
            },
        )
}

pub fn record_work_cycle(session: &mut GameSession) {
    let first = {
        let ledger = &mut session.progress.district_ledger;
        let first = ledger.work_cycles == 0;
        ledger.work_cycles = ledger.work_cycles.saturating_add(1);
        first
    };
    record_activity(session, DistrictActivityKind::WorkCycle, 1.0);
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
    record_activity(session, DistrictActivityKind::StorageBonus, amount as f32);
    if first {
        session.add_feed("Marked Storage gives a hauler extra room on its first pickup.");
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
    record_activity(session, DistrictActivityKind::PatrolQuieting, amount);
    if first {
        session.add_feed("A marked Patrol post quiets the road more effectively.");
    }
}

fn record_activity(session: &mut GameSession, kind: DistrictActivityKind, amount: f32) {
    let activity = &mut session.progress.district_ledger.recent_activity;
    activity.insert(
        0,
        DistrictActivity {
            kind,
            amount,
            elapsed_seconds: session.progress.elapsed_seconds,
        },
    );
    activity.truncate(RECENT_ACTIVITY_LIMIT);
}

fn format_activity(entry: &DistrictActivity) -> String {
    format!("{} @ {:.0}s", activity_label(entry), entry.elapsed_seconds)
}

fn activity_label(entry: &DistrictActivity) -> String {
    match entry.kind {
        DistrictActivityKind::WorkCycle => "Work cycle".to_owned(),
        DistrictActivityKind::StorageBonus => format!("Storage +{:.0} haul", entry.amount),
        DistrictActivityKind::PatrolQuieting => {
            format!("Patrol -{:.1} suspicion", entry.amount)
        }
    }
}

pub fn marked_tile_count(session: &GameSession, kind: ZoneKind) -> usize {
    session
        .world
        .zones
        .iter()
        .filter(|zone| zone.kind == kind)
        .map(|zone| zone.tiles.len())
        .sum()
}

pub fn operator_count(session: &GameSession, kind: ZoneKind) -> usize {
    if kind == ZoneKind::Work && marked_tile_count(session, kind) > 0 {
        work_staffing(session).staffed
    } else {
        assigned_operator_count(session, kind)
    }
}

pub fn staffing_gap(session: &GameSession, kind: ZoneKind) -> usize {
    marked_tile_count(session, kind).saturating_sub(operator_count(session, kind))
}

pub fn total_staffing_gap(session: &GameSession) -> usize {
    [ZoneKind::Work, ZoneKind::Storage, ZoneKind::Patrol]
        .into_iter()
        .map(|kind| staffing_gap(session, kind))
        .sum()
}

fn rule_active(session: &GameSession, kind: ZoneKind) -> bool {
    session.research.is_unlocked(Technology::DomainStewardship)
        && session
            .world
            .zones
            .iter()
            .any(|zone| zone.kind == kind && !zone.tiles.is_empty())
}

#[derive(Debug, Default, PartialEq, Eq)]
struct WorkStaffing {
    staffed: usize,
    dig_gap: usize,
    wood_gap: usize,
    flexible_gap: usize,
}

fn work_staffing(session: &GameSession) -> WorkStaffing {
    let mut dig_slots = 0;
    let mut wood_slots = 0;
    let mut flexible_slots = 0;
    for zone in session
        .world
        .zones
        .iter()
        .filter(|zone| zone.kind == ZoneKind::Work)
    {
        for tile in &zone.tiles {
            if session.world.plots.iter().any(|plot| {
                plot.position == *tile
                    && matches!(plot.status, PlotStatus::Ready | PlotStatus::Digging)
            }) {
                dig_slots += 1;
            } else if session.world.forest_tiles.contains(tile) {
                wood_slots += 1;
            } else {
                flexible_slots += 1;
            }
        }
    }

    let dig_operators = job_operator_count(session, JobKind::Dig);
    let wood_operators = job_operator_count(session, JobKind::Wood);
    let staffed_dig = dig_slots.min(dig_operators);
    let staffed_wood = wood_slots.min(wood_operators);
    let flexible_operators = dig_operators
        .saturating_sub(staffed_dig)
        .saturating_add(wood_operators.saturating_sub(staffed_wood));
    let staffed_flexible = flexible_slots.min(flexible_operators);

    WorkStaffing {
        staffed: staffed_dig + staffed_wood + staffed_flexible,
        dig_gap: dig_slots.saturating_sub(staffed_dig),
        wood_gap: wood_slots.saturating_sub(staffed_wood),
        flexible_gap: flexible_slots.saturating_sub(staffed_flexible),
    }
}

fn assigned_operator_count(session: &GameSession, kind: ZoneKind) -> usize {
    session
        .workforce
        .workers
        .iter()
        .filter(|worker| match kind {
            ZoneKind::Work => matches!(worker.assignment, JobKind::Dig | JobKind::Wood),
            ZoneKind::Storage => worker.assignment == JobKind::Haul,
            ZoneKind::Patrol => worker.assignment == JobKind::Guard,
        })
        .count()
}

fn district_tiles(session: &GameSession, kind: ZoneKind) -> Vec<TilePos> {
    session
        .world
        .zones
        .iter()
        .filter(|zone| zone.kind == kind)
        .flat_map(|zone| zone.tiles.iter().copied())
        .collect()
}

fn match_reachable_worker(
    session: &GameSession,
    kind: ZoneKind,
    worker_index: usize,
    targets: &[TilePos],
    matched_workers: &mut [Option<usize>],
    visited: &mut [bool],
) -> bool {
    let worker = &session.workforce.workers[worker_index];
    for (target_index, target) in targets.iter().enumerate() {
        if visited[target_index]
            || !worker_serves_tile(session, kind, worker.assignment, *target)
            || navigation::plan_route(session, worker.position, *target).is_err()
        {
            continue;
        }
        visited[target_index] = true;
        if matched_workers[target_index].is_none()
            || match_reachable_worker(
                session,
                kind,
                matched_workers[target_index].expect("checked matched worker"),
                targets,
                matched_workers,
                visited,
            )
        {
            matched_workers[target_index] = Some(worker_index);
            return true;
        }
    }
    false
}

fn worker_serves_tile(session: &GameSession, kind: ZoneKind, job: JobKind, tile: TilePos) -> bool {
    match kind {
        ZoneKind::Work => work_role_for_tile(session, tile)
            .map_or(matches!(job, JobKind::Dig | JobKind::Wood), |required| {
                job == required
            }),
        ZoneKind::Storage => job == JobKind::Haul,
        ZoneKind::Patrol => job == JobKind::Guard,
    }
}

fn work_role_for_tile(session: &GameSession, tile: TilePos) -> Option<JobKind> {
    if session.world.plots.iter().any(|plot| {
        plot.position == tile && matches!(plot.status, PlotStatus::Ready | PlotStatus::Digging)
    }) {
        Some(JobKind::Dig)
    } else if session.world.forest_tiles.contains(&tile) {
        Some(JobKind::Wood)
    } else {
        None
    }
}

fn job_operator_count(session: &GameSession, job: JobKind) -> usize {
    session
        .workforce
        .workers
        .iter()
        .filter(|worker| worker.assignment == job)
        .count()
}

#[cfg(test)]
mod tests;
