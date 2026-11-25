# Launch the game with this branch's AMD RDNA3 settings (tested on an RX 7600 XT).
#
# Build first, once, from the repository root:
#   cargo build --release -p victory
#
# Then:
#   .\run-rdna3.ps1 -GameDir "D:\path\to\your\game"
#
# The switches below are the speed group from README / PR #134. VICTORY_RELAY
# removed the device on RDNA3 before the relay fixes merged here (#141, #191,
# #206); with them, one 3.5-minute run held at about 2x the frame rate of the
# defaults. If the driver resets, run with -Safe (no relay).
param(
    [Parameter(Mandatory = $true)][string]$GameDir,
    [switch]$Safe
)
$ErrorActionPreference = 'Stop'
$exe = Join-Path $PSScriptRoot 'target\release\victory.exe'
if (-not (Test-Path $exe)) {
    Write-Host "victory.exe not found: run 'cargo build --release -p victory' in $PSScriptRoot first."
    exit 1
}
$title = Get-ChildItem -Path $GameDir -Filter '*.exe' | Where-Object { $_.Name -like 'game_*' } | Select-Object -First 1
if (-not $title) {
    Write-Host "No game_*.exe found in $GameDir"
    exit 1
}
$env:VICTORY_GAME_DIR = $GameDir
$env:VICTORY_F16_NEAREST = '1'
$env:VICTORY_WIDE_BACK = '1'
$env:VICTORY_CARD_BUFFERS_MIN_MB = '1'
$env:VICTORY_CARD_BUFFERS_MB = '1024'
if (-not $Safe) { $env:VICTORY_RELAY = '2' }
& $exe run $title.FullName --execute
