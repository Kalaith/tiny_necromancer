//! Persistent night-market rotation and exchange history.

use serde::{Deserialize, Serialize};

pub const MARKET_OFFER_COUNT: usize = 3;
pub const MARKET_REFRESH_SECONDS: f32 = 30.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketState {
    #[serde(default)]
    pub offer_index: usize,
    #[serde(default = "default_refresh_seconds")]
    pub refresh_seconds: f32,
    #[serde(default)]
    pub completed_trades: u32,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            offer_index: 0,
            refresh_seconds: MARKET_REFRESH_SECONDS,
            completed_trades: 0,
        }
    }
}

impl MarketState {
    pub fn normalize(&mut self) {
        self.offer_index %= MARKET_OFFER_COUNT;
        if !self.refresh_seconds.is_finite() || self.refresh_seconds <= 0.0 {
            self.refresh_seconds = MARKET_REFRESH_SECONDS;
        }
    }
}

fn default_refresh_seconds() -> f32 {
    MARKET_REFRESH_SECONDS
}
