//! Corpse discovery, quality bands, and resurrection rules.

use crate::data::GameData;
use crate::engine::suspicion;
use crate::state::{Corpse, CorpseQuality, GameSession, JobKind, UndeadKind, Worker, WorkerStatus};

pub fn discover(session: &mut GameSession, data: &GameData) -> Corpse {
    let roll = session.rng.next_f32();
    let quality = if session.economy.corpses.is_empty() || roll < 0.2 {
        CorpseQuality::Notable
    } else if roll < 0.52 {
        CorpseQuality::Sound
    } else {
        CorpseQuality::Poor
    };
    let quality_id = match quality {
        CorpseQuality::Poor => "poor",
        CorpseQuality::Sound => "sound",
        CorpseQuality::Notable => "notable",
    };
    let band = data
        .corpse_bands
        .get(quality_id)
        .expect("validated corpse band");
    let integrity = band.min_integrity
        + if quality == CorpseQuality::Notable {
            0.15
        } else {
            0.2
        };
    let strength = band.min_strength
        + if quality == CorpseQuality::Notable {
            0.12
        } else {
            0.18
        };
    let corpse = Corpse {
        id: session.economy.corpses.len() as u32 + 1,
        integrity,
        strength,
        skill: session.rng.next_f32(),
        magical_residue: session.rng.range_f32(0.2, 0.95),
        cause_of_death: if session.rng.chance(0.5) {
            "winter fever"
        } else {
            "old battlefield wound"
        }
        .to_owned(),
        quality,
    };
    session.economy.corpses.push(corpse.clone());
    session.progress.first_corpse_found = true;
    session.add_feed(format!("Found a {} corpse remnant.", quality.label()));
    corpse
}

pub fn can_raise(session: &GameSession, data: &GameData, kind: UndeadKind) -> Result<(), String> {
    let Some(def) = data.undead.get(kind.id()) else {
        return Err("Undead recipe is missing from authored data.".to_owned());
    };
    if session.economy.bones < def.bones_cost || session.economy.mana < def.mana_cost {
        return Err(format!(
            "Need {} bones and {} mana.",
            def.bones_cost, def.mana_cost
        ));
    }
    if kind == UndeadKind::BruteSkeleton
        && !session
            .economy
            .corpses
            .iter()
            .any(|corpse| corpse.quality == CorpseQuality::Notable)
    {
        return Err("A Brute needs a Notable corpse remnant.".to_owned());
    }
    Ok(())
}

pub fn raise(session: &mut GameSession, data: &GameData, kind: UndeadKind) -> Result<(), String> {
    raise_with_name(session, data, kind, None)
}

pub fn raise_with_name(
    session: &mut GameSession,
    data: &GameData,
    kind: UndeadKind,
    suggested_name: Option<&str>,
) -> Result<(), String> {
    can_raise(session, data, kind)?;
    let def = data.undead.get(kind.id()).expect("validated undead recipe");
    session.economy.bones -= def.bones_cost;
    session.economy.mana -= def.mana_cost;
    if kind == UndeadKind::BruteSkeleton {
        let corpse_index = session
            .economy
            .corpses
            .iter()
            .position(|corpse| corpse.quality == CorpseQuality::Notable)
            .unwrap();
        session.economy.corpses.remove(corpse_index);
    } else if let Some(corpse_index) = session
        .economy
        .corpses
        .iter()
        .position(|corpse| corpse.quality != CorpseQuality::Notable)
    {
        session.economy.corpses.remove(corpse_index);
    }
    let worker_id = session.workforce.next_worker_id;
    session.workforce.next_worker_id += 1;
    let position = session.world.mana_source;
    let name = suggested_name
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .filter(|name| {
            !session
                .workforce
                .workers
                .iter()
                .any(|worker| worker.name.eq_ignore_ascii_case(name))
        })
        .map(str::to_owned)
        .unwrap_or_else(|| fallback_worker_name(session, kind, worker_id));
    session.workforce.workers.push(Worker {
        id: worker_id,
        name,
        kind,
        assignment: JobKind::Haul,
        position,
        status: WorkerStatus::Idle,
        progress: 0.0,
        target_plot: None,
        carrying: 0,
        carrying_resource: None,
        priority_mode: false,
        haul_plan: None,
    });
    session.workforce.selected_worker = session.workforce.workers.len() - 1;
    session.add_feed(format!("Raised a {}. It reports for Haul duty.", def.name));
    suspicion::adjust(
        session,
        def.conspicuousness,
        "a magical resurrection disturbed the night",
    );
    Ok(())
}

const SKELETON_NAMES: &[&str] = &[
    "Knucklebones",
    "Marrow",
    "Gravewhistle",
    "Dustcap",
    "Clatter",
    "Palehand",
    "Riblet",
    "Mourn",
];

const BRUTE_SKELETON_NAMES: &[&str] = &[
    "Thump",
    "Ossifer",
    "Stonejaw",
    "Gravelord",
    "Ironrib",
    "Boulder",
    "Breakbone",
    "Maw",
];

fn fallback_worker_name(session: &GameSession, kind: UndeadKind, worker_id: u32) -> String {
    let names = match kind {
        UndeadKind::Skeleton => SKELETON_NAMES,
        UndeadKind::BruteSkeleton => BRUTE_SKELETON_NAMES,
    };
    let start = worker_id as usize % names.len();
    for offset in 0..names.len() {
        let candidate = names[(start + offset) % names.len()];
        if !session
            .workforce
            .workers
            .iter()
            .any(|worker| worker.name.eq_ignore_ascii_case(candidate))
        {
            return candidate.to_owned();
        }
    }
    let prefix = match kind {
        UndeadKind::Skeleton => "Skeleton",
        UndeadKind::BruteSkeleton => "Brute",
    };
    let mut suffix = worker_id;
    loop {
        let candidate = format!("{prefix} {suffix}");
        if !session
            .workforce
            .workers
            .iter()
            .any(|worker| worker.name.eq_ignore_ascii_case(&candidate))
        {
            return candidate;
        }
        suffix = suffix.saturating_add(1);
    }
}
