# Visual direction and UI transition plan

Status: implementation record for the playable vertical slice. Deferred art and
large-colony systems remain design direction rather than shipped scope.

## Core direction

A small, personal necromancy game that gradually becomes an undead colony builder. The player starts by watching one skeleton work beside one grave. Research lets them delegate repeated decisions, lay out a settlement, and eventually manage districts in that same landscape.

The world is the primary interface. Graves, resources, workers, buildings, and threats should be visible places and actors that the player can select. UI explains what they select and helps them give orders.

The supplied image is a visual reference, not a mandatory feature list or literal screen layout. Adopt its wooded cemetery, readable figures, warm lanterns, violet magic, visible labour, and increasingly developed settlement. Its promotional left column, “Early Game” and “End Game” headings, and descriptive captions belong outside gameplay. Farms, food, trade, housing, and iron are possible later systems, not requirements inferred from the image.

## Non-negotiable title rule

- The game name and logo appear only on the title page.
- No game name in the playing HUD, pause menu, research screen, loading overlay over a session, events, victory screen, or gameplay watermark.
- Use functional labels such as “Paused,” “Research,” and “Settlement established.” Location names such as “Hidden Cemetery” are allowed when useful, without creating a replacement permanent banner.
- For literal compliance, use a neutral native window caption such as “Game”; the title page itself carries the branding. Internal identifiers, save metadata, and distribution filenames can retain the product name.
- The title page is a dedicated composition, not a gameplay header showing through an overlay. Show the logo once, an atmospheric cemetery scene, and New Game / Continue / Settings / Quit where supported.

## Art direction

Use a fixed, orthographic top-down view with slightly angled character and building artwork so faces, doorways, and roof shapes remain readable. Keep a square ground grid; avoid rotating the camera or introducing an isometric diamond grid.

Aim for detailed pixel art with restrained texture. Prototype at a consistent 32-pixel ground tile scale, with characters roughly one tile tall and larger buildings using multiple tiles. Validate silhouettes at gameplay zoom before producing the full set. Use crisp texture filtering and stable sprite alignment; test zoom steps for shimmer. UI text remains separately rendered for legibility.

| Element | Visual treatment | Gameplay purpose |
| --- | --- | --- |
| Terrain | Moss green grass, brown soil, irregular edges, worn paths | Make the cemetery feel like a place instead of a board |
| Structures | Weathered timber, cool stone, iron fences, distinct roof shapes | Recognize function without reading every label |
| Undead | Pale bone silhouettes, different proportions and tools by role | Follow individual workers early and read labour at colony scale |
| Necromancer | Dark robe, clear staff silhouette, controlled violet glow | Keep a recognizable player presence as the settlement grows |
| Light | Amber lanterns; violet rituals; limited sickly green processing effects | Separate ordinary work, magic, and specialized production |
| HUD | Charcoal surfaces, thin muted metal borders, ivory text | Support the landscape without competing with it |
| Feedback | Cyan selection outline; amber warning icon; red critical icon | Communicate state through shape and text as well as colour |

The tone is secluded, strange, and quietly industrious. Avoid uniform black terrain, giant decorative borders, and constant purple glow. Darkness must never hide selectable objects or work states.

Construction visibly progresses through a marked footprint, scaffold, and finished structure. Graves change from undisturbed stone to disturbed earth to an open excavation. Workers carry source-backed bundles and perform short digging, hauling, woodcutting, building, guarding, and refining animations. Tool motion and carried goods make activity readable without permanent overhead text. Runtime motion interpolates between authoritative tile positions at roughly 2–3 tiles per second; pause freezes both motion and the animation clock.

Draw ground first, then low details, then actors and structures sorted by their ground contact point, then effects and selection feedback. Fade obstructing roofs or foliage when a selected actor is behind them. Decorative trees must not create misleading walkable gaps.

## Progression: research changes how the player gives orders

The following technology names and population ranges are proposals. Population describes expected scale; completed research unlocks controls. Do not gate UI solely on a worker count or elapsed time.

| Stage | Suggested scale | Technology / requirement | World change | Interface change |
| --- | --- | --- | --- | --- |
| Lone practitioner | 1–3 undead | Starting abilities | One grave, a clearing, a ruined shed, scattered supplies | Close camera, compact resources, selected-object actions |
| Small operation | 3–8 undead | **Binding Routines**, researched at the restored shed | Stockpile and repeated work routes become visible | Jobs panel with priorities and repeat orders |
| Permanent settlement | 8–20 undead | **Gravecraft**, following Binding Routines | Placeable buildings, paths, work areas, expanding boundary | Build palette, placement preview, zone painting; optional minimap |
| Organized colony | 20+ undead | **Ossuary Logistics**, following Gravecraft | Linked storage and specialized production areas | Hauling rules, production queues, population summary and alerts |
| Undead domain | Larger settlement | **Domain Stewardship**, following Logistics | District landmarks, patrol routes, ward network | District summaries and overlay controls; detailed worker selection remains available |

Introduce research through a contextual “Study bindings” action at the shed. Once this starts, expose the Research control in its permanent position. Show the current project and the next reachable discoveries; reserve the full tree for the expanded panel.

Each unlock teaches one practical change: “Set digging to repeat,” then “Place a stockpile,” then “Paint a work area.” Keep earlier commands available. Open the new control with a small highlight and a dismissible explanation; do not rearrange the entire HUD or force a zoom change.

Keep the necromancer present throughout. The slice now provides a selectable ritual actor with touch-to-move, the same four-way navigation as workers, a persistent destination, visible retargeting, route cancellation, robe sway, staff pulse, and violet ritual effects. Direct keyboard movement remains unnecessary; strategic commands do not require walking to every worker.

## Gameplay screen layout

Use the full screen as the world viewport with compact overlays. At 1280 × 720, target at least 75% of the world unobscured during ordinary play with panels collapsed, and at least 60% with one inspector open.

| Region | Early game | Colony game |
| --- | --- | --- |
| Top left | Bones, mana, wood; population | Same anchors; add unlocked resources with an overflow drawer |
| Top right | Suspicion, pause/menu, elapsed time | Same positions; day/time and speed controls only once supported |
| Bottom centre | Raise and available build actions | Stable command dock: Build, Orders, Undead, Research, Zones; tools appear as unlocked |
| Right edge | Collapsible selected-object inspector | Same inspector; production, storage, district detail according to selection |
| Bottom left | Up to three short, fading events | Actionable alerts with a History drawer; select an alert to find its source |
| Lower right | No minimap for the initial clearing | Toggleable minimap once the settlement exceeds the useful camera view |

Use a roughly 280–320 pixel inspector at the baseline resolution. Keep only one large management panel open at a time. A minimap shares the right edge when space permits and hides while a tall inspector is open. Avoid stacking panels until the map is a narrow leftover strip.

Global controls stay in place across progression. New tools fill reserved dock positions or a consistent overflow menu; do not move existing buttons. Reveal resources only when obtainable or immediately needed by a visible recipe. Do not show empty food, iron, stone, or trade systems merely to resemble the reference.

At smaller resolutions, use a bottom inspector sheet, resource overflow, and a
scrollable command dock. Keep essential targets at least 44 logical pixels.
The shipped compact layout uses a reduced world viewport above the sheet while
keeping selection actions, management panels, event choices, pause, and
placement cancellation visible through touch. Hover can add detail, but every
action must also work through selection for touch. Test larger text without
clipping costs or dismiss controls.

## Selection and orders

| Selected object | Inspector information | Main actions |
| --- | --- | --- |
| Grave | State, progress, contents when known | Dig, assign worker, raise if valid |
| Worker | Name, role, current job, carried item, reason for idleness | Give order, set priority after research, locate |
| Tree / resource node | Remaining material, work required | Harvest, designate area after research |
| Building | Construction state or output, staff, requirements | Assign, queue or cancel reserved production, upgrade when supported |
| Ground | Terrain and placement validity | Move selected actor or enter available construction tools |
| District | Workforce, stored resources, blocked work | Set policy, adjust priorities, locate bottleneck |

Left click or tap selects. A clearly chosen command followed by a ground click issues an order; do not overload ordinary selection with accidental movement. Start with the current right-drag camera pan and wheel zoom. Any future right-click quick order must distinguish a click from a drag. Provide camera buttons or gestures for touch. Escape cancels placement first, then closes the current panel, then pauses.

Building placement shows footprint, entrance, valid/invalid outline, cost, and a textual failure reason. Draw the grid only during placement, designation, or an optional overlay. Default labels appear for selection, hover, or meaningful trouble. At distant zoom, replace tiny worker details with restrained role indicators and district summaries.

HUD input consumes clicks before world selection. Dragging, closing an inspector, or scrolling a palette must never also select a grave or place a building. Modals block the underlying world consistently.

## Preserve the game's distinctive pressure

Suspicion remains an important system, not a buried statistic. Keep a small persistent indicator with its current stage. Expanding it explains the largest contributors and available responses. Show local causes in the world where the simulation supports them: visible work near the road, a patrol, exposed remains, or an active ward. Until those entities exist, use accurate alerts rather than decorative threats that imply nonexistent mechanics.

Resource gains should appear briefly at their source, with totals in the HUD. Longer production chains need explicit blocked states such as “Waiting for wood” and “No hauler assigned.” Routine work does not require a modal. Decisions and serious threats can open one.

Do not add conventional food and housing needs automatically. If later design requires them, give them a necromancy-specific purpose, such as crypt capacity or maintaining living collaborators. First establish the core bones → undead labour → construction → research → expansion loop.

## Current prototype and remaining direction

The current game has a 10 × 8 world, six authored grave positions, one starting skeleton, three building types, jobs, repeat priorities, corpse qualities, suspicion events, versioned save support, a selectable necromancer, positioned buildings, four research technologies, editable work areas, deterministic worker routing, route-aware Work/Storage/Patrol destinations, stable multi-post patrol coverage, reachable patrol coverage accounting, aggregated storage and patrol districts, an Ossuary Kiln production recipe with a three-cycle queue, a district overview with a stateful minimap, actionable operational alerts with a seven-entry History drawer, Domain Stewardship overlays for zones, worker routes, and road pressure, persistent Balanced, Secure, and Harvest policies for automated workers, location-aware Work/Storage/Patrol district rules that affect workers operating on marked tiles after Domain Stewardship, actionable Work district staffing advisories, a marked-tile district inspector, a save-persistent district performance ledger with first-use Field Notes and a bounded recent activity trail, and touch camera pan/pinch with visible zoom and recenter controls on compact viewports. The sprite sheet is registered through the texture manifest and falls back to an obvious placeholder when unavailable. The current victory condition ends the slice after the small cemetery is established; larger district simulation remains future work.

| Implemented foundation | Remaining direction |
| --- | --- |
| `src/ui.rs`, `src/ui/panels.rs`, `src/ui/responsive.rs`: world-first HUD, desktop panels, compact bottom sheet, and field-notes archive | Continue accessibility and large-text verification |
| `src/ui/world.rs`, `src/ui/world_feedback.rs`, and `src/ui/animation.rs`: full-world grid, actor interpolation, animation, complete route previews, and source feedback | Larger district simulation and multi-district route policies |
| `src/ui/components.rs`: title page, phase overlays, and grid picking | Keep the game name on the title page only |
| `src/main.rs`: neutral native window caption | Add touch camera gestures if the world outgrows one viewport |
| `src/game.rs`: camera, touch gestures, input, captures, and action dispatch | Expand the persistent colony model without resetting existing saves |
| `src/state.rs`: positioned buildings, research, zones, movement destinations, carried resources, and save migration | Extend stewardship policies beyond the current worker biases |
| `src/engine/progression.rs`, `src/engine/alerts.rs`, `src/engine/districts.rs`: research-driven capabilities, milestone, blockers, and district rules | Extend stewardship policies beyond the current worker biases |
| `assets/data/texture_manifest.json` and asset registry | Add later terrain, props, effects, and HUD assets as they become playable |

Keep simulation independent of camera zoom and panel visibility. UI should read a shared capability model derived from research; the simulation must validate those same capabilities when accepting orders. Do not implement technology progression only by hiding buttons.

Existing saves already receive deterministic defaults for positioned buildings, research, zones, and the necromancer. Future save changes need the same explicit migration policy; never silently reset a player's cemetery during the transition.

## Delivery sequence

1. **World-first UI foundation (implemented).** Dedicated title page; no in-session branding; full-screen cemetery; compact resource/status strip; contextual inspector; field-notes archive; collapsible feed and command dock. Verification covers 1280 × 720 title and desktop states plus 800 × 600 compact gameplay, pause, event, placement, research, colony, domain, notes, and victory scenes.
2. **One convincing cemetery scene (implemented for the slice).** Terrain and sprites cover the necromancer, skeleton, graves, trees, stockpile, shed, and lantern. Digging, hauling, construction progress, selection, camera transforms, route intent, and source feedback are readable.
3. **First research transition (implemented for the slice).** Binding Routines and the research chain gate repeat priorities, placement, work areas, logistics, and domain controls. Versioned saves preserve the new state with deterministic defaults.
4. **Movement and animation readability (implemented).** Workers visibly walk tile by tile, hauling has a real carried-resource phase, jobs expose procedural tool motion, and the necromancer walks, retargets, cancels, and pulses while ritualizing.
5. **First colony slice (implemented for the current slice).** The Ossuary Kiln completes a material → construction → production chain after Ossuary Logistics, workers route around basic obstructions before working, and painted Work, Storage, and Patrol areas select nearby destinations. The minimap and worker/grave cues provide first-pass domain feedback while keeping the original cemetery recognizable as part of the expanded settlement.
6. **Scale and domain controls (implemented for the current slice).** The settlement overview includes useful operational alerts with source selection, Domain Stewardship controls for district marks, worker routes, and road-pressure coverage, actionable Work district staffing guidance, marked-tile rule inspection, persistent Balanced, Secure, and Harvest policies that affect automated worker choices, additive Work, Storage, and Patrol rules that make marked districts operationally meaningful, and a save-persistent performance ledger with a bounded activity trail that records their real contribution. Future work can add deeper district simulation while validating worker selection and performance before growing the content set.
7. **Responsive command surface (implemented for the current slice).** Compact viewports use a full-window logical canvas, preserve the world above a bottom sheet, and expose the same touch-first actions for selection, management, events, pause, placement recovery, and field notes. Verification captures include the 800 × 600 composition and a 360 × 640 narrow-navigation stress state.
8. **Touch camera navigation (implemented for the current slice).** Compact viewports provide visible zoom and recenter controls plus gesture-aware map pan and pinch zoom. Claimed map gestures suppress accidental selection and command activation when the contact ends.

Defer trade, farms, housing, multiple biomes, and large defence systems until the first colony slice works. Art for later systems follows approved gameplay rather than committing production effort based solely on the reference.

## Review checklist

- Capture title, initial gameplay, selection, placement, research unlock, colony overview, pause, event, and victory/milestone screens. The name appears only on the title capture; check the native caption too.
- The initial screenshot reads as a small top-down cemetery; the later screenshot reads as a colony in the same world. Motion-focused scenes cover a worker walking, carrying, working, the necromancer walking and ritualizing, construction, and an active kiln.
- Early players see only relevant controls, and can select a grave, issue work, and raise a skeleton without opening a management dashboard.
- Later players can identify idle workers, missing materials, and suspicion sources without opening every building.
- World work remains readable with labels disabled; important statuses remain understandable without colour alone.
- Test UI click blocking, zoomed picking, drag-versus-click, placement cancellation, resize, compact bottom-sheet states, and large text.
- Verify research prerequisites in the simulation, save/load of unlocks, placements, destinations, and carried-resource defaults, and migration from an existing cemetery save.
- Keep existing job, corpse, suspicion, and progression checks passing while each related system changes.

The first implementation target is a complete playable cemetery screen with the current mechanics and the new visual hierarchy. It establishes the identity before expanding the simulation into a colony builder.
