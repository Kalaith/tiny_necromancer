<#
.SYNOPSIS
    Headless screenshot harness for Tiny Necromancer.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (TINY_NECROMANCER_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Named scenes reset runtime state before each capture.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @(
        "menu",
        "gameplay",
        "placement",
        "research",
        "orders",
        "priority-route",
        "route-policy",
        "route-policy-wait",
        "colony",
        "domain",
        "harvest-domain",
        "storage-domain",
        "storage-slots",
        "storage-full",
        "wood-slots",
        "patrol-gap",
        "work-gap",
        "work-overlap",
        "work-locked",
        "work-overlap-locked",
        "work-domain",
        "route-blocked",
        "route-domain",
        "district-route-gap",
        "notes",
        "production",
        "kiln-supply",
        "kiln-supply-loose",
        "kiln-supply-inspector",
        "worker-walking",
        "worker-haul-planned",
        "multi-source",
        "worker-carrying",
        "worker-working",
        "necromancer-walking",
        "necromancer-ritual",
        "building",
        "kiln",
        "paused",
        "event",
        "victory",
        "zoomed"
    ),
    [int]$Frames = 150,
    [int]$WindowWidth = 0,
    [int]$WindowHeight = 0,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$workspace = Split-Path -Parent $gameDir
if (-not (Test-Path (Join-Path $workspace "macroquad-toolkit"))) {
    $workspace = Split-Path -Parent $workspace
}
$shared = Join-Path $workspace "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Prefix "TINY_NECROMANCER" -Scenes $Scenes -Frames $Frames -WindowWidth $WindowWidth -WindowHeight $WindowHeight -OutputDir $OutputDir -SkipBuild:$SkipBuild
