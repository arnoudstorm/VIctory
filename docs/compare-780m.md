# laptop/780m against upstream, forks and pull requests (2026-11-25)

Status words: **merged** (in that repository's main), **proposed** (an open
pull request), **on this branch** (taken here, with this branch's tests),
**tested here** (tests or runs on the Radeon 780M recorded in
`docs/audit-780m.md`), **unverified** (not run here).

## Where the branch stands

| Ref | Commit | Date | Common ancestor with laptop/780m | laptop/780m ahead / behind |
|---|---|---|---|---|
| laptop/780m (before this round) | `f53e10e` | 2026-11-25 | – | – |
| Marvisak/VIctory main | `940d32f` | 2026-11-25 | `ffe310e` (2026-11-24) | 40 / 319 |
| fxceless/victory-gta6 main | `c96b4f2` | 2026-11-24 | `a803c87` (2026-11-24) | 25 / 1 |
| AndreRese/victory-fork main | `aa1d77f` | 2026-11-24 | `aa1d77f` | 14 / 0 |

The AndreRese repository is gone from GitHub ("Repository not found" on
2026-11-25); its commits are kept in this branch's history.

## What the branch is made of

| Source | Commits | Status |
|---|---|---|
| fxceless/victory-gta6 main up to `a803c87` | base | merged there |
| AndreRese main `aa1d77f`: fxceless PRs #134, #135, #141, #191, #203, #206 | merge `cfd6125` | proposed in fxceless (open PRs), on this branch, tested here (511 tests) |
| upstream PR #454 (watchdog hook), `67dbd11`, `75d2a79` | cherry-picked | proposed upstream, on this branch, tested here (runs) |
| 1984742628/VIctory `24a387d` (Series S PresentX planes) | `675bb84`, one conflict resolved by hand | not proposed upstream, on this branch, tested here (runs: menus drawn) |
| own: page pool in place (`92c0f07`), compile headroom gate (`174cb2a`), overlay (`308fefc`), docs | own | on this branch, tested here |

## Candidates since the branch was made

| Change | Where | Status | Relevance here |
|---|---|---|---|
| `c96b4f2`: depth bounds test; relay, wide copy-back and card copies on by default; textures picked by index uploaded while loading and before the decoration bake; one host copy per placed texture (no duplicate for views that differ only in filler words); `VICTORY_TEXTURE_ROOM_PERCENT` | fxceless main | merged there; **on this branch as `6d19463`** (applied without conflicts); test suite 515 passed, 0 failed; not yet run with the game here | correctness (depth bounds), duplicate allocations, a knob for the card's budget |
| `cd94d35`, `940d32f`: shaders mark the pages they write through views; copy-backs compare only those | upstream main | merged upstream, unverified here | copy-back cost; upstream base, needs a port |
| #595 / #1828 (asynchronous submissions, present waits for the previous frame) | upstream | proposed; tested here on an upstream base: one new test fails on the 780M | the relay on this branch covers the same ground |
| #1296 / #1617 (texture mip drop) | upstream | closed / proposed; unverified here | does not apply to this base's lazy host textures (see audit) |
| #1849 (free memory not watched, a Wine deadlock) | upstream | proposed | Wine only |
| xcl-dev/xcl (title memory as heaps without copies, commit-headroom compiles) | separate project, GPL-2.0-only | ideas only | the headroom idea is reimplemented here (`174cb2a`) |
