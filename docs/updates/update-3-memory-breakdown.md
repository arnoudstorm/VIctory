# Update 3: where the card's memory goes (test 3)

- Commit: `d185ee6 + 256 KiB windows`
- What changed: back to 256 KiB windows; Windows' per-process GPU counters logged beside the overlay; synthetic tests of buffer and list costs
- Build: diagnostics, Series S, Radeon 780M, 16 GB

## What went wrong

- The world came about 1 min after the start screen (the fastest yet:
  1,910 pipelines from the library, 3 built), and **4 s later the device
  was reset** (0x887A0007, "the GPU will not respond to more commands")
  during a submission the relay had sent ahead (30 lists). **No
  out-of-memory error** was logged, as in test 2.
- At that moment the PC had **6 MB of physical memory free**; the card
  stood at 7.05 GB of its 6.05 GB budget. A device reset with the PC's
  memory used up and the card past its budget fits the system failing to
  make the card's memory resident; the relay (`VICTORY_RELAY=2`, sending
  submissions ahead) is the other candidate, since it reset AMD drivers
  before its fixes (AndreRese's RDNA3 notes). Test 4 runs without it.

## Where the card's memory goes

Measured, not estimated:

| Part | Size | How measured |
|---|---|---|
| Copies of title memory | 3,106 MB (2,516 MB of them windows) | VIctory's count (memory line) |
| Render targets and what the host writes | 1,025 MB | VIctory's count |
| Textures the title fills | 444 MB (2,590; the free-memory gate held 271 passes) | VIctory's count |
| Host command lists | 886 MB of process memory grew while 691 lists were made | VIctory's count; `bench_tests::what_host_lists_keep`: a list keeps ~1.3 MB of 20,000 commands over a reset |
| Per-buffer overhead of small shared buffers | **0 on the card**; ~33 KiB of process memory and ~0.4 ms each at 64 KiB | `bench_tests::cost_of_small_shared_buffers` (64, 256 KiB and 4 MiB: the card is charged exactly the bytes) |

- The two system views disagree: the budget query (what the overlay shows
  and what the budget is about) said 7.6 GB in use, while Windows'
  per-process counter "GPU Process Memory\Total Committed" said 3.4 GB at
  the same time. The budget query on this card counts more than the
  process's GPU allocations (it is the system's view of the shared
  segment); the run reports keep logging both.
- 256 KiB windows again: 5,247 windows, 2,516 MB (64 KiB: 12,322, 2,394 MB
  in test 2), with a quarter of the buffers.

## Timeline

| Time (UTC) | Event |
|---|---|
| 2026-11-25T02:35:28 | Game started |
| 2026-11-25T02:35:35 | Past the start screen |
| 2026-11-25T02:36:36 | World reached (500+ draws) |
| 2026-11-25T02:36:40 | GPU device removed |
| 2026-11-25T02:36:40 | VIctory gave up (host GPU failure) |

## Numbers at the end

- Overlay: `VIctory 0.9 fps 1113.7 ms (CPU 535.5, GPU wait 558.3) 3276 draws | RAM 12604 MB | VRAM 7585/6053 MB, shared 0 MB | buffers 8862 shared, 4778 upload | texture failures 0 | pipelines 1910 cached, 3 built`
- Card: `the card: 7051 MB in use of a budget of 6051 MB`
- Copies of title memory: `host copies of title memory: 8479 alive 3106 MB, of them 5247 windows outside buffers 2516 MB`
- Render targets: `host copies the host renders to or writes: 452 alive 1025 MB`
- Textures the title fills: `host copies the title fills: 2590 alive 444 MB`
- Texture passes held back for lack of room: `passes that made none for lack of room: 271`
- Textures picked by index: `textures picked by index: 4516 uploaded so far (273 MB), 20033 waiting`
- PC: `11979 MB private (6423 MB of it in memory; the PC has 6 MB free and 5609 MB left to commit)`

## Errors and warnings, grouped

| Count | First at | Message (numbers taken out) |
|---|---|---|
| 9 | 2026-11-25T02:35:45 | `WARN victory_gpu::draw: a texture picked by index changed after its upload with nothing to tell of it: uploaded again slot=N address=0x… bytes=N frames_after_upload=N looks=N changed=N` |
| 2 | 2026-11-25T02:35:59 | `WARN victory_gpu::queue: a black frame after a frame that was not frame=N resource=0x… state=0x… draws=N copied=true black_frames=N levels=[N, N, N, N, N, N, N, N, N, N, N, N]` |
| 2 | 2026-11-25T02:35:59 | `WARN victory_gpu::queue: levels after the black frame frame=N levels=[N, N, N, N, N, N, N, N, N, N, N, N]` |
| 1 | 2026-11-25T02:35:29 | `WARN victory_hle::modules::kernel: a host DLL outside the host policy: refused dll="rpcrt4.dll"` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::shader_cache: VICTORY_SHADER_LOOP_LIMIT: translated loops stop sooner limit=N` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877e1b0, 904b14630, 1c, 904b1464` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877e1b0, 904b141d0, 1c, 904b141e` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877e1b0, 904b141d0, 1c, 904b1431` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877e1b0, 904b14630, 1c, 904b158d` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877e1b0, 904b141d0, 1c, 904b1448` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877e1b0, 904b14630, 1c, 904b14c7` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877e1b0, 904b16574, 1c, 904b171f` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877f440, 904b18104, N, 904b18288` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877ee10, 904d33838, N, 904d339bc` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877ee10, 9053ec428, N, 9053ec5ac` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877ee10, 9053ed574, N, 9053ed6f8` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877ee10, 904d30138, N, 904d302bc` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877ee10, 904d330bc, N, 904d33240` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877ee10, 904d311d8, N, 904d3135c` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877eea0, 904c1bda8, N, 904c1bf2c` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877eea0, 904baebe8, N, 904baed6c` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877eea0, 904c8a9a4, 18c, 904cc3f` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877eea0, 904cf9328, 18c, 904cfab` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877eea0, 904c1d410, N, 904c5c550` |
| 1 | 2026-11-25T02:35:32 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([29ab877eea0, 904c2a6f4, 18c, 904c62c` |

## The first error and what followed

```
2026-11-25T02:36:40.218905Z ERROR victory_gpu::host: when the host device went: the submitting thread was preparing a submission (nothing of ours was running on the card), of 26 lists, for 1.4 ms; pipeline library: no call running; its file
  34427.0 ms before, thread 18888: pipeline library: loaded [mesh 1911a12654d5fc2a, pixel f91ee51aedf0b7ba] in 0.2 ms
  34396.4 ms before, thread 16532: pipeline library: loaded [mesh 1da5c48360826443, pixel a0715f5563c5a8ee] in 0.6 ms
  34396.1 ms before, thread 29108: pipeline library: loaded [mesh 21bd00836e1dfae5, pixel a88fe11942146689] in 0.2 ms
  34396.1 ms before, thread 10904: pipeline library: loaded [mesh 21bd00836e1dfae5, pixel a88fe11942146689] in 0.2 ms
  34369.9 ms before, thread 28640: pipeline library: loaded [mesh e2bfb4680f41ceec, pixel 3c62430c3db42328] in 0.3 ms
  34369.9 ms before, thread 31708: pipeline library: loaded [mesh e2bfb4680f41ceec, pixel 3c62430c3db42328] in 0.3 ms
  34349.6 ms before, thread 15728: pipeline library: loaded [mesh 411454b37e58d4ed, pixel 0dd773b4aec5f1db] in 0.6 ms
  34270.7 ms before, thread 11912: pipeline library: loaded [mesh 93b71ecaee341239, pixel 41a3abc010f0e04d] in 0.7 ms
  34270.7 ms before, thread 18888: pipeline library: loaded [mesh 93b71ecaee341239, pixel 41a3abc010f0e04d] in 0.7 ms
  34269.5 ms before, thread 10904: pipeline library: loaded [mesh ff2afa5e9a460ff0, pixel 3084ebdc38f9b9ec] in 0.5 ms
  34269.4 ms before, thread 16532: pipeline library: loaded [mesh ff2afa5e9a460ff0, pixel 3084ebdc38f9b9ec] in 0.5 ms
  34269.4 ms before, thread 29108: pipeline library: loaded [mesh ff2afa5e9a460ff0, pixel 3084ebdc38f9b9ec] in 0.5 ms
  34269.4 ms before, thread 28640: pipeline library: loaded [mesh ff2afa5e9a460ff0, pixel 3084ebdc38f9b9ec] in 0.5 ms
  34253.4 ms before, thread 15728: pipeline library: loaded [mesh 7eefb872d56bb1d8, pixel 8c5512c9589ff899] in 0.3 ms
```

## DRED (device removal record)

```
2026-11-25T02:35:37.661591Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 2 title copies 0.0 MB; 1 refreshes 0.0 MB, 0 skipped 0.0 MB; 3 runs 0.0 ms; 0 queries 0.00 ms, moves 0.13 ms, 0.00 MB unmapped; 9940 page faults; 4 
2026-11-25T02:35:42.761715Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 9 title copies 0.0 MB; 8 refreshes 7.1 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.16 ms, 0.00 MB unmapped; 1082 page faults; 1 
2026-11-25T02:35:47.862093Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 9 title copies 0.0 MB; 8 refreshes 4.6 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.06 ms, 0.00 MB unmapped; 867 page faults; 4 w
2026-11-25T02:35:52.961883Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.4 MB, 0 skipped 0.0 MB; 1 runs 0.0 ms; 0 queries 0.00 ms, moves 0.05 ms, 0.03 MB unmapped; 279 page faults; 5 w
2026-11-25T02:35:58.062145Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.4 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.08 ms, 0.00 MB unmapped; 0 page faults; 5 wat
2026-11-25T02:36:03.170278Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 9 title copies 0.0 MB; 9 refreshes 6.1 MB, 0 skipped 0.0 MB; 4 runs 0.0 ms; 0 queries 0.00 ms, moves 0.05 ms, 0.05 MB unmapped; 755 page faults; 10 
2026-11-25T02:36:08.205564Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 10 title copies 0.0 MB; 9 refreshes 8.4 MB, 0 skipped 0.0 MB; 13 runs 0.0 ms; 0 queries 0.00 ms, moves 0.10 ms, 0.00 MB unmapped; 250 page faults; 2
2026-11-25T02:36:13.495484Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 13 title copies 0.0 MB; 12 refreshes 9.3 MB, 0 skipped 0.0 MB; 9 runs 0.0 ms; 0 queries 0.00 ms, moves 0.11 ms, 0.01 MB unmapped; 1824 page faults; 
2026-11-25T02:36:18.562342Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 22 title copies 0.1 MB; 12 refreshes 9.1 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.19 ms, 0.00 MB unmapped; 3801 page faults; 
2026-11-25T02:36:36.892477Z  INFO victory_gpu::copystat: copies per frame: 4.6 submissions; 227 title copies 0.2 MB; 375 refreshes 329.8 MB, 0 skipped 0.0 MB; 5 runs 0.0 ms; 0 queries 0.00 ms, moves 59.63 ms, 2.68 MB unmapped; 25053 page fa
2026-11-25T02:36:40.219255Z ERROR victory_gpu::host: host GPU device removed after 1 submissions sent ahead (VICTORY_RELAY=1) of 30 lists; no breadcrumbs (The specified device interface or feature level is not supported on this system. (0x8
VIctory: host GPU failure: host GPU device removed after 1 submissions sent ahead (VICTORY_RELAY=1) of 30 lists; no breadcrumbs (The specified device interface or feature level is not supported on this system. (0x887A0004)); device removed:
```

## Settings

- `--console series-s`
- `VICTORY_WINDOW_KB=256`
- `VICTORY_LATE_WAIT_LISTS=96`
- `VICTORY_RELAY=2`
- `VICTORY_TEXTURE_MIN_FREE_MB=1536 (default)`
- `VICTORY_INPLACE=1`
- `VICTORY_SHADER_LOOP_LIMIT=64`
