//! Status strip and contextual inspector.

use super::buildings;
use super::components::{compact_virtual_button, pause_control_rect};
use super::components::{stage_label, status_label, virtual_button};
use super::production::{draw_kiln_inspector, is_kiln};
use super::{Panel, UiAction, UiContext};
use crate::state::{
    BuildingKind, GamePhase, JobKind, PlotStatus, Selection, Technology, UndeadKind, WorkerStatus,
};
use macroquad::prelude::*;
use macroquad_toolkit::prelude::*;
use macroquad_toolkit::ui::Pointer;

pub(super) fn draw_status_strip(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let e = &ctx.session.economy;
    let storage_capacity =
        crate::engine::districts::storage_capacity(ctx.session, &ctx.data.config.district_rules);
    let storage_space =
        crate::engine::districts::storage_space(ctx.session, &ctx.data.config.district_rules);
    let storage_accent = if storage_space == 0 {
        dark::NEGATIVE
    } else if storage_space * 4 <= storage_capacity {
        dark::WARNING
    } else {
        Color::new(0.60, 0.86, 0.70, 1.0)
    };
    let cards = [
        (
            "BONES",
            e.bones.to_string(),
            Color::new(0.24, 0.25, 0.24, 0.94),
            dark::TEXT_BRIGHT,
        ),
        (
            "MANA",
            e.mana.to_string(),
            Color::new(0.18, 0.20, 0.34, 0.94),
            Color::new(0.73, 0.79, 1.0, 1.0),
        ),
        (
            "WOOD",
            e.wood.to_string(),
            Color::new(0.28, 0.20, 0.13, 0.94),
            Color::new(0.90, 0.74, 0.52, 1.0),
        ),
        (
            "UNDEAD",
            ctx.session.active_undead().to_string(),
            Color::new(0.12, 0.22, 0.18, 0.94),
            Color::new(0.69, 0.91, 0.78, 1.0),
        ),
        (
            "WARDS",
            e.ward_charges.to_string(),
            Color::new(0.24, 0.17, 0.31, 0.94),
            Color::new(0.86, 0.68, 1.0, 1.0),
        ),
        (
            "STORAGE",
            format!("{}/{}", e.stored_materials(), storage_capacity),
            Color::new(0.16, 0.20, 0.18, 0.94),
            storage_accent,
        ),
    ];
    for (index, (label, value, fill, accent)) in cards.into_iter().enumerate() {
        let rect = Rect::new(20.0 + index as f32 * 112.0, 18.0, 102.0, 46.0);
        draw_surface(
            rect,
            &SurfaceStyle::new(fill).with_border(1.0, accent.with_alpha(0.56)),
        );
        draw_text_block(
            label,
            rect.x + 10.0,
            rect.y + 7.0,
            rect.w - 20.0,
            14.0,
            11.0,
            0.0,
            accent,
        );
        draw_text_block(
            &value,
            rect.x + 10.0,
            rect.y + 22.0,
            rect.w - 20.0,
            20.0,
            20.0,
            0.0,
            dark::TEXT_BRIGHT,
        );
    }
    draw_surface(
        Rect::new(800.0, 18.0, 270.0, 46.0),
        &SurfaceStyle::new(Color::new(0.09, 0.11, 0.10, 0.92))
            .with_border(1.0, Color::new(0.50, 0.70, 0.56, 0.52)),
    );
    let suspicion_color =
        if ctx.session.pressure.suspicion >= ctx.data.config.suspicion_thresholds[2] {
            dark::NEGATIVE
        } else if ctx.session.pressure.suspicion >= ctx.data.config.suspicion_thresholds[1] {
            dark::WARNING
        } else {
            dark::POSITIVE
        };
    draw_text_block(
        "SUSPICION",
        814.0,
        25.0,
        100.0,
        14.0,
        11.0,
        0.0,
        suspicion_color,
    );
    draw_text_block(
        &format!(
            "{} · {:.0}%",
            stage_label(ctx.session.pressure.stage),
            ctx.session.pressure.suspicion
        ),
        814.0,
        40.0,
        230.0,
        18.0,
        15.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    progress_bar(
        975.0,
        33.0,
        82.0,
        9.0,
        ctx.session.pressure.suspicion,
        100.0,
        suspicion_color,
    );
    draw_surface(
        Rect::new(1082.0, 18.0, 178.0, 46.0),
        &SurfaceStyle::new(Color::new(0.075, 0.075, 0.08, 0.94))
            .with_border(1.0, Color::new(0.65, 0.65, 0.66, 0.42)),
    );
    draw_text_block(
        &format!(
            "TIME  {:02}:{:02}",
            (ctx.session.progress.elapsed_seconds as u32) / 60,
            ctx.session.progress.elapsed_seconds as u32 % 60
        ),
        1094.0,
        26.0,
        72.0,
        18.0,
        14.0,
        0.0,
        dark::TEXT,
    );
    if compact_virtual_button(
        pause_control_rect(),
        if ctx.session.phase == GamePhase::Paused {
            "Resume"
        } else {
            "Pause"
        },
        true,
        ButtonTone::Secondary,
        16.0,
        pointer,
    ) {
        actions.push(UiAction::TogglePause);
    }
}

pub(super) fn draw_inspector(ctx: &UiContext<'_>, pointer: Pointer, actions: &mut Vec<UiAction>) {
    let Some(selection) = ctx.session.world.selected else {
        return;
    };
    let panel = Rect::new(952.0, 86.0, 310.0, 454.0);
    draw_surface(
        panel,
        &SurfaceStyle::new(Color::new(0.055, 0.065, 0.065, 0.96))
            .with_border(1.0, Color::new(0.58, 0.70, 0.62, 0.60)),
    );
    draw_text_block(
        "INSPECTOR",
        panel.x + 18.0,
        panel.y + 16.0,
        130.0,
        18.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
    match selection {
        Selection::Grave(index) => draw_grave_inspector(ctx, pointer, actions, panel, index),
        Selection::Worker(index) => draw_worker_inspector(ctx, pointer, actions, panel, index),
        Selection::Building(index) => draw_building_inspector(ctx, pointer, actions, panel, index),
        Selection::Ground(tile) => {
            draw_text_block(
                "Ground",
                panel.x + 18.0,
                panel.y + 54.0,
                panel.w - 36.0,
                28.0,
                24.0,
                0.0,
                dark::TEXT_BRIGHT,
            );
            draw_text_block(
                &format!("Clearing tile {}, {}", tile.x + 1, tile.y + 1),
                panel.x + 18.0,
                panel.y + 94.0,
                panel.w - 36.0,
                22.0,
                15.0,
                0.0,
                dark::TEXT,
            );
            if let Some(summary) = crate::engine::districts::tile_summary(
                ctx.session,
                &ctx.data.config.district_rules,
                tile,
            ) {
                let summary_height = if summary.contains('\n') { 96.0 } else { 64.0 };
                draw_text_block(
                    &summary,
                    panel.x + 18.0,
                    panel.y + 132.0,
                    panel.w - 36.0,
                    summary_height,
                    14.0,
                    4.0,
                    dark::ACCENT,
                );
                let button = Rect::new(
                    panel.x + 18.0,
                    panel.y + 132.0 + summary_height + 8.0,
                    panel.w - 36.0,
                    44.0,
                );
                let (button_label, destination) = if ctx
                    .session
                    .research
                    .is_unlocked(Technology::DomainStewardship)
                {
                    ("Open Domain rules", Panel::Domain)
                } else {
                    ("Open Research", Panel::Research)
                };
                if virtual_button(button, button_label, true, ButtonTone::Secondary, pointer) {
                    actions.push(UiAction::TogglePanel(destination));
                }
            } else {
                draw_text_block(
                    "Select an actor or structure for contextual orders.",
                    panel.x + 18.0,
                    panel.y + 132.0,
                    panel.w - 36.0,
                    48.0,
                    14.0,
                    4.0,
                    dark::TEXT_DIM,
                );
            }
        }
        Selection::Necromancer => {
            draw_text_block(
                "Necromancer",
                panel.x + 18.0,
                panel.y + 54.0,
                panel.w - 36.0,
                28.0,
                24.0,
                0.0,
                dark::TEXT_BRIGHT,
            );
            draw_text_block(
                "Ritualist · present",
                panel.x + 18.0,
                panel.y + 94.0,
                panel.w - 36.0,
                22.0,
                15.0,
                0.0,
                dark::ACCENT,
            );
            draw_text_block(
                "Tap the clearing to move. The staff marks your active ritual focus.",
                panel.x + 18.0,
                panel.y + 132.0,
                panel.w - 36.0,
                54.0,
                14.0,
                4.0,
                dark::TEXT_DIM,
            );
            draw_text_block(
                &ctx.session.world.necromancer_destination.map_or_else(
                    || "Destination · holding ritual focus".to_owned(),
                    |tile| format!("Destination · tile {}, {}", tile.x + 1, tile.y + 1),
                ),
                panel.x + 18.0,
                panel.y + 202.0,
                panel.w - 36.0,
                20.0,
                13.0,
                0.0,
                dark::ACCENT,
            );
            if ctx.session.world.necromancer_destination.is_some()
                && virtual_button(
                    Rect::new(panel.x + 18.0, panel.y + 236.0, panel.w - 36.0, 44.0),
                    "Cancel movement",
                    ctx.session.phase == GamePhase::Playing,
                    ButtonTone::Secondary,
                    pointer,
                )
            {
                actions.push(UiAction::MoveNecromancer(
                    ctx.session.world.necromancer_position,
                ));
            }
        }
    }
}

fn draw_grave_inspector(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    index: usize,
) {
    let Some(plot) = ctx.session.world.plots.get(index) else {
        return;
    };
    draw_text_block(
        &format!("Grave {:02}", index + 1),
        panel.x + 18.0,
        panel.y + 54.0,
        panel.w - 36.0,
        28.0,
        24.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    let state = match plot.status {
        PlotStatus::Ready => "Undisturbed",
        PlotStatus::Digging => "Excavation underway",
        PlotStatus::Dug => "Open excavation",
        PlotStatus::Locked => "Outside the clearing",
    };
    draw_text_block(
        state,
        panel.x + 18.0,
        panel.y + 94.0,
        panel.w - 36.0,
        22.0,
        15.0,
        0.0,
        if plot.status == PlotStatus::Digging {
            dark::WARNING
        } else {
            dark::TEXT
        },
    );
    if plot.status == PlotStatus::Digging {
        progress_bar(
            panel.x + 18.0,
            panel.y + 128.0,
            panel.w - 36.0,
            12.0,
            plot.progress,
            ctx.data.jobs.get("dig").map_or(8.0, |job| job.work_seconds),
            dark::WARNING,
        );
    }
    draw_text_block(
        if plot.status == PlotStatus::Dug {
            "The earth is yielding bones and the chance of a corpse remnant."
        } else {
            "A worker can be assigned here when the shovel is free."
        },
        panel.x + 18.0,
        panel.y + 158.0,
        panel.w - 36.0,
        54.0,
        14.0,
        4.0,
        dark::TEXT_DIM,
    );
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 244.0, panel.w - 36.0, 44.0),
        "Assign selected worker · Dig",
        plot.status == PlotStatus::Ready && ctx.session.phase == GamePhase::Playing,
        ButtonTone::Primary,
        pointer,
    ) {
        actions.push(UiAction::AssignJob(JobKind::Dig));
    }
    let skeleton_def = ctx
        .data
        .undead
        .get(UndeadKind::Skeleton.id())
        .expect("validated skeleton recipe");
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 300.0, panel.w - 36.0, 44.0),
        &format!(
            "Raise skeleton · B{} M{}",
            skeleton_def.bones_cost, skeleton_def.mana_cost
        ),
        plot.status == PlotStatus::Dug
            && ctx.session.economy.bones >= skeleton_def.bones_cost
            && ctx.session.economy.mana >= skeleton_def.mana_cost,
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::Raise(UndeadKind::Skeleton));
    }
}

fn draw_worker_inspector(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    index: usize,
) {
    let Some(worker) = ctx.session.workforce.workers.get(index) else {
        return;
    };
    draw_text_block(
        &worker.name,
        panel.x + 18.0,
        panel.y + 54.0,
        panel.w - 36.0,
        28.0,
        24.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        &format!(
            "{} · {}",
            if worker.kind == UndeadKind::BruteSkeleton {
                "Brute skeleton"
            } else {
                "Skeleton worker"
            },
            status_label(worker.status)
        ),
        panel.x + 18.0,
        panel.y + 94.0,
        panel.w - 36.0,
        22.0,
        15.0,
        0.0,
        dark::ACCENT,
    );
    let district_hint = super::world_feedback::worker_district_hint(ctx, worker);
    let priority_route_hint = super::world_feedback::worker_priority_route_hint(ctx, index);
    let route_gap_hint = crate::engine::districts::route_gap_district(ctx.session, index)
        .map_or_else(String::new, |kind| format!(" · {} route gap", kind.label()));
    let job_detail = format!(
        "Current job: {}{}{}{}{}",
        worker.assignment.label(),
        if worker.priority_mode {
            " · priority mode"
        } else {
            ""
        },
        district_hint.map_or_else(String::new, |hint| format!(" · {hint}")),
        route_gap_hint,
        priority_route_hint.map_or_else(String::new, |hint| format!(" · {hint}"))
    );
    draw_text_block(
        &job_detail,
        panel.x + 18.0,
        panel.y + 132.0,
        panel.w - 36.0,
        22.0,
        14.0,
        0.0,
        dark::TEXT,
    );
    let activity = if worker.status == WorkerStatus::Idle {
        super::world_feedback::worker_idle_reason(ctx, worker).to_owned()
    } else {
        super::world_feedback::worker_activity_detail(worker)
    };
    draw_text_block(
        &activity,
        panel.x + 18.0,
        panel.y + 164.0,
        panel.w - 36.0,
        44.0,
        14.0,
        4.0,
        if worker.status == WorkerStatus::Idle {
            dark::WARNING
        } else {
            dark::TEXT_DIM
        },
    );
    let destination = super::world_feedback::worker_destination_label(ctx, worker);
    let route_summary = super::world_feedback::worker_route_summary(ctx, worker);
    let destination_detail = route_summary.map_or(destination.clone(), |summary| {
        format!("{destination} · {summary}")
    });
    draw_text_block(
        &format!("Destination · {destination_detail}"),
        panel.x + 18.0,
        panel.y + 208.0,
        panel.w - 36.0,
        18.0,
        12.0,
        0.0,
        dark::TEXT_DIM,
    );
    for (idx, job) in [
        JobKind::Dig,
        JobKind::Haul,
        JobKind::Guard,
        JobKind::Wood,
        JobKind::Build,
        JobKind::Refine,
    ]
    .into_iter()
    .enumerate()
    {
        let button = Rect::new(
            panel.x + 18.0 + (idx % 3) as f32 * 91.0,
            panel.y + 232.0 + (idx / 3) as f32 * 50.0,
            84.0,
            44.0,
        );
        if virtual_button(
            button,
            job.label(),
            ctx.session.phase == GamePhase::Playing
                && (job != JobKind::Refine || ctx.session.has_building(BuildingKind::OssuaryKiln)),
            if job == JobKind::Guard {
                ButtonTone::Secondary
            } else {
                ButtonTone::Primary
            },
            pointer,
        ) {
            actions.push(UiAction::AssignJob(job));
        }
    }
    if virtual_button(
        Rect::new(panel.x + 18.0, panel.y + 342.0, panel.w - 36.0, 44.0),
        if worker.priority_mode {
            "Direct orders"
        } else {
            "Repeat priorities"
        },
        ctx.session
            .research
            .is_unlocked(Technology::BindingRoutines),
        ButtonTone::Positive,
        pointer,
    ) {
        actions.push(UiAction::ToggleAutomation);
    }
    if !ctx
        .session
        .research
        .is_unlocked(Technology::BindingRoutines)
    {
        draw_text_block(
            "Restore the shed, then study Binding Routines to repeat this order.",
            panel.x + 18.0,
            panel.y + 402.0,
            panel.w - 36.0,
            36.0,
            12.0,
            4.0,
            dark::TEXT_DIM,
        );
    }
}

fn draw_building_inspector(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
    panel: Rect,
    index: usize,
) {
    let Some(building) = ctx.session.world.buildings.get(index) else {
        return;
    };
    let name = ctx
        .data
        .buildings
        .get(building.kind.id())
        .map_or(building.kind.id(), |def| def.name.as_str());
    draw_text_block(
        name,
        panel.x + 18.0,
        panel.y + 54.0,
        panel.w - 36.0,
        28.0,
        23.0,
        0.0,
        dark::TEXT_BRIGHT,
    );
    draw_text_block(
        &buildings::status_label(ctx, building),
        panel.x + 18.0,
        panel.y + 94.0,
        panel.w - 36.0,
        22.0,
        15.0,
        0.0,
        if building.complete {
            dark::POSITIVE
        } else {
            dark::WARNING
        },
    );
    if !building.complete {
        progress_bar(
            panel.x + 18.0,
            panel.y + 128.0,
            panel.w - 36.0,
            12.0,
            building.progress,
            ctx.data
                .buildings
                .get(building.kind.id())
                .map_or(10.0, |def| def.build_seconds),
            dark::WARNING,
        );
    }
    if building.complete && building.kind == BuildingKind::OssuaryKiln {
        if let Some(order) = ctx
            .session
            .progress
            .production
            .as_ref()
            .filter(|order| order.building == building.kind)
        {
            let seconds = ctx
                .data
                .buildings
                .get(building.kind.id())
                .and_then(|def| def.production.as_ref())
                .map_or(8.0, |recipe| recipe.seconds);
            draw_text_block(
                &format!(
                    "Refining ward charge · reserved {}/{}",
                    ctx.session.progress.production_queue,
                    crate::engine::progression::MAX_PRODUCTION_QUEUE
                ),
                panel.x + 18.0,
                panel.y + 126.0,
                panel.w - 36.0,
                18.0,
                13.0,
                0.0,
                dark::ACCENT,
            );
            progress_bar(
                panel.x + 18.0,
                panel.y + 148.0,
                panel.w - 36.0,
                10.0,
                order.progress,
                seconds,
                dark::ACCENT,
            );
        }
    }
    draw_text_block(
        &format!(
            "Footprint · {}, {}",
            building.position.x + 1,
            building.position.y + 1
        ),
        panel.x + 18.0,
        panel.y + 160.0,
        panel.w - 172.0,
        22.0,
        14.0,
        0.0,
        dark::TEXT_DIM,
    );
    buildings::draw_desktop_upgrade_preview(ctx, panel, building);
    buildings::draw_desktop_upgrade(ctx, pointer, actions, panel, building);
    buildings::draw_desktop_market_button(ctx, pointer, actions, panel, building);
    if building.kind == BuildingKind::WorkShed
        && building.complete
        && virtual_button(
            Rect::new(panel.x + 18.0, panel.y + 232.0, panel.w - 36.0, 44.0),
            if ctx.session.research.current == Some(Technology::BindingRoutines) {
                "Bindings in progress"
            } else {
                "Study bindings"
            },
            ctx.session.research.can_start(Technology::BindingRoutines),
            ButtonTone::Positive,
            pointer,
        )
    {
        actions.push(UiAction::StartResearch(Technology::BindingRoutines));
    }
    if building.kind == BuildingKind::WorkShed && building.complete {
        draw_text_block(
            "A restored shed is the first research station.",
            panel.x + 18.0,
            panel.y + 296.0,
            panel.w - 36.0,
            42.0,
            14.0,
            4.0,
            dark::TEXT_DIM,
        );
    }
    if is_kiln(building) {
        draw_kiln_inspector(ctx, pointer, actions, panel, building);
    }
}

pub(super) fn draw_command_dock(
    ctx: &UiContext<'_>,
    pointer: Pointer,
    actions: &mut Vec<UiAction>,
) {
    let rect = Rect::new(298.0, 618.0, 684.0, 86.0);
    draw_surface(
        rect,
        &SurfaceStyle::new(Color::new(0.045, 0.055, 0.055, 0.96))
            .with_border(1.0, Color::new(0.56, 0.67, 0.60, 0.68)),
    );
    draw_text_block(
        "COMMANDS",
        rect.x + 16.0,
        rect.y + 10.0,
        90.0,
        16.0,
        11.0,
        0.0,
        dark::TEXT_DIM,
    );
    let labels = [
        (Panel::Build, "Build"),
        (Panel::Orders, "Orders"),
        (Panel::Undead, "Undead"),
        (Panel::Research, "Research"),
        (Panel::Zones, "Zones"),
        (Panel::Domain, "Domain"),
    ];
    for (index, (panel, label)) in labels.into_iter().enumerate() {
        let button = Rect::new(
            rect.x + 106.0 + index as f32 * 96.0,
            rect.y + 28.0,
            92.0,
            44.0,
        );
        if virtual_button(
            button,
            label,
            match panel {
                Panel::Zones => ctx.session.research.is_unlocked(Technology::Gravecraft),
                Panel::Domain => ctx
                    .session
                    .research
                    .is_unlocked(Technology::DomainStewardship),
                _ => true,
            },
            if ctx.panel == panel {
                ButtonTone::Positive
            } else {
                ButtonTone::Secondary
            },
            pointer,
        ) {
            actions.push(UiAction::TogglePanel(panel));
        }
    }
}
