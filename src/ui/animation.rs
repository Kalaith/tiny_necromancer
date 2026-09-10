//! Deterministic procedural motion layered over the tiny sprite atlas.

use crate::engine::movement::{ActorMotion, Facing};
use crate::state::{JobKind, ResourceKind, UndeadKind, Worker, WorkerStatus};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct AnimationClock {
    elapsed: f32,
}

impl AnimationClock {
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    pub fn update(&mut self, dt: f32, paused: bool) {
        if !paused {
            self.elapsed = (self.elapsed + dt.max(0.0)).min(10_000.0);
        }
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ActorVisual {
    pub offset: Vec2,
    pub rotation: f32,
    pub scale: f32,
}

pub fn worker_visual(
    worker: &Worker,
    motion: Option<&ActorMotion>,
    elapsed: f32,
    tile_size: f32,
) -> ActorVisual {
    let walking =
        worker.status == WorkerStatus::Walking || motion.is_some_and(ActorMotion::is_walking);
    let phase = elapsed * std::f32::consts::TAU;
    let walk_phase = phase * 2.7;
    let breathe = (phase * 0.72).sin();
    let step = walk_phase.sin();
    let facing = motion.map_or(Facing::Down, ActorMotion::facing);
    let facing_sign = match facing {
        Facing::Left => -1.0,
        Facing::Right => 1.0,
        Facing::Up | Facing::Down => 0.0,
    };
    let mut offset = vec2(0.0, breathe * tile_size * 0.012);
    let mut rotation = breathe * 0.012;
    if walking {
        offset.y -= step.abs() * tile_size * 0.045;
        offset.x += facing_sign * tile_size * 0.018;
        rotation += step * 0.035;
    }
    match worker.assignment {
        JobKind::Dig => {
            rotation += (phase * 1.8).sin() * 0.035;
        }
        JobKind::Haul if worker.carrying_resource.is_some() => {
            offset.x += (phase * 1.35).sin() * tile_size * 0.026;
            rotation += (phase * 1.35).sin() * 0.04;
        }
        JobKind::Build => {
            offset.x += (phase * 2.1).sin() * tile_size * 0.018;
            rotation += (phase * 2.1).sin() * 0.06;
        }
        JobKind::Refine => {
            offset.y -= (phase * 1.4).sin().max(0.0) * tile_size * 0.018;
        }
        JobKind::Guard => {
            rotation += (phase * 0.65).sin() * 0.045;
        }
        JobKind::Wood | JobKind::Haul => {}
    }
    ActorVisual {
        offset,
        rotation,
        scale: undead_scale(worker.kind) * (1.0 + breathe * 0.012),
    }
}

pub fn necromancer_visual(motion: &ActorMotion, elapsed: f32, tile_size: f32) -> ActorVisual {
    let phase = elapsed * std::f32::consts::TAU;
    let walking = motion.is_walking();
    let sway = (phase * if walking { 2.5 } else { 0.75 }).sin();
    ActorVisual {
        offset: vec2(sway * tile_size * if walking { 0.025 } else { 0.012 }, 0.0),
        rotation: sway * if walking { 0.045 } else { 0.018 },
        scale: 1.0 + sway * 0.01,
    }
}

pub fn draw_worker_feedback(worker: &Worker, center: Vec2, tile_size: f32, elapsed: f32) {
    let active = worker.status != WorkerStatus::Idle || worker.carrying > 0;
    if active {
        let color = job_color(worker.assignment, worker.status);
        let icon_center = center + vec2(tile_size * 0.30, -tile_size * 0.39);
        draw_circle(
            icon_center.x,
            icon_center.y,
            tile_size * 0.105,
            color.with_alpha(0.92),
        );
        draw_text_centered_in_box(
            job_glyph(worker.assignment),
            icon_center.x - tile_size * 0.10,
            icon_center.y - tile_size * 0.10,
            tile_size * 0.20,
            tile_size * 0.20,
            (tile_size * 0.16).clamp(10.0, 18.0),
            color,
        );
    }
    if worker.carrying > 0 {
        let sway = (elapsed * std::f32::consts::TAU * 1.35).sin() * tile_size * 0.025;
        let bundle = center + vec2(sway, tile_size * 0.03);
        let bundle_color = match worker.carrying_resource.unwrap_or(ResourceKind::Bones) {
            ResourceKind::Bones => Color::new(0.82, 0.82, 0.72, 1.0),
            ResourceKind::Wood => Color::new(0.62, 0.38, 0.17, 1.0),
        };
        draw_circle(bundle.x, bundle.y, tile_size * 0.11, bundle_color);
        draw_line(
            bundle.x - tile_size * 0.09,
            bundle.y,
            bundle.x + tile_size * 0.09,
            bundle.y,
            2.0,
            Color::new(0.16, 0.10, 0.08, 0.8),
        );
    }
    if worker.status != WorkerStatus::Working
        && !(worker.assignment == JobKind::Guard && worker.status == WorkerStatus::Hiding)
    {
        return;
    }
    let pulse = (elapsed * std::f32::consts::TAU * 1.6).sin();
    match worker.assignment {
        JobKind::Dig => {
            let hand = center + vec2(tile_size * 0.08, tile_size * 0.08);
            let head = hand + vec2(tile_size * 0.15, -tile_size * (0.22 + pulse.abs() * 0.04));
            draw_line(hand.x, hand.y, head.x, head.y, 3.0, dark::WARNING);
            draw_line(
                head.x - tile_size * 0.05,
                head.y,
                head.x + tile_size * 0.07,
                head.y,
                3.0,
                dark::WARNING,
            );
        }
        JobKind::Wood => {
            let axe = center + vec2(tile_size * 0.12, -tile_size * 0.04);
            draw_line(
                axe.x - tile_size * 0.03,
                axe.y + tile_size * 0.15,
                axe.x + tile_size * 0.12,
                axe.y - tile_size * 0.16,
                3.0,
                Color::new(0.76, 0.57, 0.32, 1.0),
            );
            draw_line(
                axe.x + tile_size * 0.07,
                axe.y - tile_size * 0.14,
                axe.x + tile_size * 0.18,
                axe.y - tile_size * 0.09,
                4.0,
                dark::WARNING,
            );
        }
        JobKind::Build => {
            let hammer = center + vec2(tile_size * 0.12, -tile_size * 0.12);
            draw_line(
                hammer.x - tile_size * 0.08,
                hammer.y + tile_size * 0.14,
                hammer.x + tile_size * 0.10,
                hammer.y - tile_size * 0.15,
                3.0,
                Color::new(0.70, 0.46, 0.25, 1.0),
            );
            draw_line(
                hammer.x + tile_size * 0.03,
                hammer.y - tile_size * 0.18,
                hammer.x + tile_size * 0.16,
                hammer.y - tile_size * 0.12,
                5.0,
                dark::WARNING,
            );
        }
        JobKind::Refine => {
            let radius = tile_size * (0.18 + pulse.abs() * 0.05);
            draw_circle_lines(
                center.x,
                center.y,
                radius,
                2.0,
                dark::ACCENT.with_alpha(0.75),
            );
            draw_circle(
                center.x + tile_size * 0.18,
                center.y - tile_size * 0.18,
                tile_size * 0.028,
                dark::ACCENT,
            );
        }
        JobKind::Haul => {}
        JobKind::Guard => {
            let scan = tile_size * (0.16 + pulse.abs() * 0.03);
            draw_line(
                center.x - scan,
                center.y - tile_size * 0.20,
                center.x + scan,
                center.y - tile_size * 0.20,
                2.0,
                Color::new(0.48, 0.62, 0.95, 0.75),
            );
            draw_circle(
                center.x + scan,
                center.y - tile_size * 0.20,
                tile_size * 0.035,
                Color::new(0.48, 0.62, 0.95, 0.95),
            );
        }
    }
}

pub fn draw_necromancer_feedback(
    center: Vec2,
    tile_size: f32,
    motion: &ActorMotion,
    elapsed: f32,
    has_destination: bool,
) {
    let pulse = (elapsed * std::f32::consts::TAU * 1.2).sin() * 0.5 + 0.5;
    let violet = Color::new(0.72, 0.44, 0.94, 1.0);
    if motion.is_walking() {
        let trail = center - vec2(tile_size * 0.12, tile_size * 0.03);
        draw_line(
            trail.x - tile_size * 0.12,
            trail.y,
            trail.x + tile_size * 0.04,
            trail.y,
            2.0,
            violet.with_alpha(0.22),
        );
        return;
    }
    let radius = tile_size * (0.23 + pulse * 0.08);
    draw_circle_lines(
        center.x,
        center.y,
        radius,
        2.0,
        violet.with_alpha(0.22 + pulse * 0.24),
    );
    draw_line(
        center.x + tile_size * 0.12,
        center.y + tile_size * 0.16,
        center.x + tile_size * 0.21,
        center.y - tile_size * 0.22,
        3.0,
        violet.with_alpha(0.72),
    );
    draw_circle(
        center.x + tile_size * 0.23,
        center.y - tile_size * 0.25,
        tile_size * (0.035 + pulse * 0.018),
        violet,
    );
    if has_destination {
        draw_circle(
            center.x - tile_size * 0.22,
            center.y - tile_size * 0.18,
            tile_size * 0.025,
            violet.with_alpha(0.78),
        );
    }
}

pub fn job_glyph(job: JobKind) -> &'static str {
    match job {
        JobKind::Dig => "D",
        JobKind::Haul => "H",
        JobKind::Guard => "G",
        JobKind::Wood => "W",
        JobKind::Build => "B",
        JobKind::Refine => "R",
    }
}

fn job_color(job: JobKind, status: WorkerStatus) -> Color {
    if status == WorkerStatus::Carrying {
        return dark::POSITIVE;
    }
    match job {
        JobKind::Dig => dark::WARNING,
        JobKind::Haul => dark::POSITIVE,
        JobKind::Guard => Color::new(0.48, 0.62, 0.95, 1.0),
        JobKind::Wood => Color::new(0.84, 0.62, 0.31, 1.0),
        JobKind::Build => Color::new(0.92, 0.70, 0.34, 1.0),
        JobKind::Refine => dark::ACCENT,
    }
}

pub fn undead_scale(kind: UndeadKind) -> f32 {
    if kind == UndeadKind::BruteSkeleton {
        1.08
    } else {
        1.0
    }
}
