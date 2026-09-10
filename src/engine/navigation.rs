//! Deterministic four-way worker routes around cemetery obstructions.

use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePlan {
    steps: Vec<TilePos>,
}

impl RoutePlan {
    pub fn steps(&self) -> &[TilePos] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len().saturating_sub(1)
    }

    pub fn next_step(&self) -> Option<TilePos> {
        self.steps.get(1).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteFailure {
    OutsideCemetery,
    NoPath,
}

impl RouteFailure {
    pub fn label(self) -> &'static str {
        match self {
            Self::OutsideCemetery => "outside the cemetery",
            Self::NoPath => "obstructions seal the way",
        }
    }
}

pub fn plan_route(
    session: &GameSession,
    from: TilePos,
    target: TilePos,
) -> Result<RoutePlan, RouteFailure> {
    if from == target {
        return Ok(RoutePlan { steps: vec![from] });
    }
    if !inside(session, from) || !inside(session, target) {
        return Err(RouteFailure::OutsideCemetery);
    }
    let mut frontier = VecDeque::from([from]);
    let mut previous = HashMap::from([(from, None)]);
    while let Some(current) = frontier.pop_front() {
        for neighbor in neighbors(current) {
            if previous.contains_key(&neighbor)
                || (!inside(session, neighbor) && neighbor != target)
                || (neighbor != target && blocked(session, neighbor))
            {
                continue;
            }
            previous.insert(neighbor, Some(current));
            if neighbor == target {
                return Ok(RoutePlan {
                    steps: reconstruct_path(&previous, from, target),
                });
            }
            frontier.push_back(neighbor);
        }
    }
    Err(RouteFailure::NoPath)
}

pub fn nearest_reachable<I>(session: &GameSession, from: TilePos, candidates: I) -> Option<TilePos>
where
    I: IntoIterator<Item = TilePos>,
{
    candidates
        .into_iter()
        .filter_map(|candidate| {
            let route = plan_route(session, from, candidate).ok()?;
            Some((route.step_count(), candidate))
        })
        .min_by_key(|(steps, tile)| (*steps, tile.y, tile.x))
        .map(|(_, tile)| tile)
}

pub fn next_step(session: &GameSession, from: TilePos, target: TilePos) -> Option<TilePos> {
    if from == target {
        return Some(from);
    }
    plan_route(session, from, target).ok()?.next_step()
}

pub fn is_valid_destination(session: &GameSession, tile: TilePos) -> bool {
    inside(session, tile) && !blocked(session, tile)
}

fn reconstruct_path(
    previous: &HashMap<TilePos, Option<TilePos>>,
    start: TilePos,
    target: TilePos,
) -> Vec<TilePos> {
    let mut path = vec![target];
    let mut current = target;
    while let Some(parent) = previous.get(&current).copied().flatten() {
        current = parent;
        path.push(current);
    }
    debug_assert_eq!(current, start);
    path.reverse();
    path
}

fn neighbors(tile: TilePos) -> [TilePos; 4] {
    [
        TilePos::new(tile.x - 1, tile.y),
        TilePos::new(tile.x + 1, tile.y),
        TilePos::new(tile.x, tile.y - 1),
        TilePos::new(tile.x, tile.y + 1),
    ]
}

fn inside(session: &GameSession, tile: TilePos) -> bool {
    tile.x >= 0
        && tile.y >= 0
        && tile.x < session.world.width as i32
        && tile.y < session.world.height as i32
}

fn blocked(session: &GameSession, tile: TilePos) -> bool {
    tile.x >= session.world.road_x
        || session.world.forest_tiles.contains(&tile)
        || session.world.buildings.iter().any(|building| {
            tile.x >= building.position.x
                && tile.x < building.position.x + building.width
                && tile.y >= building.position.y
                && tile.y < building.position.y + building.height
        })
}

#[cfg(test)]
mod tests;
