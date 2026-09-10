//! Research progress kept separate from the broader serialized session model.

use super::Technology;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchState {
    #[serde(default)]
    pub completed: Vec<Technology>,
    #[serde(default)]
    pub current: Option<Technology>,
    #[serde(default)]
    pub progress: f32,
}

impl Default for ResearchState {
    fn default() -> Self {
        Self {
            completed: Vec::new(),
            current: None,
            progress: 0.0,
        }
    }
}

impl ResearchState {
    pub fn is_unlocked(&self, technology: Technology) -> bool {
        self.completed.contains(&technology)
    }

    pub fn can_start(&self, technology: Technology) -> bool {
        !self.is_unlocked(technology)
            && self.current.is_none()
            && technology
                .prerequisite()
                .is_none_or(|prerequisite| self.is_unlocked(prerequisite))
    }
}
