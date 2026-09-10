//! Deterministic four-way worker routes around cemetery obstructions.

use crate::state::GameSession;
use macroquad_toolkit::grid::TilePos;
use std::collections::{HashMap, VecDeque};

pub fn next_step(session: &GameSession, from: TilePos, target: TilePos) -> Option<TilePos> {
    if from == target {
        return Some(from);
    }
    if !inside(session, target) {
        return None;
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
                return first_step(&previous, from, target);
            }
            frontier.push_back(neighbor);
        }
    }
    None
}

pub fn is_valid_destination(session: &GameSession, tile: TilePos) -> bool {
    inside(session, tile) && !blocked(session, tile)
}

fn first_step(
    previous: &HashMap<TilePos, Option<TilePos>>,
    start: TilePos,
    target: TilePos,
) -> Option<TilePos> {
    let mut current = target;
    while let Some(parent) = previous.get(&current).copied().flatten() {
        if parent == start {
            return Some(current);
        }
        current = parent;
    }
    None
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
