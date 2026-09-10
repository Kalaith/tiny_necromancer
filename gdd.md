# Tiny Necromancer — Game Design Document

## 1. High concept

The player is a hidden necromancer restoring a neglected cemetery beside a
road. Bones become undead labour, labour makes the cemetery grow, and every
visible sign of activity raises suspicion. The current release is a playable
vertical slice ending when a small settlement is established.

The presentation is a fixed-resolution, orthographic top-down Macroquad scene
with a full-world viewport, contextual inspector, compact status strip, and
stable command dock. The tone is secluded, strange, and quietly industrious.

## 2. Design pillars

1. The cemetery is the interface: objects are visible places and actors that
   can be selected directly.
2. Every essential action has a visible tap/click target; keyboard shortcuts
   are optional conveniences.
3. Progression changes how orders are given while preserving earlier commands.
4. Suspicion is a readable pressure system with causes, stages, and responses.
5. Simulation outcomes use the session-owned toolkit RNG so saves and tests are
   deterministic.

## 3. Core loop

1. Select a grave and assign a worker to dig it.
2. Haul loose bones and wood into the hidden stockpile.
3. Raise skeletons and a Brute when a Notable corpse is found.
4. Build the Work Shed and Grave Lantern and open more grave plots.
5. Research the technology chain, then use the newly available controls.
6. Keep suspicion below the configured victory limit while reaching the
   settlement milestone.

## 4. Player role and verbs

The player is a clandestine necromancer who directly selects objects, assigns
jobs, raises undead, places buildings, opens plots, starts research, paints
work areas, resolves road-pressure events, pauses, and saves.

Workers follow their assigned job or, after Binding Routines, the shared
priority list. The simulation chooses their targets and advances work between
visible orders; the player does not micromanage walking paths.

## 5. Systems and authored data

### 5.1 Economy and progression

Bones, mana, and wood are the primary resources. `game_config.json` authors
starting resources, mana capacity and regeneration, plot expansion costs,
research durations, corpse discovery chance, victory targets, and suspicion
thresholds. `buildings.json`, `undead.json`, and `jobs.json` author costs,
work rates, capacities, suspicion effects, and construction effects.

The vertical-slice victory rule requires the configured undead and usable-plot
targets, a complete Work Shed and Grave Lantern, a Brute, and suspicion below
the configured victory maximum.

### 5.2 Jobs and corpses

Jobs are Dig, Haul, Guard, Wood, and Build. Each job reads its work time,
output, speed, suspicion, and guard mitigation from `jobs.json`. Corpse quality
bands and Brute eligibility live in `corpses.json`; resurrection recipes live
in `undead.json`.

### 5.3 Suspicion and events

Suspicion is clamped to 0–100 and advances through Calm, Rumour, Questioning,
and Investigation using the ordered thresholds in `game_config.json`.
`events.json` supplies the road-pressure event for each non-calm stage and its
resource, suspicion, and digging effects. Active events block simulation until
the player taps a choice.

### 5.4 Determinism and saves

`SeededRng` is stored with the session and serialized into the versioned save.
Saves use `save_to_slot_with_version` and
`load_from_slot_with_migration`; legacy payloads receive deterministic defaults
for positioned buildings, research, zones, and the necromancer.

## 6. World and content inventory

The current world is a 10 × 8 cemetery with six authored grave positions, a
forest edge, a road, a mana source, a stockpile, one starting skeleton, two
building types, four research technologies, three corpse qualities, and three
zone types. A single sprite sheet supplies the necromancer, worker, and
building silhouettes; missing textures fall back to an obvious placeholder.

The planned colony expansion remains deliberately deferred: specialized
production, district policy, larger populations, trade, farms, housing, and
additional biomes are not part of the current victory slice.

## 7. UI and interaction flow

The main menu shows the game identity and visible **New Game** / **Continue**
targets. Playing shows the world first, then status, inspector, field notes,
and command dock. Panels return `UiAction` intents; `Game` applies those intents
to the session through the engine services.

Selection is tap/click on an object or ground tile. A selected worker exposes
job buttons and, after research, repeat priorities. A selected grave exposes
dig and raise actions. Building placement previews footprint, collision, cost,
and a textual failure reason, with **Cancel placement** always visible.

The event modal blocks the world and presents only choice buttons. Pause,
save, load, recovery, and replay actions are also visible controls. Desktop
right-drag and wheel camera controls supplement the touch path.

## 8. Toolkit mapping

| Need | Toolkit piece |
| --- | --- |
| Embedded JSON and diagnostics | `data_loader`, `include_json_str!` |
| Runtime texture loading | `AssetManager`, `TextureConfig` |
| UI layout and buttons | `VirtualUi`, `Pointer`, `SurfaceStyle`, `button_rect_tone_at` |
| Camera and picking | `CameraTransform`, `CameraBounds`, `TilePos` |
| Cross-system intents | `EventBus<UiAction>` |
| Persistence | Versioned slot helpers with migration |
| Deterministic randomness | `SeededRng` |
| Feedback | `NotificationManager` |
| Verification | `capture_window_conf`, `CaptureConfig`, and the shared capture script |

## 9. Scope and roadmap

The current implementation covers the first three visual-direction milestones:
world-first UI, a convincing cemetery scene, and the first research transition.
The next milestone is a deeper colony slice with walkability, linked storage,
and one complete material-to-construction chain. Later work can add district
summaries and domain controls only after the current interaction remains clear
at smaller viewport sizes.

The detailed presentation decisions and review checklist live in
`docs/visual-direction-and-ui-plan.md`.
