//! Touch-first controls for the shared worker priority list.

use super::components::virtual_button;
use super::{UiAction, UiContext};
use crate::state::Technology;
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_orders_panel(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let rect = Rect::new(238.0, 106.0, 680.0, 490.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.045, 0.055, 0.055, 0.98))
            .with_border(1.0, Color::new(0.58, 0.70, 0.62, 0.75)),
    );
    draw_text_block(
        "WORKER ORDERS",
        rect.x + 24.0,
        rect.y + 20.0,
        240.0,
        22.0,
        18.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    if ctx
        .session
        .research
        .is_unlocked(Technology::DomainStewardship)
    {
        draw_text_block(
            &format!("DOMAIN POLICY · {}", ctx.session.stewardship_policy.label()),
            rect.x + 424.0,
            rect.y + 24.0,
            232.0,
            18.0,
            11.0,
            0.0,
            dark::ACCENT,
        );
    }
    if !ctx
        .session
        .research
        .is_unlocked(Technology::BindingRoutines)
    {
        draw_text_block(
            "Restore the work shed, then study Binding Routines to tune repeat priorities.",
            rect.x + 24.0,
            rect.y + 64.0,
            rect.w - 48.0,
            52.0,
            16.0,
            4.0,
            dark::TEXT,
        );
        return;
    }
    draw_text_block(
        "Automated workers choose the first available duty from top to bottom.",
        rect.x + 24.0,
        rect.y + 50.0,
        rect.w - 48.0,
        20.0,
        14.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_text_block(
        "PRIORITY",
        rect.x + 24.0,
        rect.y + 86.0,
        90.0,
        16.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_text_block(
        "ACTIVE",
        rect.x + 454.0,
        rect.y + 86.0,
        70.0,
        16.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_text_block(
        "REORDER",
        rect.x + 530.0,
        rect.y + 86.0,
        110.0,
        16.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    draw_order_rows(ctx, pointer, actions, rect);
    draw_order_footer(rect);
}

fn draw_order_footer(rect: Rect) {
    draw_text_block(
        "Changes apply to every worker using Repeat priorities and are saved with the cemetery.",
        rect.x + 24.0,
        rect.bottom() - 30.0,
        rect.w - 48.0,
        18.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn draw_order_rows(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>, rect: Rect) {
    for (index, job) in ctx.session.workforce.priorities.iter().copied().enumerate() {
        let y = rect.y + 106.0 + index as f32 * 58.0;
        draw_surface(
            Rect::new(rect.x + 24.0, y, rect.w - 48.0, 50.0),
            &SurfaceStyle::new(Color::new(0.075, 0.085, 0.085, 1.0))
                .with_border(1.0, Color::new(0.40, 0.47, 0.43, 0.45)),
        );
        draw_text_block(
            &format!("{}.", index + 1),
            rect.x + 38.0,
            y + 16.0,
            26.0,
            18.0,
            14.0,
            0.0,
            dark::ACCENT,
        );
        draw_text_block(
            job.label(),
            rect.x + 68.0,
            y + 9.0,
            130.0,
            20.0,
            15.0,
            0.0,
            dark::TEXT_BRIGHT,
        );
        let description = ctx
            .data
            .jobs
            .get(job.id())
            .map_or("Available duty", |definition| {
                definition.description.as_str()
            });
        draw_text_block(
            description,
            rect.x + 208.0,
            y + 9.0,
            232.0,
            34.0,
            12.0,
            3.0,
            dark::TEXT_DIM,
        );
        let active = ctx
            .session
            .workforce
            .workers
            .iter()
            .filter(|worker| worker.priority_mode && worker.assignment == job)
            .count();
        draw_text_block(
            &format!("{active} auto"),
            rect.x + 454.0,
            y + 17.0,
            66.0,
            18.0,
            12.0,
            0.0,
            if active > 0 {
                dark::POSITIVE
            } else {
                dark::TEXT_DIM
            },
        );
        draw_priority_button(
            pointer,
            actions,
            Rect::new(rect.x + 526.0, y + 3.0, 64.0, 44.0),
            job,
            -1,
            "Up",
            index > 0 && ctx.session.phase == crate::state::GamePhase::Playing,
        );
        draw_priority_button(
            pointer,
            actions,
            Rect::new(rect.x + 596.0, y + 3.0, 64.0, 44.0),
            job,
            1,
            "Down",
            index + 1 < ctx.session.workforce.priorities.len()
                && ctx.session.phase == crate::state::GamePhase::Playing,
        );
    }
}

fn draw_priority_button(
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    rect: Rect,
    job: crate::state::JobKind,
    direction: i32,
    label: &str,
    enabled: bool,
) {
    if virtual_button(rect, label, enabled, ButtonTone::Secondary, pointer) {
        actions.push(UiAction::MovePriority(job, direction));
    }
}
