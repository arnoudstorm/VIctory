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

1. **Age out windows that only descriptor slots hold** (memory, crashes):
   a window no list has used for N frames and no slot read through a
   table goes; a later read makes it again from title memory (it is a
   copy). Expected: most of the 2.6 GB held by slots.
2. **Shorten the submission lock** (frame time): record outside it and
   take it only to submit.
3. **Small-block churn** (RAM, CPU): pool the sub-4 KiB allocations of
   recording.
4. **Texture backlog**: upload the slots draws read first (needs knowing
   which indices shaders read; today the most recently written go first).

Changes made so far are listed in `docs/report-radeon-780m.md` and in the
commit log of this branch.
