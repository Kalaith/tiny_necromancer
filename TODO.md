# Tiny Necromancer TODO

This list records follow-up work identified while reviewing the updated
`AGENTS.md` and `CODE_STANDARDS.md` on 2026-09-12.

Current baseline: `cargo fmt -- --check`, `cargo test`, and
`cargo clippy --all-targets --all-features -- -D warnings` pass. The source gate
passes with an empty exception list, and every Rust file is currently below the
800-line hard limit.

## High Priority

- [ ] Migrate the binary-only crate to a testable library boundary. Add
  `src/lib.rs`, move the game modules behind that library, and keep `main.rs`
  focused on the Macroquad entry point and loop. Rework integration tests to
  exercise intentional public APIs instead of only checking repository files.
- [ ] Move the legacy unit tests out of `src/` into the crate-level `tests/`
  directory. This includes the `src/data/tests.rs`, `src/state/**/tests.rs`,
  `src/engine/**/tests.rs`, and `src/ui/**/tests.rs` trees. Preserve the useful
  regression coverage, split suites by responsibility, remove `#[cfg(test)]`
  module declarations from production files, and keep each feature near the
  five-test target using table-driven assertions where appropriate.
- [ ] Remove the broad `#[allow(dead_code)]` on `UiAction`. Delete or add a
  real visible action path for currently unconstructed variants such as
  `SelectPlot` and `QueueBuilding`, then remove no-op unused-value workarounds
  such as `let _ = self.assets.len()`.
- [ ] Finish the data-driven content pass. Move authored map layout and
  starting content currently hardcoded in `src/state.rs` (plot/building
  positions, road and resource locations, forest tiles, the starting worker,
  and initial feedback) into typed JSON under `assets/data/`. Audit player-facing
  strings in `src/game.rs`, `src/state.rs`, and `src/ui/` and move authored text
  into JSON where it is content rather than presentation/layout. Extend
  semantic validation and data tests for the new references.

## Maintainability

- [ ] Split production files that are already past the 600-line planning
  threshold, keeping named module filenames and cohesive responsibilities:
  `src/state.rs` (793), `src/game/capture.rs` (771), `src/ui/world.rs` (724),
  `src/engine/jobs.rs` (702), `src/ui/hud.rs` (697), `src/engine/districts.rs`
  (685), `src/ui/world_feedback.rs` (679), `src/ui/responsive/panels.rs`
  (671), `src/game.rs` (651), and `src/ui/responsive.rs` (604).
- [ ] Refactor functions beyond the 100-line absolute maximum while doing the
  file split. The current offenders include `Game::apply_action`,
  `GameData::validate`, `jobs::simulate_haul`, `jobs::simulate`,
  `draw_stewardship_readout`, `draw_compact_domain_panel`, the worker and
  building inspectors, `draw_worker_feedback`, `draw_feed_history_panel`,
  `draw_compact_status`, `draw_route_hint`, `draw_compact_zones_panel`,
  `draw_graves`, and `GameSession::new`.
- [ ] During the refactor, keep state ownership explicit: engine services
  should return results, UI modules should continue returning `UiAction`
  intents, and action dispatch/persistence should be separated from rendering
  where a cohesive module boundary makes that ownership clearer.

## Verification Follow-up

- [ ] After the library/test migration and any gameplay changes, run the
  parameterless project publisher and refresh only the affected captures in
  `docs/verification/`. Recheck common desktop and narrow touch viewports,
  including menu entry, tutorial guidance, placement cancellation, event
  recovery, save/load, and camera controls.
