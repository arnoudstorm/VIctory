# Radeon 780M laptop branch: today's update (2026-11-25)

**Status: paused. I'm continuing in 4-7 hours.** Branch `laptop/780m` on
`arnoudstorm/VIctory`, pull request #1 into the fork's `main`.

## Where it stands

- **Start-screen flicker fixed** (`7d0ed69`): brightness jumps 67 -> 22.
  Not a photosensitivity certification.
- **Startup reliable, no more "Not responding"**: 3 of 3 startups reach
  the start screen; 0 s "Not responding" after the Project Flamingo
  stand-in window port (was 11-26 s per run).
- **The world runs much longer**: with lazy command lists and tiled memory
  copies, 180-260 s in the world per run instead of 33 s.
- **Closest to gameplay yet**: holding A skips the intro and reaches the
  gameplay loading screen, but that load runs out of memory (video
  memory peaks 6.5-7.8 GB against a ~5.8 GB budget).
- **Tests:** 536 passed, 0 failed.

## What changed today

| Commit | Change | Default |
|---|---|---|
| `7d0ed69` | Start-screen flicker fix | on |
| `00b6f89` | Lazy host command lists (`VICTORY_LAZY_LISTS`), pooled preludes (`VICTORY_PRELUDE_POOL`) | opt-in; lazy lists on in the laptop run script |
| `8b5c8bf` | Tiled memory copies (`VICTORY_TILED_WINDOWS`) | opt-in; on in the laptop run script |
| `fc4aa27`, `514eb21` | Flamingo ports: page unmaps in one call, stand-in window | on |
| `d556a15`, `af43097`, `9810434`, `a862e67` | Flamingo fixes: half floats, sampler, audio, input callbacks | on |

## Gameplay tests (`gameplay-tests-2026-11-25.md`)

| Test | Setting | Result |
|---|---|---|
| 1 | baseline | intro sequence, 330 s |
| 2 | relay off | intro sequence, 330 s |
| 3 | texture share 60 % | device removed at 269 s |
| 4 | pooled preludes | intro sequence, furthest of 1-4 |
| 5 | pooled preludes + relay off + hold A | **intro skipped**, gameplay load ran out of memory at 132 s |

## Today's documents

| Document | What it covers |
|---|---|
| `final-report-2026-11-25.md` | The whole session, with a hand-over list |
| `audit-2026-11-25.md` | Memory and stability audit |
| `gameplay-tests-2026-11-25.md` | The five gameplay tests |
| `flicker-audit.md` | Start-screen flicker: cause, fix, measurements |
| `stability-780m.md` | Startup and menu stability runs |
| `resolution-hdr-780m.md` | Resolution, HDR and presentation |
| `design-tiled-windows.md` | Tiled memory copies: design, probes, results |
| `phase4-scene-transitions.md` | The start-screen to world transition |
| `compare-flamingo-2026-11-25.md` | Project Flamingo: what it fixed, what was ported |
| `phase6-tests-telemetry-fps.md` | Tests, benchmarks, frame-rate tables |

A Word version of the audit exists outside the repository: the asset
policy refuses binary files, and an exception for it was not added.

## Next session

1. Get the gameplay load through within memory: close every other
   program, then keep cutting what the load holds (copies of game memory,
   render targets).
2. Resolve PR #1's conflicts with the fork's `main`.
3. Look at the title's own breakpoint at `+0x5149aec` and the refused mesh
   pipeline.
