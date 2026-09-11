//! Serializable workers and the shared repeat-priority policy.

use super::{JobKind, ResourceKind, RoutePolicy, UndeadKind, WorkerStatus};
use macroquad_toolkit::grid::TilePos;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HaulDestination {
    #[default]
    #[serde(rename = "storage")]
    Storage,
    #[serde(rename = "kiln")]
    Kiln,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaulPlan {
    pub resource: ResourceKind,
    pub source: TilePos,
    pub destination: TilePos,
    pub storage_policy: RoutePolicy,
    #[serde(default)]
    pub destination_kind: HaulDestination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub id: u32,
    pub name: String,
    pub kind: UndeadKind,
    pub assignment: JobKind,
    pub position: TilePos,
    pub status: WorkerStatus,
    pub progress: f32,
    pub target_plot: Option<usize>,
    pub carrying: i32,
    #[serde(default)]
    pub carrying_resource: Option<ResourceKind>,
    #[serde(default)]
    pub priority_mode: bool,
    #[serde(default)]
    pub haul_plan: Option<HaulPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkforceState {
    pub workers: Vec<Worker>,
    pub selected_worker: usize,
    pub next_worker_id: u32,
    #[serde(default = "default_priorities")]
    pub priorities: Vec<JobKind>,
}

fn default_priorities() -> Vec<JobKind> {
    vec![
        JobKind::Guard,
        JobKind::Haul,
        JobKind::Dig,
        JobKind::Build,
        JobKind::Wood,
        JobKind::Refine,
    ]
}

impl WorkforceState {
    pub fn default_priorities() -> Vec<JobKind> {
        default_priorities()
    }

    pub fn normalize_priorities(&mut self) {
        let defaults = default_priorities();
        let mut normalized = Vec::with_capacity(defaults.len());
        for priority in self.priorities.iter().copied() {
            if !normalized.contains(&priority) {
                normalized.push(priority);
            }
        }
        for priority in defaults {
            if !normalized.contains(&priority) {
                normalized.push(priority);
            }
        }
        self.priorities = normalized;
    }
}
