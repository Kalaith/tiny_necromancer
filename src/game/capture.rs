//! Deterministic presentation scenes used by the verification harness.

mod actors;
mod kiln;
mod scenes;

use super::Game;
use crate::data::SuspicionStage;
use crate::state::{BuildingKind, GamePhase, GameSession};
use crate::ui::{self, DomainOverlays, Panel};
use macroquad::prelude::*;
use macroquad_toolkit::camera::CameraTransform;

impl Game {
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.session = GameSession::new(&self.data.config);
        if scene != "menu" {
            self.session.begin();
        }
        self.notifications.clear();
        self.events.drain().for_each(drop);
        self.tick_accumulator = 0.0;
        self.motions.reset(&self.session);
        self.animation.reset();
        self.camera = CameraTransform::new(Vec2::ZERO, 1.0).expect("valid initial camera");
        self.camera_drag = None;
        self.panel = Panel::None;
        self.placement = None;
        self.zone_mode = None;
        self.domain_overlays = DomainOverlays::default();
        match scene {
            "menu" | "gameplay" | "scrolled" => {}
            "research" => self.prepare_capture_research(),
            "orders" => self.prepare_capture_orders(),
            "priority-route" => self.prepare_capture_priority_route(),
            "route-policy" => self.prepare_capture_route_policy(),
            "route-policy-wait" => self.prepare_capture_route_policy_wait(),
            "colony" => self.prepare_capture_colony(),
            "market" => self.prepare_capture_market(),
            "market-contract" => self.prepare_capture_market_contract(),
            "market-locked" => self.prepare_capture_market_locked(),
            "domain" => self.prepare_capture_domain(),
            "harvest-domain" => self.prepare_capture_harvest_domain(),
            "storage-domain" => self.prepare_capture_storage_domain(),
            "storage-slots" => self.prepare_capture_storage_slots(),
            "storage-full" => self.prepare_capture_storage_full(),
            "wood-slots" => self.prepare_capture_wood_slots(),
            "patrol-gap" => self.prepare_capture_patrol_gap(),
            "work-gap" => self.prepare_capture_work_gap(),
            "work-overlap" => self.prepare_capture_work_overlap(),
            "work-locked" => self.prepare_capture_work_locked(),
            "work-overlap-locked" => self.prepare_capture_work_overlap_locked(),
            "work-domain" => self.prepare_capture_work_domain(),
            "route-blocked" => self.prepare_capture_route_blocked(),
            "route-domain" => self.prepare_capture_route_domain(),
            "district-route-gap" => self.prepare_capture_district_route_gap(),
            "notes" => self.prepare_capture_notes(),
            "production" => self.prepare_capture_production(),
            "kiln-hush" => self.prepare_capture_kiln_hush(),
            "kiln-supply" => self.prepare_capture_kiln_supply(),
            "kiln-supply-loose" => self.prepare_capture_kiln_supply_loose(),
            "kiln-supply-inspector" => self.prepare_capture_kiln_supply_inspector(),
            "placement" => {
                self.panel = Panel::Build;
                self.placement = Some(BuildingKind::WorkShed);
            }
            "paused" => self.session.phase = GamePhase::Paused,
            "event" => {
                self.session.pressure.suspicion = self.data.config.suspicion_thresholds[0];
                self.session.pressure.stage = SuspicionStage::Rumour;
                self.session.pressure.active_event = Some("rumour".to_owned());
            }
            "worker-walking" => self.prepare_capture_worker_walking(),
            "worker-haul-planned" => self.prepare_capture_worker_haul_planned(),
            "multi-source" => self.prepare_capture_multi_source(),
            "worker-carrying" => self.prepare_capture_worker_carrying(),
            "worker-working" => self.prepare_capture_worker_working(),
            "necromancer-walking" => self.prepare_capture_necromancer_walking(),
            "necromancer-ritual" => self.prepare_capture_necromancer_ritual(),
            "building" => self.prepare_capture_building(),
            "kiln" => self.prepare_capture_kiln(),
            "victory" => self.prepare_capture_victory(),
            "zoomed" => {
                self.camera.zoom_at(
                    ui::world_grid_rect(),
                    ui::world_grid_rect().center(),
                    1.25,
                    (0.75, 1.5),
                );
                self.camera.pan_screen(vec2(20.0, -12.0));
            }
            other => panic!("Unknown Tiny Necromancer capture scene: {other}"),
        }
        self.motions.reset(&self.session);
    }
}
