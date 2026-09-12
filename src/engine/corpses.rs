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
    session.workforce.workers.push(Worker {
        id: worker_id,
        name: if kind == UndeadKind::BruteSkeleton {
            "Thump"
        } else {
            "Knucklebones"
        }
        .to_owned(),
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
