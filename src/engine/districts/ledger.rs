//! Recorded district effects and their player-facing summaries.

use super::*;

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
    activity.truncate(super::RECENT_ACTIVITY_LIMIT);
}

fn format_activity(entry: &DistrictActivity) -> String {
    format!("{} @ {:.0}s", activity_label(entry), entry.elapsed_seconds)
}

fn activity_label(entry: &DistrictActivity) -> String {
    match entry.kind {
        DistrictActivityKind::WorkCycle => "Work cycle".to_owned(),
        DistrictActivityKind::StorageBonus => format!("Storage +{:.0} haul", entry.amount),
        DistrictActivityKind::PatrolQuieting => format!("Patrol -{:.1} suspicion", entry.amount),
    }
}
