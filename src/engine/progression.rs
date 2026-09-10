//! Construction, plot expansion, and the vertical-slice finish line.

use crate::data::GameData;
use crate::engine::suspicion;
use crate::state::{Building, BuildingKind, GamePhase, GameSession, PlotStatus, Technology};
use macroquad_toolkit::grid::TilePos;

pub fn queue_building(
    session: &mut GameSession,
    data: &GameData,
    kind: BuildingKind,
) -> Result<(), String> {
    queue_building_at(
        session,
        data,
        kind,
        crate::state::default_building_position_for_kind(kind),
    )
}

pub fn queue_building_at(
    session: &mut GameSession,
    data: &GameData,
    kind: BuildingKind,
    position: TilePos,
) -> Result<(), String> {
    if session.has_building(kind) || session.building_in_progress(kind) {
        return Err("That building is already present or under construction.".to_owned());
    }
    let width = if kind == BuildingKind::WorkShed { 2 } else { 1 };
    let height = if kind == BuildingKind::WorkShed { 2 } else { 1 };
    if position.x < 0
        || position.y < 0
        || position.x + width > session.world.width as i32
        || position.y + height > session.world.height as i32
    {
        return Err("That footprint would fall outside the cemetery boundary.".to_owned());
    }
    let overlaps = session.world.buildings.iter().any(|building| {
        let ax = position.x < building.position.x + building.width
            && position.x + width > building.position.x;
        let ay = position.y < building.position.y + building.height
            && position.y + height > building.position.y;
        ax && ay
    });
    if overlaps {
        return Err("That footprint overlaps another structure.".to_owned());
    }
    let def = data
        .buildings
        .get(kind.id())
        .expect("validated building recipe");
    if session.economy.bones < def.bones_cost
        || session.economy.mana < def.mana_cost
        || session.economy.wood < def.wood_cost
    {
        return Err(format!(
            "Need {} bones, {} mana, and {} wood.",
            def.bones_cost, def.mana_cost, def.wood_cost
        ));
    }
    session.economy.bones -= def.bones_cost;
    session.economy.mana -= def.mana_cost;
    session.economy.wood -= def.wood_cost;
    session.world.buildings.push(Building {
        kind,
        progress: 0.0,
        complete: false,
        position,
        width,
        height,
    });
    session.progress.first_building_started = true;
    session.add_feed(format!("Construction started: {}.", def.name));
    suspicion::adjust(
        session,
        def.suspicion_delta,
        "new construction near the road",
    );
    Ok(())
}

pub fn advance_construction(session: &mut GameSession, data: &GameData, dt: f32) -> Option<String> {
    let index = session
        .world
        .buildings
        .iter()
        .position(|building| !building.complete)?;
    let kind = session.world.buildings[index].kind;
    let def = data
        .buildings
        .get(kind.id())
        .expect("validated building recipe");
    let shed_bonus = if session.has_building(BuildingKind::WorkShed) {
        data.buildings
            .get(BuildingKind::WorkShed.id())
            .expect("validated work shed")
            .speed_multiplier
    } else {
        1.0
    };
    session.world.buildings[index].progress += dt * shed_bonus;
    if session.world.buildings[index].progress >= def.build_seconds {
        session.world.buildings[index].progress = def.build_seconds;
        session.world.buildings[index].complete = true;
        if kind == BuildingKind::WorkShed {
            session.economy.shovels += 1;
        }
        let message = format!("{} is complete: {}", def.name, def.effect_text);
        session.add_feed(message.clone());
        Some(message)
    } else {
        None
    }
}

pub fn start_research(session: &mut GameSession, technology: Technology) -> Result<(), String> {
    if !session.has_building(BuildingKind::WorkShed) {
        return Err("Restore the work shed before studying bindings.".to_owned());
    }
    if !session.research.can_start(technology) {
        return Err(match session.research.current {
            Some(current) => format!("The shed is already studying {}.", current.label()),
            None => format!("{} is not reachable yet.", technology.label()),
        });
    }
    session.research.current = Some(technology);
    session.research.progress = 0.0;
    session.add_feed(format!("Study begun: {}.", technology.label()));
    Ok(())
}

pub fn advance_research(session: &mut GameSession, data: &GameData, dt: f32) -> Option<String> {
    let technology = session.research.current?;
    let duration = technology.duration(&data.config);
    session.research.progress += dt;
    if session.research.progress < duration {
        return None;
    }
    session.research.progress = duration;
    session.research.current = None;
    session.research.completed.push(technology);
    let message = format!(
        "Research complete: {} — {}",
        technology.label(),
        technology.description()
    );
    session.add_feed(message.clone());
    Some(message)
}

pub fn unlock_plot(session: &mut GameSession, data: &GameData) -> Result<(), String> {
    if session.progress.unlocked_plots >= data.config.victory_plots {
        return Err("All six cemetery plots are already usable.".to_owned());
    }
    let cost = crate::state::next_plot_unlock_cost(&data.config, session.progress.unlocked_plots);
    if session.economy.wood < cost {
        return Err(format!("Opening the next plot needs {cost} wood."));
    }
    let next = session.progress.unlocked_plots;
    let Some(plot) = session.world.plots.get_mut(next) else {
        return Err("No authored plot remains to unlock.".to_owned());
    };
    plot.status = PlotStatus::Ready;
    session.progress.unlocked_plots += 1;
    session.economy.wood -= cost;
    session.add_feed(format!(
        "Plot {} is now usable. The road feels closer.",
        next + 1
    ));
    suspicion::adjust(session, 3.0, "expanding the disturbed cemetery");
    Ok(())
}

pub fn check_victory(session: &mut GameSession, data: &GameData) {
    let has_brute = session
        .workforce
        .workers
        .iter()
        .any(|worker| worker.kind == crate::state::UndeadKind::BruteSkeleton);
    if session.workforce.workers.len() >= data.config.victory_undead
        && session.progress.unlocked_plots >= data.config.victory_plots
        && session.has_building(BuildingKind::WorkShed)
        && session.has_building(BuildingKind::GraveLantern)
        && has_brute
        && session.pressure.suspicion < data.config.victory_suspicion_max
    {
        session.phase = GamePhase::Victory;
        session.add_feed("The cemetery is a tiny, thriving operation. You made it.");
    }
}

#[cfg(test)]
mod tests;
