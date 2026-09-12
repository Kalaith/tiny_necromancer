//! Loose-resource piles and compatibility helpers for the economy state.

use super::ResourceKind;
use macroquad_toolkit::grid::TilePos;
use serde::{Deserialize, Serialize};

pub const DEFAULT_STORAGE_CAPACITY: i32 = 96;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LooseResourcePile {
    pub position: TilePos,
    pub amount: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyState {
    pub bones: i32,
    pub mana: i32,
    pub mana_fraction: f32,
    pub wood: i32,
    #[serde(default = "default_storage_capacity")]
    pub storage_capacity: i32,
    pub shovels: i32,
    pub loose_bones: i32,
    pub loose_wood: i32,
    #[serde(default)]
    pub ward_charges: i32,
    #[serde(default)]
    pub loose_bones_source: Option<TilePos>,
    #[serde(default)]
    pub loose_wood_source: Option<TilePos>,
    #[serde(default)]
    pub loose_bones_piles: Vec<LooseResourcePile>,
    #[serde(default)]
    pub loose_wood_piles: Vec<LooseResourcePile>,
    pub corpses: Vec<super::Corpse>,
}

impl EconomyState {
    pub fn stored_materials(&self) -> i32 {
        self.bones.saturating_add(self.wood)
    }

    pub fn storage_space(&self, capacity: i32) -> i32 {
        capacity.saturating_sub(self.stored_materials()).max(0)
    }

    pub fn store_resource(&mut self, resource: ResourceKind, amount: i32, capacity: i32) -> i32 {
        let stored = amount.max(0).min(self.storage_space(capacity));
        match resource {
            ResourceKind::Bones => self.bones += stored,
            ResourceKind::Wood => self.wood += stored,
        }
        stored
    }

    pub fn normalize_loose_piles(&mut self) {
        if self.loose_bones_piles.is_empty() && self.loose_bones > 0 {
            self.loose_bones_piles.push(LooseResourcePile {
                position: self
                    .loose_bones_source
                    .unwrap_or_else(compatibility_stockpile_position),
                amount: self.loose_bones,
            });
        }
        if self.loose_wood_piles.is_empty() && self.loose_wood > 0 {
            self.loose_wood_piles.push(LooseResourcePile {
                position: self
                    .loose_wood_source
                    .unwrap_or_else(compatibility_stockpile_position),
                amount: self.loose_wood,
            });
        }
        Self::compact_piles(&mut self.loose_bones_piles);
        Self::compact_piles(&mut self.loose_wood_piles);
        self.refresh_metadata(ResourceKind::Bones);
        self.refresh_metadata(ResourceKind::Wood);
    }

    pub fn loose_piles(&self, resource: ResourceKind, fallback: TilePos) -> Vec<LooseResourcePile> {
        let piles = self.piles(resource);
        if !piles.is_empty() {
            return piles.to_vec();
        }
        let (amount, source) = match resource {
            ResourceKind::Bones => (self.loose_bones, self.loose_bones_source),
            ResourceKind::Wood => (self.loose_wood, self.loose_wood_source),
        };
        if amount > 0 {
            vec![LooseResourcePile {
                position: source.unwrap_or(fallback),
                amount,
            }]
        } else {
            Vec::new()
        }
    }

    pub fn loose_amount_at(&self, resource: ResourceKind, position: TilePos) -> i32 {
        self.loose_piles(resource, compatibility_stockpile_position())
            .iter()
            .filter(|pile| pile.position == position)
            .map(|pile| pile.amount)
            .sum()
    }

    pub fn add_loose(&mut self, resource: ResourceKind, position: TilePos, amount: i32) {
        if amount <= 0 {
            return;
        }
        self.normalize_loose_piles();
        let piles = self.piles_mut(resource);
        if let Some(pile) = piles.iter_mut().find(|pile| pile.position == position) {
            pile.amount += amount;
        } else {
            piles.push(LooseResourcePile { position, amount });
        }
        self.refresh_metadata(resource);
    }

    pub fn take_loose(&mut self, resource: ResourceKind, position: TilePos, amount: i32) -> i32 {
        if amount <= 0 {
            return 0;
        }
        self.normalize_loose_piles();
        let taken = {
            let piles = self.piles_mut(resource);
            let Some(index) = piles.iter().position(|pile| pile.position == position) else {
                return 0;
            };
            let taken = piles[index].amount.min(amount);
            piles[index].amount -= taken;
            if piles[index].amount == 0 {
                piles.remove(index);
            }
            taken
        };
        self.refresh_metadata(resource);
        taken
    }

    fn piles(&self, resource: ResourceKind) -> &[LooseResourcePile] {
        match resource {
            ResourceKind::Bones => &self.loose_bones_piles,
            ResourceKind::Wood => &self.loose_wood_piles,
        }
    }

    fn piles_mut(&mut self, resource: ResourceKind) -> &mut Vec<LooseResourcePile> {
        match resource {
            ResourceKind::Bones => &mut self.loose_bones_piles,
            ResourceKind::Wood => &mut self.loose_wood_piles,
        }
    }

    fn compact_piles(piles: &mut Vec<LooseResourcePile>) {
        let mut compacted = Vec::with_capacity(piles.len());
        for pile in piles.drain(..) {
            if pile.amount <= 0 {
                continue;
            }
            if let Some(existing) = compacted
                .iter_mut()
                .find(|existing: &&mut LooseResourcePile| existing.position == pile.position)
            {
                existing.amount += pile.amount;
            } else {
                compacted.push(pile);
            }
        }
        *piles = compacted;
    }

    fn refresh_metadata(&mut self, resource: ResourceKind) {
        let (total, source) = {
            let piles = self.piles(resource);
            (
                piles.iter().map(|pile| pile.amount).sum::<i32>(),
                piles.last().map(|pile| pile.position),
            )
        };
        match resource {
            ResourceKind::Bones => {
                self.loose_bones = total;
                self.loose_bones_source = source;
            }
            ResourceKind::Wood => {
                self.loose_wood = total;
                self.loose_wood_source = source;
            }
        }
    }
}

fn default_storage_capacity() -> i32 {
    DEFAULT_STORAGE_CAPACITY
}

fn compatibility_stockpile_position() -> TilePos {
    // Saves from before the layout became authored data have no source tile.
    TilePos::new(6, 5)
}
