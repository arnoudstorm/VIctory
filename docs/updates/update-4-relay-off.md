# Update 4: relay off (test 4)

- Commit: `e69ecc5`
- What changed: VICTORY_RELAY=0: every submission waited for (AndreRese's -Safe), all else as test 3
- Build: diagnostics, Series S, Radeon 780M, 16 GB

## What went wrong

- Without the relay the run ended at **the same point** as test 3: the
  world at 02:39:32, the end **5 s later**. This time as out-of-memory
  (7 lines, then "no host list for a state prelude"), not as a device
  reset. The PC had 370 MB free; the card stood at 7.0 GB of 6.05 GB.
- Frame times in the world without the relay: 1.3-1.5 s at 2,200-2,800
  draws (with it, test 3: 1.1 s at 3,276).
- **Conclusion from tests 2-4:** the relay is not what ends the runs. The
  device reset of tests 2 and 3 and the out-of-memory of test 4 are two
  faces of the same thing: when the world's first scenes stream in, the
  PC's 16 GB runs out (6-370 MB free) while the card's share of it is
  1.0-1.6 GB past its budget. Which of the two errors shows depends on
  which allocation fails first.
- What the world needs here at that moment, measured over tests 2-4:
  copies of title memory ~3.1 GB, render targets ~1.0 GB, textures the
  title fills 0.4-0.5 GB (held back by the gate), host command lists
  ~0.9 GB, the process ~12-13 GB private in all.

## What can still change it

- **Not settings:** window grid, list counts, texture budgets and the
  relay have each been measured (updates 1-4 and `docs/audit-780m.md`);
  none moves the end by more than a few hundred MB.
- **Code:** using the title's memory in place instead of copying it
  (~3 GB). On this base only private memory and single views can be
  opened as heaps; the page pool's scattered views need one heap per
  view and a buffer per heap, which is a larger change.
- **Hardware:** 32 GB of RAM doubles the card's share; AndreRese's RDNA3
  notes list 32 GB as a requirement for the world.

## Timeline

| Time (UTC) | Event |
|---|---|
| 2026-11-25T02:38:23 | Game started |
| 2026-11-25T02:38:28 | Past the start screen |
| 2026-11-25T02:39:32 | World reached (500+ draws) |
| 2026-11-25T02:39:37 | First out-of-memory (0x8007000E) |
| 2026-11-25T02:39:37 | VIctory gave up (host GPU failure) |

## Numbers at the end

- Overlay: `VIctory 0.7 fps 1462.9 ms (CPU 783.9, GPU wait 547.1) 2758 draws | RAM 12644 MB | VRAM 7688/6051 MB, shared 0 MB | buffers 8817 shared, 4907 upload | texture failures 0 | pipelines 1904 cached, 2 built`
- Card: `the card: 7043 MB in use of a budget of 6046 MB`
- Copies of title memory: `host copies of title memory: 8398 alive 3127 MB, of them 5154 windows outside buffers 2537 MB`
- Render targets: `host copies the host renders to or writes: 452 alive 1025 MB`
- Textures the title fills: `host copies the title fills: 2721 alive 501 MB`
- Texture passes held back for lack of room: `passes that made none for lack of room: 227`
- Textures picked by index: `textures picked by index: 4620 uploaded so far (334 MB), 20437 waiting`
- PC: `11966 MB private (3263 MB of it in memory; the PC has 370 MB free and 5637 MB left to commit)`

## Errors and warnings, grouped

| Count | First at | Message (numbers taken out) |
|---|---|---|
| 9 | 2026-11-25T02:38:38 | `WARN victory_gpu::draw: a texture picked by index changed after its upload with nothing to tell of it: uploaded again slot=N address=0x… bytes=N frames_after_upload=N looks=N changed=N` |
| 6 | 2026-11-25T02:39:37 | `WARN victory_gpu::host: no shared buffer error=Not enough memory resources are available to complete this operation. (0x…) size=N` |
| 2 | 2026-11-25T02:38:53 | `WARN victory_gpu::queue: a black frame after a frame that was not frame=N resource=0x… state=0x… draws=N copied=true black_frames=N levels=[N, N, N, N, N, N, N, N, N, N, N, N]` |
| 2 | 2026-11-25T02:38:54 | `WARN victory_gpu::queue: levels after the black frame frame=N levels=[N, N, N, N, N, N, N, N, N, N, N, N]` |
| 1 | 2026-11-25T02:38:23 | `WARN victory_hle::modules::kernel: a host DLL outside the host policy: refused dll="rpcrt4.dll"` |
| 1 | 2026-11-25T02:38:25 | `WARN victory_gpu::shader_cache: VICTORY_SHADER_LOOP_LIMIT: translated loops stop sooner limit=N` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1de64cf00a0, 504b141d0, 1c, 504b1431` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1de64cf00a0, 504b14630, 1c, 504b14c7` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1de64cf00a0, 504b141d0, 1c, 504b141e` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1de64cf00a0, 504b14630, 1c, 504b1464` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1de64cf00a0, 504b141d0, 1c, 504b1448` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1de64cf00a0, 504b14630, 1c, 504b158d` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1de64cf00a0, 504b16574, 1c, 504b171f` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf410, 504b18104, N, 504b18288` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf470, 504d33838, N, 504d339bc` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf470, 5053ec428, N, 5053ec5ac` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf470, 5053ed574, N, 5053ed6f8` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf470, 504d30138, N, 504d302bc` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf470, 5053a5298, N, 5053a541c` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf470, 504d311d8, N, 504d3135c` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf650, 504c1bda8, N, 504c1bf2c` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf650, 504baebe8, N, 504baed6c` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf650, 504c8a9a4, 18c, 504cc3f` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf650, 504cf9328, 18c, 504cfab` |
| 1 | 2026-11-25T02:38:26 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1dd8ecaf650, 504c1d410, N, 504c5c550` |

## Before the first out-of-memory

```
2026-11-25T02:39:36.987238Z  INFO victory_gpu::pipeline: first bind pipeline=0x1dee25d81d0 frame=1432 stage="pixel" size=2612 hash=2edbd011d7110758
2026-11-25T02:39:36.987181Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df10555280 frame=1432 stage="pixel" size=2624 hash=16400b2f082a2006
2026-11-25T02:39:36.988791Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df10556d30 frame=1432 stage="mesh" size=3760 hash=0f50af1be3633c5d
2026-11-25T02:39:36.988858Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df10556d30 frame=1432 stage="pixel" size=2140 hash=c0167939dc389778
2026-11-25T02:39:36.989704Z  INFO victory_gpu::pipeline: first bind pipeline=0x1deeefb9a80 frame=1432 stage="mesh" size=3388 hash=734a975814cb5e78
2026-11-25T02:39:36.989736Z  INFO victory_gpu::pipeline: first bind pipeline=0x1deeefb9a80 frame=1432 stage="pixel" size=2508 hash=48e1424d47797b18
2026-11-25T02:39:36.990434Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df0faf9440 frame=1432 stage="mesh" size=4168 hash=f607be37ee4fba77
2026-11-25T02:39:36.990461Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df0faf9440 frame=1432 stage="pixel" size=2144 hash=8b24403f37708a44
2026-11-25T02:39:36.994523Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df101a5530 frame=1432 stage="mesh" size=15252 hash=ff2afa5e9a460ff0
2026-11-25T02:39:36.994558Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df102841d0 frame=1432 stage="mesh" size=3764 hash=4c1c0bc01e07c903
2026-11-25T02:39:36.994580Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df102841d0 frame=1432 stage="pixel" size=2624 hash=16400b2f082a2006
2026-11-25T02:39:36.994574Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df101a5530 frame=1432 stage="pixel" size=2196 hash=3084ebdc38f9b9ec
2026-11-25T02:39:37.011566Z  WARN victory_gpu::host: no shared buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=3932160
2026-11-25T02:39:37.012045Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df101a2dd0 frame=1432 stage="mesh" size=4188 hash=21bd00836e1dfae5
2026-11-25T02:39:37.012090Z  INFO victory_gpu::pipeline: first bind pipeline=0x1df101a2dd0 frame=1432 stage="pixel" size=2216 hash=a88fe11942146689
```

## The first error and what followed

```
2026-11-25T02:39:37.016927Z ERROR victory_gpu::host: no host list for a state prelude; a diagnostics build with VICTORY_D3D12_DEBUG=1 names the command
VIctory: host GPU failure: no host list for a state prelude; a diagnostics build with VICTORY_D3D12_DEBUG=1 names the command
2026-11-25T02:39:37.018149Z  WARN victory_gpu::host: no shared buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=3670016
2026-11-25T02:39:37.018843Z  WARN victory_gpu::host: no shared buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=2621440
2026-11-25T02:39:37.019774Z  WARN victory_gpu::host: no shared buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=1835008
```

## DRED (device removal record)

```
2026-11-25T02:38:31.430626Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 2 title copies 0.0 MB; 1 refreshes 0.0 MB, 0 skipped 0.0 MB; 3 runs 0.0 ms; 0 queries 0.00 ms, moves 0.15 ms, 0.00 MB unmapped; 10609 page faults; 4
2026-11-25T02:38:36.763734Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 10 title copies 0.0 MB; 9 refreshes 8.0 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.11 ms, 0.00 MB unmapped; 791 page faults; 1 
2026-11-25T02:38:41.830635Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 9 title copies 0.0 MB; 9 refreshes 5.1 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.03 ms, 0.00 MB unmapped; 509 page faults; 3 w
2026-11-25T02:38:46.930698Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.4 MB, 0 skipped 0.0 MB; 1 runs 0.0 ms; 0 queries 0.00 ms, moves 0.04 ms, 0.03 MB unmapped; 616 page faults; 5 w
2026-11-25T02:38:52.030356Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.4 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.06 ms, 0.00 MB unmapped; 31 page faults; 5 wa
2026-11-25T02:38:57.147956Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 9 title copies 0.0 MB; 9 refreshes 5.9 MB, 0 skipped 0.0 MB; 4 runs 0.0 ms; 0 queries 0.00 ms, moves 0.05 ms, 0.05 MB unmapped; 735 page faults; 10 
2026-11-25T02:39:02.210884Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 10 title copies 0.0 MB; 9 refreshes 8.4 MB, 0 skipped 0.0 MB; 13 runs 0.0 ms; 0 queries 0.00 ms, moves 0.10 ms, 0.00 MB unmapped; 268 page faults; 2
2026-11-25T02:39:07.264349Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 11 refreshes 9.2 MB, 0 skipped 0.0 MB; 10 runs 0.0 ms; 0 queries 0.00 ms, moves 0.14 ms, 0.32 MB unmapped; 1602 page faults;
2026-11-25T02:39:12.363669Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 23 title copies 0.1 MB; 12 refreshes 9.1 MB, 0 skipped 0.0 MB; 1 runs 0.0 ms; 0 queries 0.00 ms, moves 0.16 ms, 0.07 MB unmapped; 4300 page faults; 
2026-11-25T02:39:32.347630Z  INFO victory_gpu::copystat: copies per frame: 4.6 submissions; 240 title copies 0.2 MB; 381 refreshes 317.5 MB, 0 skipped 0.0 MB; 6 runs 0.0 ms; 0 queries 0.00 ms, moves 51.65 ms, 2.44 MB unmapped; 24864 page fa
```

## Settings

- `--console series-s`
- `VICTORY_RELAY=0`
- `VICTORY_WINDOW_KB=256`
- `VICTORY_LATE_WAIT_LISTS=96`
- `VICTORY_TEXTURE_MIN_FREE_MB=1536 (default)`
- `VICTORY_INPLACE=1`
- `VICTORY_SHADER_LOOP_LIMIT=64`
