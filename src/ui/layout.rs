//! Viewport geometry shared by desktop and compact touch layouts.

use super::{Panel, LOGICAL_HEIGHT, LOGICAL_WIDTH};
use macroquad::prelude::{screen_height, screen_width, Rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiLayout {
    pub logical_width: f32,
    pub logical_height: f32,
    pub compact: bool,
    pub world_rect: Rect,
    pub sheet_rect: Rect,
}

impl UiLayout {
    pub fn current(panel: Panel) -> Self {
        Self::for_dimensions(screen_width(), screen_height(), panel)
    }

    pub fn for_dimensions(width: f32, height: f32, panel: Panel) -> Self {
        let compact = width < 1180.0 || height < 690.0;
        if !compact {
            return Self {
                logical_width: LOGICAL_WIDTH,
                logical_height: LOGICAL_HEIGHT,
                compact: false,
                world_rect: Rect::new(0.0, 0.0, LOGICAL_WIDTH, LOGICAL_HEIGHT),
                sheet_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            };
        }
        let management = matches!(
            panel,
            Panel::Orders | Panel::Research | Panel::Domain | Panel::Feed
        );
        let preferred_sheet_height = if management {
            height * 0.62
        } else {
            height * 0.46
        };
        let sheet_height = preferred_sheet_height
            .clamp(272.0, 390.0)
            .min(height - 128.0);
        let sheet_y = (height - sheet_height).max(128.0);
        Self {
            logical_width: width,
            logical_height: height,
            compact: true,
            world_rect: Rect::new(0.0, 0.0, width, sheet_y),
            sheet_rect: Rect::new(0.0, sheet_y, width, height - sheet_y),
        }
    }

    pub fn contains_compact_sheet(&self, point: macroquad::prelude::Vec2) -> bool {
        self.compact && self.sheet_rect.contains(point)
    }
}

#[cfg(test)]
mod tests;
