# Audit: graphics, memory and performance on a Radeon 780M

Branch `laptop/780m`. Measured on a Ryzen 7 8840HS / Radeon 780M (RDNA3,
integrated, Direct3D budget about 6 GB of 16 GB shared RAM), the title's
Series X|S build 1.0.2.1 in Series S mode, diagnostics build, with the
settings in `docs/report-radeon-780m.md` plus `VICTORY_RELAY=2`,
`VICTORY_WIDE_REFRESH=1`, `VICTORY_WIDE_BACK=1`, `VICTORY_JOIN_COPIES=1`,
`VICTORY_WINDOW_KB=256`, `VICTORY_WINDOW_TAILS=1`. Every number below is
from a run's own log (the 5-second lines `copystat`, `memory`, `card's
clock`), one minute into the world unless said otherwise. Offsets, counts
and VIctory's messages only (docs/asset-policy.md).

## What the problems come down to

| Problem | Measured | Root cause |
|---|---|---|
| Runs end ~1 min into the world | card 7.0-7.8 GB of a 6.0 GB budget, then E_OUTOFMEMORY and device removal | copies of title memory kept alive (below) |
| Low frame rate | 1.2-6.8 s per world frame; card 4-36 % busy | CPU-side copying of title memory and waiting on one submission lock |
| Excessive RAM | process private 7.3 GB in the menus, 13.2 GB one minute into the world; 944 MB of commit left | write-combined copies (4.0 GB), driver allocations (4.7 GB), 0.9 M small heap blocks |
| Missing textures | 23,580 slots of textures picked by index queued, 21,877 due, oldest 3,860 frames; 1,466 uploaded (568 MB) | upload pass bounded per frame; the backlog (~9 GB at the average size) cannot fit the budget |
| Missing geometry | 813 draws and 360 indirect draws a frame dropped | 436 pipelines still being built (first run with new settings); counted and logged, not hidden |

## Memory

Process memory one minute into the world (memory map line):

| Kind | Committed | In RAM |
|---|---|---|
| title heaps (console addresses) | 2,942 MB | 155 MB |
| views of the title's pages | 6,765 MB | 155 MB |
| write-combined buffers shared with the card (copies of title memory) | 4,000 MB | 44 MB |
| other private allocations of 1 MB and more (driver) | 4,652 MB | 262 MB |
| process heap segments | 1,668 MB | 300 MB |

- **Windows of title memory are kept by descriptor slots.** On the 256 KiB
  grid, 5,318 windows (2,577 MB) are shown by descriptor slots and 5,348
  held by lists; only 1,192 MB of their pages were ever copied. A window
  stays alive for as long as any slot of the title's bindless heap shows
  it, and the title leaves views in the heap long after it stops reading
  them: nothing ages such windows out. This is an unbounded cache and the
  main reason the card's budget fills.
- On the 4 MiB grid the same was 652 windows, 2,800 MB; on 256 KiB, 5,199
  windows, 2,408 MB: a finer grid saves ~0.4 GB, not the whole.
- VIctory's own heap: 1,063 MB in 935,521 blocks, 921,320 of them under
  4 KiB (143 MB).
- Upload buffers made: 2,039 (1,073 MB); shared buffers made: 10,523
  (4,877 MB) in five minutes: steady churn.

## Time per world frame

| Run | Frame | CPU preparing | of it copies | GPU | Card busy |
|---|---|---|---|---|---|
| defaults (4 MiB windows) | 6,814 ms | 6,080 ms | 6,409 ms (1,383 MB) | 330 ms | 4-15 % |
| wide refresh/back, join copies | 2,292 ms | 918 ms | 315 ms (615 MB) | 470 ms | 36 % |
| + relay (`VICTORY_RELAY=2`) | 1,195 ms | 617 ms | 171 ms (357 MB) | 399 ms | 28-31 % |

- Before the wide settings, a frame refreshed 16 GB of copies (37,418
  refreshes), took 1.9 M page faults and checked 10.4 GB with the write
  watch (9.0 GB unchanged).
- One recording thread waits **1,049 ms a frame for the submission lock**
  in the relay run: recording on six threads serialises there.

## Textures

- The pass for textures picked by index stages 192 MB / 250 ms a frame
  (three times that with a backlog over 1,024 slots), most recently
  written slots first. In the world: 64 uploads recorded and 234 left out
  a frame; 412 slots passed over because their memory is not mapped.
- With the card at its budget the pass stops making textures
  (`resource::room`); in the runs measured it stopped on time first.

## Validation done

- `cargo test --workspace` on `laptop/780m`, GPU and shader tests on the
  780M itself: 511 passed, 0 failed, 8 ignored (before the overlay); the
  GPU crate after it: 120 passed, 1 ignored.
- Debug layer, GPU-based validation and DRED exist as switches
  (`VICTORY_D3D12_DEBUG`, `VICTORY_D3D12_GBV`, `VICTORY_D3D12_DRED`); runs
  with them are slower than the title's watchdog allows without
  `VICTORY_NO_WATCHDOG`.

## Fixes, ranked by measured effect

A first ranking put two fixes on top that the code and the logs rule out:

- **Not a fix: ageing out windows that descriptor slots hold.** Shaders
  that pick buffers by a computed index can read any buffer view of the
  heap, so `heap_buffers` keeps every view's copy valid and refreshes it
  each frame. A window freed under a slot would leave that slot null and
  such shaders reading zeros: a silent wrong result. The 2.6 GB is the
  cost of copying title memory for bindless reads, not a leak. What
  removes it is using title memory in place (in place covers the
  windows in private memory; the page pool's views are scattered).
- **Not a fix: the "submission lock" wait.** The 1,049 ms a frame counted
  there is the time from a submission's call to the start of its
  preparing, which includes waiting for flights to land. Of it, 897 ms is
  "title memory read": a submission reads title memory that a submission
  in flight writes (8.5 times a frame). That wait keeps what the title
  reads correct. In the same frames the relay thread waited 921 ms for
  the card with nothing to land: the 780M runs for most of each 1.2 s
  world frame. The world is GPU-bound now.

What remains, by effect:

1. **GPU time per frame** (frame rate): find the submissions and shaders
   that take the card's ~0.9 s a frame (`VICTORY_GPU_TIMING=1` names the
   slowest commands per list) and fix the costly translations.
2. **Small-block churn** (RAM, CPU): 921,320 blocks under 4 KiB.
3. **Texture backlog**: upload the slots draws read first (needs knowing
   which indices shaders read; today the most recently written go first).
4. **Memory for bindless copies**: more title memory in place (the page
   pool's views), which needs placed buffers that span views.

## Synthetic measurements (no game data)

`crates/victory-gpu/src/host.rs`, `bench_tests` (commit `c6acf1d`), run on
the 780M with
`cargo test -p victory-gpu --lib bench_tests -- --ignored --test-threads=1 --nocapture`:

| Test | Result | Status |
|---|---|---|
| 64 x 8 MB buffers shared with the CPU, half dropped, half retired | card +512 MB, 0 MB left; 0 parked in the retired list (32 dropped at once) | measured, passes |
| the same 512 MB made and dropped three times | process private left over: 256, 256, 256 MB | measured: driver reuse, not a leak |
| a retired object while its submission runs | kept until that submission finished, dropped after | measured, passes |
| one-copy submissions, each waited for | 1.106 ms each | measured |
| 256 MB copy between shared buffers | 24.1 GB/s (11.1 ms) | measured |

What follows from them:

- Resource lifetimes on the shared-buffer and retirement paths hold: no
  leak, nothing parked past its submission.
- At the 30-49 submissions a world frame makes, round trips cost about
  35-55 ms of a 1,200 ms frame: submission overhead is not what limits
  the frame rate; the GPU work is.
- Not measured here: behaviour at the card's budget (filling 6 GB of the
  PC's memory on purpose would take the rest of the system with it), and
  the GPU time of the title's own shaders (needs the game).

## Test suite (Phase 4)

| After | Passed | Failed | Ignored |
|---|---|---|---|
| `f53e10e` (baseline before this round, as recorded) | 511 | 0 | 8 |
| `6d19463` (fxceless `c96b4f2`) | 515 | 0 | 8 |
| `c6acf1d` (synthetic benchmarks, ignored by default) | 515 | 0 | 12 |

## Settings for 16 GB and a Radeon 780M

In the run script (outside the repository), with why; **not yet run with
the game**, so their effect in the world is unverified:

| Setting | Value | Why |
|---|---|---|
| `VICTORY_TEXTURE_ROOM_PERCENT` | 75 (instead of `VICTORY_TEXTURE_BUDGET_MB=1536`) | a texture-only cap ignores the buffers and targets that share the ~6 GB budget, and it overrides the share check; 75 % of the whole budget keeps ~1.5 GB free |
| `VICTORY_TEXTURE_STAGE_MB` | 96 | half the default staging per frame (3x during a backlog): a lower memory peak |
| `VICTORY_COMPILE_HEADROOM_MB` | 2048 | the world runs had 0.9-1.9 GB of commit left |
| frame capture | off unless `-Capture` | each saved frame is a GPU read-back |
| `-Release` | release build | no 5-second diagnostics lines; scripted keys need the diagnostics build |

Kept from before (measured): Series S, `VICTORY_RELAY=2`, wide refresh and
copy-back, joined copies, 256 KiB windows with tails, `VICTORY_INPLACE=1`,
`VICTORY_CARD_BUFFERS_MB=256`, `VICTORY_SHADER_LOOP_LIMIT=64`,
`VICTORY_F16_NEAREST=1`, `VICTORY_NO_WATCHDOG=1`.

## Unresolved

- Runs end about a minute into the world on the card's budget; the copies
  for bindless reads (~2.4-2.6 GB) are needed as long as title memory is
  copied, not used in place.
- Missing textures: the backlog is larger than the budget.
- The world's GPU time (~0.9 s a frame) is not broken down by shader yet.

Changes made so far are listed in `docs/report-radeon-780m.md` and in the
commit log of this branch.
