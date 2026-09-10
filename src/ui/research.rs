//! Research and work-area panels.

use super::components::virtual_button;
use super::{UiAction, UiContext};
use crate::state::{Technology, ZoneKind};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_research_panel(
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
        "RESEARCH",
        rect.x + 24.0,
        rect.y + 20.0,
        160.0,
        22.0,
        18.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        "Practical discoveries keep earlier commands intact.",
        rect.x + 24.0,
        rect.y + 50.0,
        420.0,
        20.0,
        14.0,
        0.0,
        dark::TEXT_DIM,
    );
    if let Some(current) = ctx.session.research.current {
        draw_text_block(
            &format!(
                "Studying {} · {:.0}%",
                current.label(),
                ctx.session.research.progress / current.duration(&ctx.data.config) * 100.0
            ),
            rect.x + 24.0,
            rect.y + 86.0,
            560.0,
            22.0,
            15.0,
            0.0,
            dark::ACCENT,
        );
        progress_bar(
            rect.x + 24.0,
            rect.y + 118.0,
            600.0,
            10.0,
            ctx.session.research.progress,
            current.duration(&ctx.data.config),
            dark::ACCENT,
        );
    }
    let techs = [
        Technology::BindingRoutines,
        Technology::Gravecraft,
        Technology::OssuaryLogistics,
        Technology::DomainStewardship,
    ];
    for (index, technology) in techs.into_iter().enumerate() {
        let y = rect.y + 154.0 + index as f32 * 70.0;
        let complete = ctx.session.research.is_unlocked(technology);
        let reachable = ctx.session.research.can_start(technology);
        draw_surface(
            Rect::new(rect.x + 24.0, y, 600.0, 56.0),
            &SurfaceStyle::new(if complete {
                Color::new(0.11, 0.22, 0.17, 1.0)
            } else {
                Color::new(0.075, 0.085, 0.085, 1.0)
            })
            .with_border(
                1.0,
                if complete {
                    dark::POSITIVE
                } else {
                    Color::new(0.40, 0.47, 0.43, 0.55)
                },
            ),
        );
        draw_text_block(
            technology.label(),
            rect.x + 40.0,
            y + 10.0,
            210.0,
            20.0,
            16.0,
            0.0,
            dark::TEXT_BRIGHT,
        );
        draw_text_block(
            technology.description(),
            rect.x + 40.0,
            y + 32.0,
            390.0,
            18.0,
            12.0,
            0.0,
            dark::TEXT_DIM,
        );
        let in_progress = ctx.session.research.current == Some(technology);
        if complete {
            draw_text_block(
                "DISCOVERED",
                rect.x + 492.0,
                y + 19.0,
                110.0,
                18.0,
                11.0,
                0.0,
                dark::POSITIVE,
            );
        } else if in_progress {
            draw_text_block(
                "STUDYING",
                rect.x + 496.0,
                y + 19.0,
                104.0,
                18.0,
                11.0,
                0.0,
                dark::ACCENT,
            );
        } else if virtual_button(
            Rect::new(rect.x + 478.0, y + 6.0, 124.0, 44.0),
            if reachable { "Study" } else { "Locked" },
            reachable,
            ButtonTone::Secondary,
            pointer,
        ) {
            actions.push(UiAction::StartResearch(technology));
        }
    }
}
pub(super) fn draw_zones_panel(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let rect = Rect::new(350.0, 460.0, 580.0, 146.0);
    if !ctx.session.research.is_unlocked(Technology::Gravecraft) {
        draw_small_info_panel(
            "WORK AREAS",
            "Study Gravecraft after Binding Routines to paint work areas across the clearing.",
        );
        return;
    }
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.055, 0.065, 0.065, 0.97))
            .with_border(1.0, Color::new(0.56, 0.67, 0.60, 0.70)),
    );
    draw_text_block(
        "WORK AREAS · tap a tool, then paint or clear tiles",
        rect.x + 18.0,
        rect.y + 14.0,
        440.0,
        16.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
    for (index, kind) in [ZoneKind::Work, ZoneKind::Storage, ZoneKind::Patrol]
        .into_iter()
        .enumerate()
    {
        let button = Rect::new(
            rect.x + 18.0 + index as f32 * 124.0,
            rect.y + 42.0,
            112.0,
            44.0,
        );
        if virtual_button(
            button,
            kind.label(),
            ctx.zone_mode != Some(kind),
            if ctx.zone_mode == Some(kind) {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            pointer,
        ) {
            actions.push(UiAction::ToggleZone(kind));
        }
    }
    let message = if ctx
        .session
        .research
        .is_unlocked(Technology::DomainStewardship)
    {
        format!(
            "District 01 · zones W{} S{} P{} · rules apply on marked tiles.",
            zone_count(ctx, ZoneKind::Work),
            zone_count(ctx, ZoneKind::Storage),
            zone_count(ctx, ZoneKind::Patrol),
        )
    } else if ctx
        .session
        .research
        .is_unlocked(Technology::OssuaryLogistics)
    {
        "Storage and Patrol use nearby marked posts; Work favors nearby gathering.".to_owned()
    } else {
        "Work and patrol markings are visible; Logistics will connect storage and production."
            .to_owned()
    };
    draw_text_block(
        &message,
        rect.x + 18.0,
        rect.y + 104.0,
        rect.w - 36.0,
        22.0,
        13.0,
        0.0,
        dark::TEXT_DIM,
    );
}

fn zone_count(ctx: &UiContext<'_>, kind: ZoneKind) -> usize {
    ctx.session
        .world
        .zones
        .iter()
        .filter(|zone| zone.kind == kind)
        .map(|zone| zone.tiles.len())
        .sum()
}

pub(super) fn draw_small_info_panel(title: &str, message: &str) {
    draw_surface(
        Rect::new(350.0, 500.0, 580.0, 100.0),
        &SurfaceStyle::new(Color::new(0.055, 0.065, 0.065, 0.97))
            .with_border(1.0, Color::new(0.56, 0.67, 0.60, 0.70)),
    );
    draw_text_block(title, 370.0, 518.0, 200.0, 18.0, 12.0, 0.0, dark::TEXT_DIM);
    draw_text_block(message, 370.0, 548.0, 540.0, 36.0, 14.0, 4.0, dark::TEXT);
}
