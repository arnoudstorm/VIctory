# Phase 6: tests, synthetic benchmarks, telemetry and frame rates (780M)

Branch `laptop/780m` at `b81625a`, diagnostics build, Series S, Radeon
780M with 16 GB shared RAM. Runs automated, no keys (the title starts its
world by itself).

## Tests

`cargo test --workspace`: **527 passed, 0 failed** (523 this morning; +1
lazy list test, +3 tiled window tests).

## Synthetic benchmarks (`host::bench_tests`, run alone)

| Benchmark | Result |
|---|---|
| Small shared buffers (512 MB in each size) | card charged exactly the bytes at 64 KiB, 256 KiB and 4 MiB; process +535 / +294 / +264 MB; 410 / 532 / 1,420 us each to make |
| Memory left after 512 MB made and dropped, 3 rounds | 256, 24, 24 MB (no growth after the first) |
| Shared buffers given back | card +512 MB, 0 left; process +256 MB, 0 left |
| Submission and copy cost | 0.92 ms per waited submission; 34.4 GB/s between shared buffers |
| What host lists keep | empty list 1,328 KiB; allocator alone 11 KiB; list on a shared allocator 217 KiB; 235 KiB kept over a reset |
| Lazy lists (`lazy_list_tests`) | 242 KiB a list until opened, 1,338 KiB once opened |
| Window tiles (`window_probe_tests`) | untouched committed windows cost in full; reserved windows cost only mapped tiles; tiles on heaps opened on our own memory remove the device |

## Telemetry run `p6-telemetry` (laptop settings: lazy lists + tiled windows)

1-second samples (harness CSV: memory, paging, commit, GPU counters) and
VIctory's overlay line every second.

- **260 s in the world** (the earlier baseline, `res-2160`: 33 s before
  it ended out of memory). No out-of-memory and no device removal.
- **It ended differently:** at ~249 s the title stopped on a breakpoint
  exception (`0x80000003`) at `+0x5149aec` on many of its threads at
  once, and its own crash handler (Breakpad) wrote a crash log and a
  minidump; VIctory's network policy blocked the upload to
  `submit.backtrace.io`. A deliberate break in the title's code (an
  assert or fatal-error path); its cause is not known.
- Peak RAM 12.9 GB, peak card use 7.7 GB (budget ~5.7-5.9 GB):
  still over the budget, but the world kept running.

## Frame times (overlay lines; world = 500+ draws a frame)

| Run | Part | Lines | Mean | Median | P95 | P99 | Max |
|---|---|---|---|---|---|---|---|
| p6-telemetry (now) | world | 150 | 1,710 ms | 1,397 ms | 4,080 ms | 4,620 ms | 4,803 ms |
| p6-telemetry (now) | start screen | 65 | 146 ms | 33 ms | 591 ms | 3,143 ms | 3,143 ms |
| res-2160 (earlier) | world | 23 | 1,354 ms | 1,371 ms | 2,327 ms | 2,738 ms | 2,738 ms |
| res-2160 (earlier) | start screen | 57 | 48 ms | 34 ms | 54 ms | 733 ms | 733 ms |
| p5-poolc | world | 48 | 1,281 ms | 1,084 ms | 3,026 ms | 4,048 ms | 4,048 ms |

The earlier world sample is short (23 lines, before it ran out of memory),
so its tail is not comparable; the longer run reaches more of the world's
heavy scenes.

## Seconds at each frame rate

No 0-fps or no-frame periods were logged (every overlay line had a frame
rate above 0.2).

| fps | now (lazy+tiled) | earlier baseline | pool+runs |
|---|---|---|---|
| 0.2-0.3 | 35 | - | 4 |
| 0.3-0.4 | 33 | - | 6 |
| 0.4-0.5 | 34 | 5 | 5 |
| 0.5-0.6 | 29 | 4 | 2 |
| 0.6-0.7 | 34 | 5 | 5 |
| 0.7-0.8 | 27 | 7 | 6 |
| 0.8-0.9 | 18 | - | 6 |
| 0.9-1.0 | 27 | 6 | 11 |
| 1-2 | 30 | 7 | 17 |
| 2-3 | 1 | - | 4 |
| 4-6 | - | - | 2 |
| 13-14 | - | - | 1 |
| 15-25 | 4 | 6 | 3 |
| 25-29 | 4 | 16 | 2 |
| 29-30 | 8 | 7 | 9 |
| 30+ | 45 | 27 | 113 |
| **In all** | **329 s** | **90 s** | **196 s** |

Share of the "now" run: 81 % below 2 fps (267 of 329 s, the world), 17 %
at 25 fps or more (57 s, the start screen).
