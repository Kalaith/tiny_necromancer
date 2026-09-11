//! Persistent one-tier upgrades for completed settlement structures.

use super::{Building, BuildingKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildingUpgrades {
    #[serde(default)]
    pub work_shed: u8,
    #[serde(default)]
    pub grave_lantern: u8,
    #[serde(default)]
    pub ossuary_kiln: u8,
}

impl BuildingUpgrades {
    pub fn tier(&self, kind: BuildingKind) -> u8 {
        match kind {
            BuildingKind::WorkShed => self.work_shed,
            BuildingKind::GraveLantern => self.grave_lantern,
            BuildingKind::OssuaryKiln => self.ossuary_kiln,
        }
    }

    pub fn upgrade(&mut self, kind: BuildingKind) -> bool {
        let tier = match kind {
            BuildingKind::WorkShed => &mut self.work_shed,
            BuildingKind::GraveLantern => &mut self.grave_lantern,
            BuildingKind::OssuaryKiln => &mut self.ossuary_kiln,
        };
        if *tier >= 1 {
            return false;
        }
        *tier = 1;
        true
    }

    pub fn normalize(&mut self, buildings: &[Building]) {
        self.work_shed = self.work_shed.min(1).min(u8::from(
            buildings
                .iter()
                .any(|building| building.kind == BuildingKind::WorkShed && building.complete),
        ));
        self.grave_lantern = self.grave_lantern.min(1).min(u8::from(
            buildings
                .iter()
                .any(|building| building.kind == BuildingKind::GraveLantern && building.complete),
        ));
        self.ossuary_kiln = self.ossuary_kiln.min(1).min(u8::from(
            buildings
                .iter()
                .any(|building| building.kind == BuildingKind::OssuaryKiln && building.complete),
        ));
    }
}
