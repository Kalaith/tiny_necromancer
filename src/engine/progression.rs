//! Construction, plot expansion, and the vertical-slice finish line.

use crate::data::GameData;
use crate::engine::suspicion;
use crate::state::{
    Building, BuildingKind, GamePhase, GameSession, PlotStatus, ProductionOrder, Technology,
};
use macroquad_toolkit::grid::TilePos;

pub const MAX_PRODUCTION_QUEUE: usize = 3;

pub fn production_input_need(session: &GameSession, resource: crate::state::ResourceKind) -> i32 {
    let Some(order) = session.progress.production.as_ref() else {
        return 0;
    };
    match resource {
        crate::state::ResourceKind::Bones => order.bones_remaining,
        crate::state::ResourceKind::Wood => order.wood_remaining,
    }
}

pub fn production_destination(session: &GameSession) -> Option<TilePos> {
    session
        .world
        .buildings
        .iter()
        .find(|building| building.kind == BuildingKind::OssuaryKiln && building.complete)
        .map(Building::work_position)
}

pub fn deliver_production_input(
    session: &mut GameSession,
    resource: crate::state::ResourceKind,
    amount: i32,
) -> i32 {
    let delivered = amount.min(production_input_need(session, resource)).max(0);
    let Some(order) = session.progress.production.as_mut() else {
        return 0;
    };
    match resource {
        crate::state::ResourceKind::Bones => order.bones_remaining -= delivered,
        crate::state::ResourceKind::Wood => order.wood_remaining -= delivered,
    }
    delivered
}

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
    if kind == BuildingKind::OssuaryKiln
        && !session.research.is_unlocked(Technology::OssuaryLogistics)
    {
        return Err("Study Ossuary Logistics before raising a kiln.".to_owned());
    }
    if session.has_building(kind) || session.building_in_progress(kind) {
        return Err("That building is already present or under construction.".to_owned());
    }
    let (width, height) = kind.dimensions();
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
    if (0..width).any(|x| {
        (0..height).any(|y| {
            session
                .world
                .is_building_obstacle(TilePos::new(position.x + x, position.y + y))
        })
    }) {
        return Err("That footprint covers a grave, trees, or the road.".to_owned());
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

pub fn start_production(
    session: &mut GameSession,
    data: &GameData,
    kind: BuildingKind,
) -> Result<(), String> {
    if kind == BuildingKind::OssuaryKiln
        && !session.research.is_unlocked(Technology::OssuaryLogistics)
    {
        return Err("Study Ossuary Logistics before loading the kiln.".to_owned());
    }
    if !session.has_building(kind) {
        return Err("The kiln must be complete before it can refine wards.".to_owned());
    }
    let recipe = data
        .buildings
        .get(kind.id())
        .and_then(|building| building.production.as_ref())
        .ok_or_else(|| "That structure has no production recipe.".to_owned())?;
    let active = session.progress.production.is_some();
    if active && session.progress.production_queue >= MAX_PRODUCTION_QUEUE {
        return Err("The kiln's ward queue is full.".to_owned());
    }
    if active {
        if session.economy.bones < recipe.bones_cost || session.economy.wood < recipe.wood_cost {
            return Err(format!(
                "Need {} bones and {} wood to reserve a kiln cycle.",
                recipe.bones_cost, recipe.wood_cost
            ));
        }
        session.economy.bones -= recipe.bones_cost;
        session.economy.wood -= recipe.wood_cost;
        session.progress.production_queue += 1;
        session.add_feed("Another ward cycle is reserved in the kiln.");
    } else {
        let bones_available = session.economy.bones.min(recipe.bones_cost);
        let wood_available = session.economy.wood.min(recipe.wood_cost);
        session.economy.bones -= bones_available;
        session.economy.wood -= wood_available;
        let bones_remaining = recipe.bones_cost - bones_available;
        let wood_remaining = recipe.wood_cost - wood_available;
        session.progress.production = Some(ProductionOrder {
            building: kind,
            progress: 0.0,
            bones_remaining,
            wood_remaining,
        });
        if bones_remaining == 0 && wood_remaining == 0 {
            session.add_feed("The Ossuary Kiln is loaded; assign a worker to Refine Wards.");
        } else {
            session.add_feed(format!(
                "The Ossuary Kiln needs {bones_remaining} bones and {wood_remaining} wood; assign Haul."
            ));
        }
    }
    Ok(())
}

pub fn cancel_production(
    session: &mut GameSession,
    data: &GameData,
    kind: BuildingKind,
) -> Result<(), String> {
    let Some(order) = session.progress.production.as_ref() else {
        return Err("There is no active ward cycle to adjust.".to_owned());
    };
    if order.building != kind {
        return Err("That structure has no active ward cycle.".to_owned());
    }
    if session.progress.production_queue == 0 {
        return Err("There is no reserved ward cycle to cancel.".to_owned());
    }
    let recipe = data
        .buildings
        .get(kind.id())
        .and_then(|building| building.production.as_ref())
        .ok_or_else(|| "That structure has no production recipe.".to_owned())?;
    session.progress.production_queue -= 1;
    session.economy.bones += recipe.bones_cost;
    session.economy.wood += recipe.wood_cost;
    session.add_feed(format!(
        "A reserved ward cycle is cancelled; +{} bones and +{} wood return to storage.",
        recipe.bones_cost, recipe.wood_cost
    ));
    Ok(())
}

pub fn advance_production(session: &mut GameSession, data: &GameData, dt: f32) -> Option<String> {
    let mut order = session.progress.production.clone()?;
    if order.bones_remaining > 0 || order.wood_remaining > 0 {
        session.progress.production = Some(order);
        return None;
    }
    let recipe = data
        .buildings
        .get(order.building.id())
        .and_then(|building| building.production.as_ref())
        .expect("validated production recipe");
    order.progress += dt;
    if order.progress < recipe.seconds {
        session.progress.production = Some(order);
        return None;
    }
    session.economy.ward_charges += recipe.output_amount;
    let next_cycle = session.progress.production_queue > 0;
    if next_cycle {
        session.progress.production_queue -= 1;
        session.progress.production = Some(ProductionOrder {
            building: order.building,
            progress: 0.0,
            bones_remaining: 0,
            wood_remaining: 0,
        });
    } else {
        session.progress.production = None;
    }
    let message = format!(
        "{} Ward charge ready. {}{}",
        recipe.output_amount,
        recipe.effect_text,
        if next_cycle {
            " Next reserved cycle begins."
        } else {
            ""
        }
    );
    session.add_feed(message.clone());
    Some(message)
}

pub fn use_ward_charge(session: &mut GameSession) -> Result<(), String> {
    if session.economy.ward_charges <= 0 {
        return Err("There is no sealed ward charge to spend.".to_owned());
    }
    session.economy.ward_charges -= 1;
    suspicion::adjust_quiet(session, -8.0, "a sealed ward quiets the cemetery");
    session.add_feed("A ward charge is spent; the cemetery falls quiet.");
    Ok(())
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
