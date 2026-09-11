//! Persistent night-market rotation and exchange history.

use serde::{Deserialize, Serialize};

pub const MARKET_OFFER_COUNT: usize = 3;
pub const MARKET_REFRESH_SECONDS: f32 = 30.0;
pub const MARKET_CONTRACT_SECONDS: f32 = 75.0;
pub const MARKET_CONTRACT_BONUS_FAVOR: u32 = 2;
pub const BROKER_STANDING_THRESHOLDS: [u32; 3] = [0, 3, 8];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketContract {
    pub offer_index: usize,
    #[serde(default = "default_contract_seconds")]
    pub remaining_seconds: f32,
    #[serde(default = "default_contract_bonus")]
    pub bonus_favor: u32,
}

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
    #[serde(default)]
    pub contract: Option<MarketContract>,
    #[serde(default)]
    pub completed_contracts: u32,
}

impl Default for MarketState {
    fn default() -> Self {
        Self {
            offer_index: 0,
            refresh_seconds: MARKET_REFRESH_SECONDS,
            completed_trades: 0,
            favor: 0,
            contract: None,
            completed_contracts: 0,
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
        if let Some(contract) = self.contract.as_mut() {
            contract.offer_index %= MARKET_OFFER_COUNT;
            if !contract.remaining_seconds.is_finite() || contract.remaining_seconds <= 0.0 {
                contract.remaining_seconds = MARKET_CONTRACT_SECONDS;
            }
            contract.bonus_favor = contract.bonus_favor.min(MARKET_CONTRACT_BONUS_FAVOR);
        }
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

    pub fn refresh_interval(&self) -> f32 {
        match self.standing_tier() {
            0 => MARKET_REFRESH_SECONDS,
            1 => 27.0,
            _ => 24.0,
        }
    }
}

fn default_refresh_seconds() -> f32 {
    MARKET_REFRESH_SECONDS
}

fn default_contract_seconds() -> f32 {
    MARKET_CONTRACT_SECONDS
}

fn default_contract_bonus() -> u32 {
    MARKET_CONTRACT_BONUS_FAVOR
}

#[cfg(test)]
mod tests;
