//! Save-persistent route choices for automated district workers.

use super::ZoneKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutePolicy {
    #[default]
    MarkedFirst,
    Nearest,
    MarkedOnly,
}

impl RoutePolicy {
    pub fn label(self) -> &'static str {
        match self {
            Self::MarkedFirst => "Marked first",
            Self::Nearest => "Nearest",
            Self::MarkedOnly => "Marked only",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::MarkedFirst => Self::Nearest,
            Self::Nearest => Self::MarkedOnly,
            Self::MarkedOnly => Self::MarkedFirst,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictRoutePolicies {
    #[serde(default)]
    pub work: RoutePolicy,
    #[serde(default)]
    pub storage: RoutePolicy,
    #[serde(default)]
    pub patrol: RoutePolicy,
}

impl DistrictRoutePolicies {
    pub fn for_kind(self, kind: ZoneKind) -> RoutePolicy {
        match kind {
            ZoneKind::Work => self.work,
            ZoneKind::Storage => self.storage,
            ZoneKind::Patrol => self.patrol,
        }
    }

    pub fn cycle(&mut self, kind: ZoneKind) -> RoutePolicy {
        let policy = match kind {
            ZoneKind::Work => &mut self.work,
            ZoneKind::Storage => &mut self.storage,
            ZoneKind::Patrol => &mut self.patrol,
        };
        *policy = policy.next();
        *policy
    }
}
