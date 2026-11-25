# Update 2: free-memory texture gate with DRED (test 2)

- Commit: `d185ee6`
- What changed: textures picked by index also wait while the PC has under 1.5 GB of physical memory free; D3D12 DRED on
- Build: diagnostics, Series S, Radeon 780M, 16 GB

## What went wrong

- **Progress over test 1:** the run got past loading into the world (world
  at 02:30:25, about 2 min after start) and drew up to 3,847 draws a frame
  for about a minute, with **no out-of-memory error at all** (test 1: out
  of memory while loading). The gate held texture uploads back 2,484
  times while the PC had 0.5-1.4 GB free.
- **How it ended:** the device was **reset** (DXGI_ERROR_DEVICE_RESET,
  0x887A0007, "the GPU will not respond to more commands") during a
  submission the relay had sent ahead: one list, its last operation a
  dispatch 8x1x1 of compute program `55679a0547649d0b`. DRED recorded no
  page fault, so it was not a read or write of an address the card did not
  have.
- **Two candidates, not told apart by this log:**
  1. **The card far past its budget**: 7.7 GB in use of 6.07 GB. The
     system then has to move the card's memory to the PC's and back to run
     a submission; with 0.6 GB of the PC's memory free that can fail and
     reset the device.
  2. **That compute program**: the same program was in the 2.4 s
     submission that hit Windows' 2 s timeout in an earlier run (before
     TdrDelay was raised). With TdrDelay at 60 s a long dispatch alone
     should not reset the device.
- **Where the card's memory went** at the end: copies of title memory
  3.0 GB (2.4 GB of them windows, 12,322 of them on the 64 KiB grid),
  render targets 1.2 GB, textures the title fills 0.4 GB (the gate kept
  them low), and **about 3.1 GB not in VIctory's own counts** (driver
  objects: pipelines, command lists, descriptor heaps, and the overhead of
  20,818 shared buffers made in the run).
- **Cost of the gate:** 22,259 textures picked by index waiting, 1,160
  uploaded (427 MB): many surfaces stay without their texture.
- Next: break the 3.1 GB down (the system's per-process GPU allocation
  counters), go back to 256 KiB windows (half the objects for +0.2 GB),
  and time the compute program on its own.

## Timeline

| Time (UTC) | Event |
|---|---|
| 2026-11-25T02:28:11 | Game started |
| 2026-11-25T02:28:19 | Past the start screen |
| 2026-11-25T02:30:25 | World reached (500+ draws) |
| 2026-11-25T02:31:16 | GPU device removed |
| 2026-11-25T02:31:16 | VIctory gave up (host GPU failure) |

## Numbers at the end

- Overlay: `VIctory 0.4 fps 2422.4 ms (CPU 588.9, GPU wait 1737.3) 3847 draws | RAM 13123 MB | VRAM 7651/6073 MB, shared 0 MB | buffers 20818 shared, 1747 upload | texture failures 0 | pipelines 1883 cached, 27 built`
- Card: `the card: 7706 MB in use of a budget of 6068 MB`
- Copies of title memory: `host copies of title memory: 15373 alive 3012 MB, of them 12322 windows outside buffers 2394 MB`
- Render targets: `host copies the host renders to or writes: 533 alive 1188 MB`
- Textures the title fills: `host copies the title fills: 1189 alive 384 MB`
- Texture passes held back for lack of room: `passes that made none for lack of room: 2484`
- Textures picked by index: `textures picked by index: 1160 uploaded so far (427 MB), 22259 waiting`
- PC: `13219 MB private (3954 MB of it in memory; the PC has 639 MB free and 4442 MB left to commit)`

## Errors and warnings, grouped

| Count | First at | Message (numbers taken out) |
|---|---|---|
| 200 | 2026-11-25T02:28:27 | `WARN victory_gpu::draw: a texture picked by index changed after its upload with nothing to tell of it: uploaded again slot=N address=0x… bytes=N frames_after_upload=N looks=N changed=N` |
| 42 | 2026-11-25T02:31:16 | `WARN victory_gpu::host: no shared buffer error=The GPU device instance has been suspended. Use GetDeviceRemovedReason to determine the appropriate action. (0x…) size=N` |
| 4 | 2026-11-25T02:28:42 | `WARN victory_gpu::queue: a black frame after a frame that was not frame=N resource=0x… state=0x… draws=N copied=true black_frames=N levels=[N, N, N, N, N, N, N, N, N, N, N, N]` |
| 4 | 2026-11-25T02:28:42 | `WARN victory_gpu::queue: levels after the black frame frame=N levels=[N, N, N, N, N, N, N, N, N, N, N, N]` |
| 1 | 2026-11-25T02:28:11 | `WARN victory_hle::modules::kernel: a host DLL outside the host policy: refused dll="rpcrt4.dll"` |
| 1 | 2026-11-25T02:28:15 | `WARN victory_gpu::shader_cache: VICTORY_SHADER_LOOP_LIMIT: translated loops stop sooner limit=N` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f825a62d60, 904b147c0, 1c, 904b1490` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f825a62d60, 904b147c0, 1c, 904b147d` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f825a62d60, 904b14c20, 1c, 904b14c3` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f825a62d60, 904b147c0, 1c, 904b14a7` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f825a62d60, 904b14c20, 1c, 904b1526` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f825a62d60, 904b14c20, 1c, 904b15ec` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f825a62d60, 904b160b8, 1c, 904b179c` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753cf0, 904b186f4, N, 904b18878` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753f90, 904d33e28, N, 904d33fac` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753f90, 9053eca18, N, 9053ecb9c` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753f90, 9053edb64, N, 9053edce8` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753f90, 904d30728, N, 904d308ac` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753f90, 905319aec, N, 90531a0ac` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753f90, 904d317c8, N, 904d3194c` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753e40, 904c1c398, N, 904c1c51c` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753e40, 904baf1d8, N, 904baf35c` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753e40, 904c8af94, 18c, 904cc45` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753e40, 904cf9918, 18c, 904cfb1` |
| 1 | 2026-11-25T02:28:16 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1f807753e40, 904c1da00, N, 904c5cb40` |

## The first error and what followed

```
2026-11-25T02:31:16.812869Z ERROR victory_gpu::host: when the host device went: the submitting thread was preparing a submission (nothing of ours was running on the card), of 1 lists, for 17.1 ms; pipeline library: make of f24e90bbce20bb208
  36828.9 ms before, thread 25144: sweep: 1 host copies of textures the title fills let go (64 KiB), each held by its title texture alone (no list, no slot); the first: title texture 0x1f98559e090
  35773.8 ms before, thread 25144: sweep: 1 host copies of textures the title fills let go (64 KiB), each held by its title texture alone (no list, no slot); the first: title texture 0x1f9a04e70e0
  34761.4 ms before, thread 25144: sweep: 98 host copies of windows of title memory let go (9856 KiB), held by no list and no slot
  34711.0 ms before, thread 25144: sweep: 91 host copies of textures the title fills let go (5824 KiB), each held by its title texture alone (no list, no slot); the first: title texture 0x1f91c665840
  33615.5 ms before, thread 25144: sweep: 275 host copies of windows of title memory let go (33344 KiB), held by no list and no slot
  32569.1 ms before, thread 25144: sweep: 109 host copies of windows of title memory let go (14080 KiB), held by no list and no slot
  32524.4 ms before, thread 25144: sweep: 13 host copies of textures the title fills let go (3776 KiB), each held by its title texture alone (no list, no slot); the first: title texture 0x1f93b1697c0
  31551.4 ms before, thread 23992: pipeline library: made [vertex 58d926085fca4999, pixel edc1cad8ffb79bc9] in 23.1 ms (the library did not load it: The parameter is incorrect. (0x80070057))
  31516.3 ms before, thread 23992: pipeline library: saved 924034 bytes, renamed over the library's file
  31437.6 ms before, thread 25144: sweep: 25 host copies of windows of title memory let go (16640 KiB), held by no list and no slot
  31261.3 ms before, thread 32848: pipeline library: made [vertex 58d926085fca4999, pixel edc1cad8ffb79bc9] in 6.1 ms (the library did not load it: The parameter is incorrect. (0x80070057))
  31231.8 ms before, thread 37156: pipeline library: made [vertex ca5fa787aecf4e22, pixel 1092022f53279ee7] in 13.7 ms (the library did not load it: The parameter is incorrect. (0x80070057))
  31231.3 ms before, thread 29792: pipeline library: made [vertex 5ec7617b0f7cb43c, pixel 813a7ead7f09ebd7] in 10.0 ms (the library did not load it: The parameter is incorrect. (0x80070057))
  30259.0 ms before, thread 25144: sweep: 94 host copies of windows of title memory let go (15232 KiB), held by no list and no slot
```

## DRED (device removal record)

```
2026-11-25T02:28:21.766641Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 1 title copies 0.0 MB; 1 refreshes 0.0 MB, 0 skipped 0.0 MB; 2 runs 0.0 ms; 0 queries 0.00 ms, moves 0.14 ms, 0.00 MB unmapped; 13518 page faults; 3
2026-11-25T02:28:27.001559Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 6 title copies 0.0 MB; 4 refreshes 3.0 MB, 0 skipped 0.0 MB; 2 runs 0.0 ms; 0 queries 0.00 ms, moves 0.20 ms, 0.00 MB unmapped; 11409 page faults; 3
2026-11-25T02:28:32.100580Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.4 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.10 ms, 0.00 MB unmapped; 6100 page faults; 5 
2026-11-25T02:28:37.233464Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.4 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.05 ms, 0.00 MB unmapped; 7033 page faults; 5 
2026-11-25T02:28:42.366135Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.6 MB, 0 skipped 0.0 MB; 1 runs 0.0 ms; 0 queries 0.00 ms, moves 0.10 ms, 0.00 MB unmapped; 8326 page faults; 5 
2026-11-25T02:28:47.433133Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 11 title copies 0.0 MB; 10 refreshes 9.4 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.05 ms, 0.00 MB unmapped; 5287 page faults; 
2026-11-25T02:28:52.499870Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 11 title copies 0.0 MB; 10 refreshes 9.2 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.03 ms, 0.00 MB unmapped; 8241 page faults; 
2026-11-25T02:28:57.599684Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 10 title copies 0.0 MB; 9 refreshes 7.9 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.03 ms, 0.00 MB unmapped; 6463 page faults; 7
2026-11-25T02:29:02.682785Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 13 title copies 0.0 MB; 12 refreshes 9.7 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.05 ms, 0.00 MB unmapped; 8769 page faults; 
2026-11-25T02:29:07.850433Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 11 title copies 0.0 MB; 10 refreshes 9.1 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.04 ms, 0.00 MB unmapped; 7610 page faults; 
2026-11-25T02:29:12.965418Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 11 title copies 0.0 MB; 10 refreshes 7.9 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.04 ms, 0.00 MB unmapped; 7234 page faults; 
2026-11-25T02:29:18.858796Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 13 title copies 0.0 MB; 11 refreshes 9.4 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.05 ms, 0.03 MB unmapped; 8450 page faults; 
2026-11-25T02:29:24.109265Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 17 title copies 0.0 MB; 11 refreshes 8.9 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.12 ms, 0.00 MB unmapped; 9474 page faults; 
2026-11-25T02:29:29.334567Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 14 title copies 0.0 MB; 10 refreshes 8.0 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.08 ms, 0.00 MB unmapped; 5392 page faults; 
2026-11-25T02:29:34.729563Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 11 refreshes 9.2 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.06 ms, 0.00 MB unmapped; 1063 page faults; 
2026-11-25T02:29:40.339883Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 10 refreshes 8.9 MB, 0 skipped 0.0 MB; 4 runs 0.0 ms; 0 queries 0.00 ms, moves 0.14 ms, 0.09 MB unmapped; 1957 page faults; 
2026-11-25T02:29:45.761990Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 10 refreshes 8.1 MB, 0 skipped 0.0 MB; 3 runs 0.0 ms; 0 queries 0.00 ms, moves 0.15 ms, 0.87 MB unmapped; 1422 page faults; 
2026-11-25T02:29:50.898530Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 10 refreshes 9.1 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.08 ms, 0.00 MB unmapped; 263 page faults; 8
2026-11-25T02:29:56.065074Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 10 refreshes 9.1 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.06 ms, 0.00 MB unmapped; 204 page faults; 8
2026-11-25T02:30:01.165142Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 11 title copies 0.0 MB; 10 refreshes 8.0 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.06 ms, 0.00 MB unmapped; 290 page faults; 7
2026-11-25T02:30:06.351439Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 14 title copies 0.0 MB; 11 refreshes 9.2 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.08 ms, 0.00 MB unmapped; 336 page faults; 8
2026-11-25T02:30:25.362979Z  INFO victory_gpu::copystat: copies per frame: 5.2 submissions; 402 title copies 0.2 MB; 857 refreshes 750.3 MB, 0 skipped 0.0 MB; 10 runs 0.0 ms; 0 queries 0.00 ms, moves 163.47 ms, 5.02 MB unmapped; 45611 page 
2026-11-25T02:30:31.628459Z  INFO victory_gpu::copystat: copies per frame: 44.3 submissions; 10285 title copies 6.3 MB; 26627 refreshes 16845.2 MB, 0 skipped 0.0 MB; 214 runs 0.2 ms; 0 queries 0.00 ms, moves 1989.33 ms, 103.01 MB unmapped; 
2026-11-25T02:30:36.694812Z  INFO victory_gpu::copystat: copies per frame: 33.8 submissions; 9185 title copies 5.4 MB; 33164 refreshes 14102.2 MB, 0 skipped 0.0 MB; 53 runs 0.1 ms; 0 queries 0.00 ms, moves 291.04 ms, 33.98 MB unmapped; 1190
2026-11-25T02:30:43.830745Z  INFO victory_gpu::copystat: copies per frame: 42.8 submissions; 9198 title copies 5.4 MB; 33237 refreshes 15454.3 MB, 0 skipped 0.0 MB; 57 runs 0.1 ms; 0 queries 0.00 ms, moves 178.01 ms, 14.76 MB unmapped; 5558
2026-11-25T02:30:49.992645Z  INFO victory_gpu::copystat: copies per frame: 49.0 submissions; 9208 title copies 5.9 MB; 27799 refreshes 14671.8 MB, 0 skipped 0.0 MB; 62 runs 0.2 ms; 0 queries 0.00 ms, moves 503.46 ms, 34.90 MB unmapped; 2407
2026-11-25T02:30:56.478745Z  INFO victory_gpu::copystat: copies per frame: 50.0 submissions; 8878 title copies 5.7 MB; 26537 refreshes 16154.5 MB, 0 skipped 0.0 MB; 82 runs 0.2 ms; 0 queries 0.00 ms, moves 501.01 ms, 17.80 MB unmapped; 2839
2026-11-25T02:31:01.892090Z  INFO victory_gpu::copystat: copies per frame: 38.3 submissions; 5292 title copies 4.1 MB; 20925 refreshes 8523.2 MB, 0 skipped 0.0 MB; 47 runs 0.1 ms; 0 queries 0.00 ms, moves 179.24 ms, 25.15 MB unmapped; 42270
2026-11-25T02:31:07.551225Z  INFO victory_gpu::copystat: copies per frame: 41.3 submissions; 9357 title copies 6.2 MB; 24056 refreshes 12757.5 MB, 0 skipped 0.0 MB; 58 runs 0.0 ms; 0 queries 0.00 ms, moves 195.66 ms, 45.52 MB unmapped; 6897
2026-11-25T02:31:13.294890Z  INFO victory_gpu::copystat: copies per frame: 54.2 submissions; 14400 title copies 8.5 MB; 30158 refreshes 18991.0 MB, 0 skipped 0.0 MB; 66 runs 0.1 ms; 0 queries 0.00 ms, moves 216.03 ms, 41.25 MB unmapped; 154
2026-11-25T02:31:16.813151Z ERROR victory_gpu::host: host GPU device removed after 1 submissions sent ahead (VICTORY_RELAY=1) of 1 lists; breadcrumbs (done/count, list) []; list 0 of 1, of its 8: dispatch 8x1x1 [compute 55679a0547649d0b] | 
VIctory: host GPU failure: host GPU device removed after 1 submissions sent ahead (VICTORY_RELAY=1) of 1 lists; breadcrumbs (done/count, list) []; list 0 of 1, of its 8: dispatch 8x1x1 [compute 55679a0547649d0b] | 55679a0547649d0b: u4=0xa01
```

## Settings

- `--console series-s`
- `VICTORY_D3D12_DRED=1`
- `VICTORY_TEXTURE_MIN_FREE_MB=1536 (default)`
- `VICTORY_TEXTURE_ROOM_PERCENT=75`
- `VICTORY_WINDOW_KB=64`
- `VICTORY_LATE_WAIT_LISTS=96`
- `VICTORY_RELAY=2`
- `VICTORY_INPLACE=1`
- `VICTORY_SHADER_LOOP_LIMIT=64`
