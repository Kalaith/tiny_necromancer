# Tiny Necromancer — Game Design Document

## 1. High concept

The player is a hidden necromancer restoring a neglected cemetery beside a
road. Bones become undead labour, labour makes the cemetery grow, and every
visible sign of activity raises suspicion. The current release is a playable
vertical slice ending when a small settlement is established.

The presentation is an orthographic top-down Macroquad scene with a full-world
desktop viewport, contextual inspector, compact status strip, and stable command
dock. Smaller touch viewports keep the world visible above a bottom command
sheet with the same actions, plus visible zoom and recenter controls and
single-finger pan / two-finger pinch camera gestures. The tone is secluded,
strange, and quietly industrious.

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
priority list, which can be reordered from the touch-first Orders board. The
simulation chooses their targets and advances work between visible orders; the
player does not micromanage walking paths. Tile positions
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
colony. The Orders board exposes every repeat duty, its active automated worker
count, and touch-sized Up/Down controls. Older saves retain their authored
order and receive any newly introduced duty at the end of the list.
Loose bones and wood retain a source tile. A hauler walks to that source, picks
up a capacity-limited bundle, visibly carries it to Storage, and only then
credits the stockpile. This keeps world feedback and resource totals aligned.

### 5.3 Suspicion and events

Suspicion is clamped to 0–100 and advances through Calm, Rumour, Questioning,
and Investigation using the ordered thresholds in `game_config.json`.
`events.json` supplies the road-pressure event for each non-calm stage and its
resource, suspicion, and digging effects. Active events block simulation until
the player taps a choice.

Operational alerts are derived from the same session state: unattended
construction, loaded production without a Refine worker, loose materials
without a Haul order, open graves without a Dig order, and questioning-level
road pressure. Each alert is a touch target that selects its source so the
player can move from diagnosis to an order without searching the map.

Domain Stewardship adds a dedicated readout for the settlement: bound workers,
marked district tiles, ward charges, workforce activity, pressure stage, and
the latest pressure cause. Its touch controls independently show or hide zone
marks, worker destination routes, and a road-pressure watch overlay. The panel
also offers a source-selection target for the first active blocker and a
visible ward-charge response.

The current stewardship policy is persistent save data. Balanced respects the
shared worker priority list, Secure brings Guard duty forward and responds at
Rumour pressure, and Harvest brings Dig, Haul, and Wood forward when those
orders are available. The policy affects only automated workers; direct orders
remain authoritative.

After Domain Stewardship, marked districts also carry small operational rules:
Work tiles speed Dig and Wood work, Storage tiles increase a hauler's bundle,
and Patrol tiles strengthen guard mitigation. Empty districts and pre-domain
saves keep the original behavior, so the system remains additive rather than
making zone painting mandatory.

### 5.4 Determinism and saves

`SeededRng` is stored with the session and serialized into the versioned save.
Saves use `save_to_slot_with_version` and
`load_from_slot_with_migration`; legacy payloads receive deterministic defaults
for positioned buildings, research, zones, production, ward charges,
stewardship policy, and the necromancer. A pending necromancer destination and carried-resource type are
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
longer production run. A reserved follow-up cycle can be cancelled from the
kiln inspector; its recipe materials return immediately while the active cycle
keeps its progress.

## 7. UI and interaction flow

The main menu shows the game identity and visible **New Game** / **Continue**
targets. Playing shows the world first, then status, inspector, field notes,
and command dock. Panels return `UiAction` intents; `Game` applies those intents
to the session through the engine services.

Selection is tap/click on an object or ground tile. A selected worker exposes
job buttons, current destination, carried item, an idle reason, and, after
research, repeat priorities. The Orders panel reorders the shared list without
keyboard input, and automated workers can select Refine when a kiln cycle is
active. The selected worker receives a short route hint,
active jobs show compact badges and tool motion, and source progress appears in
the world. A selected grave exposes dig and raise actions. Building placement
previews footprint, collision, cost, and a textual failure reason, with
**Cancel placement** always visible. The selected necromancer shows a
destination marker, violet ritual pulse, and a **Cancel movement** target while
walking.

The field-notes area becomes an Operational Alerts tray when a real blocker
exists. Tapping an alert selects its source; when the cemetery is healthy, the
same space returns to fading field notes. The Settlement View minimap echoes
painted zones, structures, workers, the
necromancer, and the current selection so the domain remains legible while
the world is busy. Active zone tools preview the hovered tile with **Tap to
mark** or **Tap to clear** copy. The event modal blocks the world and presents
only choice buttons. Pause, save, load, recovery, and replay actions are also
visible controls. Desktop right-drag and wheel camera controls supplement the
touch path.

The History target opens a tall Field Notes Archive with the seven most recent
entries, newest-first age labels, and a visible Close target. It is available
from both the healthy field-notes surface and the operational-alert surface,
so important feedback is never hidden behind the current short list.

After Domain Stewardship, the Domain command opens a tall settlement readout.
Its three overlay buttons keep district marks, worker routes, and road
pressure independently readable on the same world-first map. A blocker can be
located from the readout, while **Quiet ward** remains a visible response when
charges and suspicion make it applicable. A touch-sized policy control cycles
between Balanced, Secure, and Harvest and states each policy's effect.

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
Stewardship now surfaces a compact district overview, actionable operational
alerts, toggleable zone, route, and pressure overlays, and small bonuses for
marked district rules. Responsive layouts now preserve those interactions at
smaller viewport sizes with a reduced world area and touch-sized bottom sheet.

The detailed presentation decisions and review checklist live in
`docs/visual-direction-and-ui-plan.md`.
