# Audit: what keeps the game from running smoothly on the Radeon 780M (2026-11-25)

Branch `laptop/780m` at `fdba2d2`. This is an audit only: nothing in it
has been fixed yet. The measurements come from today's harness runs
(`fsr-start`, `ffx-builtin`, `ffx-every`, `ffx-dump`; logs and CSVs in
the session notes, not in the repo).

## Where the game is now

With `VICTORY_FFX_BUILTIN=1` (opt-in, commit `1272861`), the game
reaches **gameplay**: the control prompt appears, the camera moves, and
the 3D world is on screen. In `ffx-every` the world rendered continuously
for 5 min 40 s until the harness stopped it. There was no black frame
after the cut into the world, no device removal and no "Not responding".

It is **not smooth and not fully correct**: 1.4-1.6 fps in the world
(645-717 ms per frame), anything the sun does not hit is near black, and
free RAM is down to 6-7 MB.

## The blockers, most important first

| # | Blocker | Effect | Evidence | Size of the fix |
|---|---|---|---|---|
| 1 | **The CPU and GPU wait on each other** | most of the frame time | Per frame (`ffx-every`, end of run): 683 ms in total. The submitting thread waits 387 ms for the card, but the card is only about 57 % busy. There are 16 forced syncs ("settles") per frame, all for "title memory read": the title reads back memory the GPU wrote, so each read waits for all submitted work. CPU 284 ms per frame | large: let reads that do not need fresh GPU results skip the wait, or keep that data GPU-only |
| 2 | **CPU-side checks of shader writes** | CPU time, memory | 403 MB per frame snapshotted and compared on the CPU (390 ranges of title buffer copies, 370 MB of it in copies of 1 MiB or more). The log suggests `VICTORY_CARD_BUFFERS_MIN_MB=1` (compare on the card instead); not measured | medium: try the existing switch first, one run |
| 3 | **Texture refreshes** | CPU time | 55.6 ms per frame reading, hashing and detiling title textures, plus 158 hash passes and 38 compare passes on the card per frame | medium |
| 4 | **Memory is exhausted** | runs end or slow down, paging | Private memory 12.3-12.5 GB of 16 GB, free RAM falls to 5-7 MB, up to 153,612 pages read in per second. The card's budget is 5,758 MB and VIctory uses 7,061 MB, about 1.3 GB over. `fsr-start` ended by itself at 190 s with 5 MB free. Earlier: copies of title memory 2.4-3.7 GB, render targets 1.0-1.2 GB, textures 0.3-1.2 GB, command lists ~0.9 GB (`final-report-2026-11-25.md`) | large: Flamingo `a79befd`/`d549a33` (views of title memory in place) are the known candidates, not ported |
| 5 | **Ray-traced lighting is off** | anything the sun does not hit is black (interiors, the shadowed side of people and buildings) | 308 dropped dispatches per frame, all "a stage of the ray-tracing structure builder, which is off" (`frame-1609-ops.txt`). `docs/fork-ray-tracing.md`: the title's indirect light and lamp and sun shadows are ray traced in its own compute programs. `VICTORY_BVH_BUILDER=1` exists, but some builder pipelines are refused (they ask for 64-lane waves) or do not translate, and 5 shadow-mask trace programs do not translate | very large, multi-session; correctness, not speed |
| 6 | **The upscaler is only a stopgap** | softer picture, opt-in only | FSR4's 20 compute programs are console machine code in `ACCC`/`FSPX` parts, which do not translate. `VICTORY_FFX_BUILTIN=1` replaces them with a bilinear pass (`fsr4-black-output.md`). Off by default, so the default run still shows a black world | small to make it the laptop default; large to get real temporal upscaling (upstream `0b35b7f` uses AMD's PC FidelityFX DLL, which is not on this machine) |
| 7 | **Smaller rendering defects** | visual errors | Dark rectangles floating above the yard, blocky low-resolution cloud patches (frames 1867, 2059, 2155 of `ffx-every`), some shots washed out to near white (frame 2505 of `ffx-builtin`). Causes not checked | unknown until a dump is read |
| 8 | **One mesh pipeline refused by the driver** | its draws are missing | `mesh f5e50eb2108c425b` + `pixel ce3f0432a08f1d35`, "The parameter is incorrect" (0x80070057), every run. Flamingo `d3ebb55` may fix it | small to medium |
| 9 | **Missing methods** | only matter without the switch | Pipeline state slot 0x58 and command list slot 0x1a8 (`ClearUnorderedAccessViewUint`), both called by the FSR4 DLL, which no longer loads with the switch. Upstream `0b35b7f` has 0x1a8/0x1b0 | small |
| 10 | **Slow start, one crash seen earlier** | waiting before play | In `ffx-every` the first world frame came 82 s after launch (12:01:21 to 12:02:43). `final-report`: the title's own crash handler fired once after about 4 minutes in the world (breakpoint at title `+0x5149aec`, not investigated) | medium |

## Frame-time breakdown (one world frame, `ffx-every`)

| Part | ms per frame |
|---|---|
| Whole frame (overlay) | 645-717 (1.4-1.6 fps) |
| CPU (overlay) | ~260-295 |
| Submitting thread waiting for the card | ~387 |
| Card idle while our thread was working | ~238 |
| Texture refreshes, all threads | ~56 |
| Relay thread landing copies back | ~129 (beside the submitting thread) |
| Draws per frame | ~5,480 |

## What to do first (suggested order, one change per run)

1. Make `VICTORY_FFX_BUILTIN=1` part of the laptop run, so the world is
   visible by default on this machine (#6). This is a `run-laptop.ps1`
   change, which needs your OK because its defaults are kept as they are.
2. One run with `VICTORY_CARD_BUFFERS_MIN_MB=1` and compare the frame
   time and CPU ms (#2).
3. Find which title reads cause the 16 waits per frame (#1). The log
   already counts them by cause; next, name the memory ranges.
4. Memory (#4): port Flamingo `a79befd` + `d549a33` and measure the peak.
5. Ray tracing (#5): one run with `VICTORY_BVH_BUILDER=1` to see which
   builder pipelines are still refused.

## Tests

`cargo test --workspace`: 543 passed, 0 failed (536 before today's
upscaler change).
