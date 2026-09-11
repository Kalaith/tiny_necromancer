//! Save-persistent kiln history and recipe outcome totals.

use crate::data::ProductionRecipeKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductionLedger {
    #[serde(default)]
    pub total_cycles: u32,
    #[serde(default)]
    pub ward_cycles: u32,
    #[serde(default)]
    pub hush_ash_cycles: u32,
    #[serde(default)]
    pub wards_sealed: i32,
    #[serde(default)]
    pub suspicion_quieted: f32,
}

impl ProductionLedger {
    pub fn record(
        &mut self,
        recipe: ProductionRecipeKind,
        output_amount: i32,
        suspicion_delta: f32,
    ) {
        self.total_cycles = self.total_cycles.saturating_add(1);
        match recipe {
            ProductionRecipeKind::WardCharge => {
                self.ward_cycles = self.ward_cycles.saturating_add(1);
            }
            ProductionRecipeKind::HushAsh => {
                self.hush_ash_cycles = self.hush_ash_cycles.saturating_add(1);
            }
        }
        self.wards_sealed = self.wards_sealed.saturating_add(output_amount.max(0));
        if suspicion_delta < 0.0 {
            self.suspicion_quieted += -suspicion_delta;
        }
    }

    pub fn summary(&self) -> String {
        if self.total_cycles == 0 {
            return "Kiln ledger · no cycles sealed yet.".to_owned();
        }
        format!(
            "Kiln ledger · {} cycles · {} wards sealed · {:.0} suspicion quieted",
            self.total_cycles, self.wards_sealed, self.suspicion_quieted
        )
    }
}

#[cfg(test)]
mod tests;
