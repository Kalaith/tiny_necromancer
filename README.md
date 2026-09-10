# Tiny Necromancer

Tiny Necromancer is a touch-first cemetery management vertical slice. Start
with one grave and one skeleton, then turn bones into labour, construction,
research, and a small hidden settlement while keeping suspicion below the
investigation threshold.

## How to play

Tap or click **NEW GAME**. In the cemetery, tap a grave, worker, building, or
ground tile to select it, then use the visible inspector and command-dock
buttons to give orders. Building placement includes a visible
**CANCEL PLACEMENT** button, and **PAUSE**, **RESUME**, **SAVE**, and **LOAD** are
available as on-screen controls. Desktop players may also right-drag to pan
and use the mouse wheel to zoom.

The authored loop is:

1. Assign a skeleton to dig graves and haul loose bones.
2. Raise more workers, including a Brute from a Notable corpse remnant.
3. Build the Work Shed and Grave Lantern, then expand the usable plots.
4. Study the research chain to unlock repeat priorities, placement, work areas,
   logistics, and domain controls.
5. Establish the settlement before suspicion reaches the investigation limit.

## Development

This project uses Rust 2021, Macroquad 0.4, and the sibling
`macroquad-toolkit` path dependency. Game data is embedded from
`assets/data/` through the toolkit data loader; the runtime uses the toolkit
for asset management, persistence, camera transforms, virtual UI, pointer
input, notifications, event dispatch, deterministic RNG, and screenshot
capture.

Run the focused checks from this directory:

```powershell
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
.\publish.ps1
```

Refresh the visual verification captures with:

```powershell
.\scripts\capture_ui.ps1
```

Captures belong directly in `docs/verification/`. The root
`catalog_thumbnail.png` is the title-screen capture used by the catalog.

## Project documentation

- [`gdd.md`](gdd.md) — implemented rules, scope, and future milestone intent.
- [`docs/visual-direction-and-ui-plan.md`](docs/visual-direction-and-ui-plan.md) — visual hierarchy and roadmap.
- `AGENTS.md`, `CODE_STANDARDS.md`, `GAME_DEVELOPMENT_GUIDE.md`, and
  `MACROQUAD_TOOLKIT.md` — managed RustGames standards copied from
  `rust_management/docs`; do not hand-edit these project copies.
