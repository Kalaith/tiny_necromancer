//! Optional shared name-generator client used to prefetch undead names.

use crate::state::Worker;
use macroquad_toolkit::net::{HttpClient, Pending};
use serde::Deserialize;

const REQUEST_TIMEOUT_SECONDS: f32 = 6.0;
const RETRY_COOLDOWN_SECONDS: f32 = 10.0;
const PREFETCH_COUNT: usize = 8;
const MIN_BUFFERED_NAMES: usize = 3;

#[cfg(debug_assertions)]
const API_BASE: &str = "http://127.0.0.1/name_generator/api/v1";
#[cfg(not(debug_assertions))]
const API_BASE: &str = "/name_generator/api/v1";

#[derive(Debug, Deserialize)]
struct NameResponse {
    names: Vec<String>,
}

/// A non-blocking pool of names from the shared WebHatchery service.
///
/// The game can still raise immediately when the service is not configured or
/// reachable; [`crate::engine::corpses`] supplies the deterministic fallback.
pub struct NameGenerator {
    api: Option<HttpClient>,
    pending: Option<Pending<NameResponse>>,
    buffered_names: Vec<String>,
    retry_cooldown: f32,
}

impl NameGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            api: configured_api(),
            pending: None,
            buffered_names: Vec::new(),
            retry_cooldown: 0.0,
        };
        generator.request_more();
        generator
    }

    /// Poll the in-flight request once per frame and keep a small buffer ready.
    pub fn poll(&mut self, dt: f32) {
        self.retry_cooldown = (self.retry_cooldown - dt.max(0.0)).max(0.0);

        let result = self
            .pending
            .as_mut()
            .and_then(|request| request.poll_timed(dt, REQUEST_TIMEOUT_SECONDS));
        if let Some(result) = result {
            self.pending = None;
            match result {
                Ok(response) => self.absorb(response.names),
                Err(_) => self.retry_cooldown = RETRY_COOLDOWN_SECONDS,
            }
        }

        if self.pending.is_none()
            && self.retry_cooldown <= 0.0
            && self.buffered_names.len() < MIN_BUFFERED_NAMES
        {
            self.request_more();
        }
    }

    /// Take the first buffered name that does not duplicate a current worker.
    pub fn take_name(&mut self, workers: &[Worker]) -> Option<String> {
        let index = self.buffered_names.iter().position(|candidate| {
            !workers
                .iter()
                .any(|worker| worker.name.eq_ignore_ascii_case(candidate))
        })?;
        Some(self.buffered_names.remove(index))
    }

    fn request_more(&mut self) {
        let Some(api) = self.api.as_ref() else {
            return;
        };
        self.pending = Some(api.get(&format!(
            "/generate_name.php?count={PREFETCH_COUNT}&gender=any&culture=any&method=fantasy_generated&type=nickname&period=medieval&excludeReal=1"
        )));
    }

    fn absorb(&mut self, names: Vec<String>) {
        for name in names {
            let name = name.trim();
            if name.is_empty()
                || self
                    .buffered_names
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(name))
            {
                continue;
            }
            self.buffered_names.push(name.to_owned());
        }
    }
}

fn configured_api() -> Option<HttpClient> {
    let key = option_env!("TINY_NECROMANCER_NAME_GENERATOR_API_KEY")
        .or(option_env!("NAME_GENERATOR_API_KEY"))
        .or(if cfg!(debug_assertions) {
            Some("development_key_123")
        } else {
            None
        })?;
    Some(HttpClient::new(API_BASE).with_header("X-API-KEY", key))
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self::new()
    }
}
