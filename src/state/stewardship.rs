//! Persistent policy for workers that follow the domain's shared priorities.

use super::JobKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StewardshipPolicy {
    #[default]
    Balanced,
    Secure,
    Harvest,
}

impl StewardshipPolicy {
    pub fn id(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Secure => "secure",
            Self::Harvest => "harvest",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Balanced => "Balanced",
            Self::Secure => "Secure",
            Self::Harvest => "Harvest",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Balanced => Self::Secure,
            Self::Secure => Self::Harvest,
            Self::Harvest => Self::Balanced,
        }
    }

    pub fn bias(self, job: JobKind) -> usize {
        match self {
            Self::Balanced => 0,
            Self::Secure => usize::from(job != JobKind::Guard),
            Self::Harvest => {
                usize::from(!matches!(job, JobKind::Dig | JobKind::Haul | JobKind::Wood))
            }
        }
    }
}
