//! Tile-authoritative movement orders and render-time actor interpolation.

use crate::engine::navigation;
use crate::state::{GameSession, ResourceKind};
use macroquad::prelude::{vec2, Vec2};
use macroquad_toolkit::grid::TilePos;
use std::collections::HashMap;

pub const WALK_SECONDS: f32 = 0.38;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

impl Facing {
    fn from_step(from: TilePos, to: TilePos, fallback: Self) -> Self {
        let delta = TilePos::new(to.x - from.x, to.y - from.y);
        if delta.x < 0 {
            Self::Left
        } else if delta.x > 0 {
            Self::Right
        } else if delta.y < 0 {
            Self::Up
        } else if delta.y > 0 {
            Self::Down
        } else {
            fallback
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActorMotion {
    previous: TilePos,
    target: TilePos,
    progress: f32,
    facing: Facing,
}

impl ActorMotion {
    pub fn new(tile: TilePos) -> Self {
        Self {
            previous: tile,
            target: tile,
            progress: 1.0,
            facing: Facing::Down,
        }
    }

    fn sync(&mut self, logical_tile: TilePos, dt: f32, paused: bool) {
        if logical_tile != self.target {
            self.previous = self.target;
            self.target = logical_tile;
            self.progress = 0.0;
            self.facing = Facing::from_step(self.previous, self.target, self.facing);
        }
        if !paused && self.previous != self.target {
            self.progress = (self.progress + dt / WALK_SECONDS).min(1.0);
        }
    }

    pub fn visual_position(&self) -> Vec2 {
        let from = vec2(self.previous.x as f32, self.previous.y as f32);
        let to = vec2(self.target.x as f32, self.target.y as f32);
        from.lerp(to, self.progress)
    }

    pub fn facing(&self) -> Facing {
        self.facing
    }

    pub fn is_walking(&self) -> bool {
        self.previous != self.target && self.progress < 1.0
    }

    fn occupies_tile(&self, tile: TilePos) -> bool {
        self.target == tile || (self.is_walking() && self.previous == tile)
    }
}

#[derive(Debug, Clone)]
pub struct MotionState {
    workers: HashMap<u32, ActorMotion>,
    necromancer: ActorMotion,
}

impl MotionState {
    pub fn new(session: &GameSession) -> Self {
        let mut motion = Self {
            workers: HashMap::new(),
            necromancer: ActorMotion::new(session.world.necromancer_position),
        };
        motion.reset(session);
        motion
    }

    pub fn reset(&mut self, session: &GameSession) {
        self.workers.clear();
        for worker in &session.workforce.workers {
            self.workers
                .insert(worker.id, ActorMotion::new(worker.position));
        }
        self.necromancer = ActorMotion::new(session.world.necromancer_position);
    }

    pub fn update(&mut self, session: &GameSession, dt: f32, paused: bool) {
        let frame_dt = dt.clamp(0.0, 0.1);
        let active_ids: Vec<u32> = session
            .workforce
            .workers
            .iter()
            .map(|worker| worker.id)
            .collect();
        self.workers
            .retain(|worker_id, _| active_ids.contains(worker_id));
        for worker in &session.workforce.workers {
            self.workers
                .entry(worker.id)
                .or_insert_with(|| ActorMotion::new(worker.position))
                .sync(worker.position, frame_dt, paused);
        }
        self.necromancer
            .sync(session.world.necromancer_position, frame_dt, paused);
    }

    pub fn worker(&self, worker_id: u32) -> Option<&ActorMotion> {
        self.workers.get(&worker_id)
    }

    pub fn worker_position(&self, worker_id: u32) -> Option<Vec2> {
        self.worker(worker_id).map(ActorMotion::visual_position)
    }

    pub fn worker_occupies_tile(&self, worker_id: u32, tile: TilePos) -> bool {
        self.worker(worker_id)
            .is_some_and(|motion| motion.occupies_tile(tile))
    }

    pub fn necromancer(&self) -> &ActorMotion {
        &self.necromancer
    }

    pub fn necromancer_occupies_tile(&self, tile: TilePos) -> bool {
        self.necromancer.occupies_tile(tile)
    }
}

pub fn request_necromancer_destination(
    session: &mut GameSession,
    destination: TilePos,
) -> Result<(), String> {
    if destination == session.world.necromancer_position {
        session.world.necromancer_destination = None;
        return Ok(());
    }
    if !navigation::is_valid_destination(session, destination) {
        return Err("That tile is blocked or outside the cemetery.".to_owned());
    }
    if let Err(failure) =
        navigation::plan_route(session, session.world.necromancer_position, destination)
    {
        return Err(format!(
            "The necromancer cannot reach that tile: {}.",
            failure.label()
        ));
    }
    session.world.necromancer_destination = Some(destination);
    Ok(())
}

pub fn simulate_necromancer(session: &mut GameSession) -> Option<String> {
    let destination = session.world.necromancer_destination?;
    if destination == session.world.necromancer_position {
        session.world.necromancer_destination = None;
        return None;
    }
    let current = session.world.necromancer_position;
    match navigation::plan_route(session, current, destination) {
        Ok(route) => {
            let Some(next) = route.next_step() else {
                session.world.necromancer_destination = None;
                return None;
            };
            session.world.necromancer_position = next;
            if next == destination {
                session.world.necromancer_destination = None;
            }
            None
        }
        Err(failure) => {
            session.world.necromancer_destination = None;
            Some(format!(
                "The necromancer's route failed: {}.",
                failure.label()
            ))
        }
    }
}

pub fn drop_worker_cargo(session: &mut GameSession, index: usize) {
    let Some(worker) = session.workforce.workers.get_mut(index) else {
        return;
    };
    let amount = worker.carrying;
    let resource = worker.carrying_resource;
    if amount <= 0 {
        worker.carrying = 0;
        worker.carrying_resource = None;
        worker.haul_plan = None;
        worker.status = crate::state::WorkerStatus::Idle;
        return;
    }
    match resource.unwrap_or(ResourceKind::Bones) {
        ResourceKind::Bones => {
            session
                .economy
                .add_loose(ResourceKind::Bones, worker.position, amount);
        }
        ResourceKind::Wood => {
            session
                .economy
                .add_loose(ResourceKind::Wood, worker.position, amount);
        }
    }
    worker.carrying = 0;
    worker.carrying_resource = None;
    worker.haul_plan = None;
    worker.status = crate::state::WorkerStatus::Idle;
    worker.progress = 0.0;
}
