//! Persistent night-market rotation and exchange history.

use serde::{Deserialize, Serialize};

pub const MARKET_OFFER_COUNT: usize = 3;
pub const MARKET_REFRESH_SECONDS: f32 = 30.0;
pub const BROKER_STANDING_THRESHOLDS: [u32; 3] = [0, 3, 8];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    #[serde(default)]
    pub offer_index: usize,
    #[serde(default = "default_refresh_seconds")]
    pub refresh_seconds: f32,
    #[serde(default)]
    pub completed_trades: u32,
    #[serde(default)]
    pub favor: u32,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            offer_index: 0,
            refresh_seconds: MARKET_REFRESH_SECONDS,
            completed_trades: 0,
            favor: 0,
        }
    }
}

impl MarketState {
    pub fn normalize(&mut self) {
        self.offer_index %= MARKET_OFFER_COUNT;
        if !self.refresh_seconds.is_finite() || self.refresh_seconds <= 0.0 {
            self.refresh_seconds = MARKET_REFRESH_SECONDS;
        }
        self.favor = self.favor.max(self.completed_trades);
    }

    pub fn standing_tier(&self) -> usize {
        BROKER_STANDING_THRESHOLDS
            .iter()
            .enumerate()
            .rev()
            .find_map(|(tier, threshold)| (self.favor >= *threshold).then_some(tier))
            .unwrap_or(0)
    }

    pub fn standing_label(&self) -> &'static str {
        match self.standing_tier() {
            0 => "Whisper",
            1 => "Acquainted",
            _ => "Trusted",
        }
    }

    pub fn next_standing_target(&self) -> Option<u32> {
        BROKER_STANDING_THRESHOLDS
            .iter()
            .copied()
            .find(|threshold| *threshold > self.favor)
    }

    pub fn standing_progress(&self) -> f32 {
        let tier = self.standing_tier();
        let Some(target) = self.next_standing_target() else {
            return 1.0;
        };
        let start = BROKER_STANDING_THRESHOLDS[tier];
        (self.favor.saturating_sub(start) as f32 / (target - start) as f32).clamp(0.0, 1.0)
    }
}

fn default_refresh_seconds() -> f32 {
    MARKET_REFRESH_SECONDS
}

#[cfg(test)]
mod tests;
