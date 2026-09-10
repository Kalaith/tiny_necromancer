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
6. Build the Ossuary Kiln and assign a worker to refine ward charges.
7. Keep suspicion below the configured victory limit while reaching the
   settlement milestone.

## 4. Player role and verbs

The player is a clandestine necromancer who directly selects objects, assigns
jobs, raises undead, places buildings, opens plots, starts research, paints
work areas, resolves road-pressure events, pauses, and saves.

Workers follow their assigned job or, after Binding Routines, the shared
priority list. The simulation chooses their targets and advances work between
visible orders; the player does not micromanage walking paths. Tile positions
remain authoritative for simulation and saves, while the runtime presents each
step through a short interpolation with facing, walking sway, and job motion.
The necromancer uses the same four-way navigation, accepts a new destination by
tapping the clearing, and can cancel a route from the visible inspector.

## 5. Systems and authored data

### 5.1 Economy and progression

Bones, mana, and wood are the primary resources. `game_config.json` authors
starting resources, mana capacity and regeneration, plot expansion costs,
research durations, corpse discovery chance, victory targets, and suspicion
thresholds. `buildings.json`, `undead.json`, and `jobs.json` author costs,
work rates, capacities, suspicion effects, construction effects, and the
Ossuary Kiln's ward-charge recipe.

The vertical-slice victory rule requires the configured undead and usable-plot
targets, a complete Work Shed and Grave Lantern, a Brute, and suspicion below
the configured victory maximum.

### 5.2 Jobs and corpses

Jobs are Dig, Haul, Guard, Wood, Build, and Refine. Each job reads its work
time, output, speed, suspicion, and guard mitigation from `jobs.json`. Corpse quality
bands and Brute eligibility live in `corpses.json`; resurrection recipes live
in `undead.json`.

Painted Work areas guide digging and wood gathering toward the nearest eligible
grave or forest tile, Storage areas choose the hauler's drop points, and Patrol
areas choose the nearest guard post. Haulers use the
nearest marked Storage tile from their current position, which keeps larger
stockpile districts practical. An empty area keeps the original fallback
target so marking a zone is never required to continue a job. Tapping a marked
tile clears it, so an accidental mark can be corrected without restarting the
colony.
Loose bones and wood retain a source tile. A hauler walks to that source, picks
up a capacity-limited bundle, visibly carries it to Storage, and only then
credits the stockpile. This keeps world feedback and resource totals aligned.

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
for positioned buildings, research, zones, production, ward charges, and the
necromancer. A pending necromancer destination and carried-resource type are
persistent; render-only interpolation and animation clocks are reset from the
authoritative tiles after New Game and Load so actors cannot detach visually.

## 6. World and content inventory

The current world is a 10 × 8 cemetery with six authored grave positions, a
forest edge, a road, a mana source, a stockpile, one starting skeleton, three
building types, four research technologies, three corpse qualities, and three
zone types. A single sprite sheet supplies the necromancer, worker, and
building silhouettes; missing textures fall back to an obvious placeholder.

The kiln's first specialized production loop is playable, while district
policy, larger populations, trade, farms, housing, and additional biomes remain
deliberately deferred from the current victory slice.

The Ossuary Kiln loads one ward-charge cycle at a time and accepts up to three
reserved follow-up cycles. Reserved materials are removed when the cycle is
queued, so the player can see the true available stock before committing to a
longer production run.

## 7. UI and interaction flow

The main menu shows the game identity and visible **New Game** / **Continue**
targets. Playing shows the world first, then status, inspector, field notes,
and command dock. Panels return `UiAction` intents; `Game` applies those intents
to the session through the engine services.

Selection is tap/click on an object or ground tile. A selected worker exposes
job buttons, current destination, carried item, an idle reason, and, after
research, repeat priorities. The selected worker receives a short route hint,
active jobs show compact badges and tool motion, and source progress appears in
the world. A selected grave exposes dig and raise actions. Building placement
previews footprint, collision, cost, and a textual failure reason, with
**Cancel placement** always visible. The selected necromancer shows a
destination marker, violet ritual pulse, and a **Cancel movement** target while
walking.

The Settlement View minimap echoes painted zones, structures, workers, the
necromancer, and the current selection so the domain remains legible while
the world is busy. Active zone tools preview the hovered tile with **Tap to
mark** or **Tap to clear** copy. The event modal blocks the world and presents
only choice buttons. Pause, save, load, recovery, and replay actions are also
visible controls. Desktop right-drag and wheel camera controls supplement the
touch path.

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

The current implementation covers the first colony slice beyond the initial
visual-direction milestones: workers route around structure footprints, Work,
Storage, and Patrol zones choose nearby destinations, actors interpolate between
simulation tiles, and procedural animation makes digging, hauling, gathering,
construction, guarding, refining, walking, and ritual focus readable. Domain
Stewardship surfaces a compact district overview. The next milestone can add
larger-scale production alerts and stewardship controls while preserving the
clear touch-first interaction at smaller viewport sizes.

The detailed presentation decisions and review checklist live in
`docs/visual-direction-and-ui-plan.md`.
