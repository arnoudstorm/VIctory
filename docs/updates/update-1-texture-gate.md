# Update 1: texture gate run (test 1)

- Commit: `14bbc4b`
- What changed: no new copies of textures picked by index while the card is past its budget; 64 KiB windows; 96 lists in the air
- Build: diagnostics, Series S, Radeon 780M, 16 GB

## What went wrong

- The run ended **while loading the world**, before its first frame: the
  host driver could not close a command list (E_OUTOFMEMORY) and VIctory
  stopped ("host GPU failure").
- It was **not the card's budget** this time: 3.6 GB in use of 7.5 GB. It
  was **physical memory**: the PC had 0.5-1.3 GB free in the last minute
  (commit still had 9.4 GB of room). A driver for a card that shares the
  PC's memory needs free pages for command buffers.
- What filled it: textures uploaded **while loading** (fxceless `c96b4f2`,
  "textures picked by index are uploaded while loading"): 9,929 uploads,
  1,296 MB, and 7,378 copies of textures the title fills alive (1,538 MB)
  before the world's first frame. In the run before (`run-2026-11-25-0316`)
  the same copies were 0.8 GB after a minute in the world.
- The texture gate of this commit was never reached ("passes that made none
  for lack of room: 0"): the card stayed under its budget.
- Next (test 2): the same gate also holds these copies back while the PC's
  free memory is short, and DRED records the device removal.

## Timeline

| Time (UTC) | Event |
|---|---|
| 2026-11-25T02:20:58 | Game started |
| 2026-11-25T02:21:06 | Past the start screen |
| 2026-11-25T02:22:59 | First out-of-memory (0x8007000E) |
| 2026-11-25T02:22:59 | VIctory gave up (host GPU failure) |

## Numbers at the end

- Overlay: `VIctory 1.2 fps 858.0 ms (CPU 73.8, GPU wait 12.0) 6 draws | RAM 9130 MB | VRAM 4079/7542 MB, shared 0 MB | buffers 10 shared, 11472 upload | texture failures 0 | pipelines 1876 cached, 0 built`
- Card: `the card: 3615 MB in use of a budget of 7542 MB`
- Copies of title memory: `host copies of title memory: 9 alive 187 MB, of them 10 windows outside buffers 14 MB`
- Render targets: `host copies the host renders to or writes: 88 alive 359 MB`
- Textures the title fills: `host copies the title fills: 7378 alive 1538 MB`
- Texture passes held back for lack of room: `passes that made none for lack of room: 0`
- Textures picked by index: `textures picked by index: 9929 uploaded so far (1296 MB), 16033 waiting`
- PC: `8749 MB private (4767 MB of it in memory; the PC has 996 MB free and 9435 MB left to commit)`

## Errors and warnings, grouped

| Count | First at | Message (numbers taken out) |
|---|---|---|
| 46 | 2026-11-25T02:21:14 | `WARN victory_gpu::draw: a texture picked by index changed after its upload with nothing to tell of it: uploaded again slot=N address=0x… bytes=N frames_after_upload=N looks=N changed=N` |
| 4 | 2026-11-25T02:22:59 | `WARN victory_gpu::host: no upload buffer error=Not enough memory resources are available to complete this operation. (0x…) size=N` |
| 2 | 2026-11-25T02:21:29 | `WARN victory_gpu::queue: a black frame after a frame that was not frame=N resource=0x… state=0x… draws=N copied=true black_frames=N levels=[N, N, N, N, N, N, N, N, N, N, N, N]` |
| 1 | 2026-11-25T02:20:58 | `WARN victory_hle::modules::kernel: a host DLL outside the host policy: refused dll="rpcrt4.dll"` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::shader_cache: VICTORY_SHADER_LOOP_LIMIT: translated loops stop sooner limit=N` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492847650, 504b147c0, 1c, 504b1490` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492847650, 504b147c0, 1c, 504b14a7` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492847650, 504b14c20, 1c, 504b1526` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492847650, 504b14c20, 1c, 504b14c3` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492847650, 504b147c0, 1c, 504b14e2` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492847650, 504b14c20, 1c, 504b15ec` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492847650, 504b16b64, 1c, 504b177e` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492aaea30, 504b186f4, N, 504b18878` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492ab8eb0, 504d33e28, N, 504d33fac` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492ab8eb0, 5053eca18, N, 5053ecb9c` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492ab8eb0, 5053edb64, N, 5053edce8` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492ab8eb0, 504d30728, N, 504d308ac` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492ab8eb0, 504d336ac, N, 504d33830` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492ab8eb0, 504d317c8, N, 504d3194c` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492aae9d0, 504c1c398, N, 504c1c51c` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492aae9d0, 504b1a290, N, 504b7ada0` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492aae9d0, 504c8af94, 18c, 504cc45` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492aae9d0, 504cf9918, 18c, 504cfb1` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492aae9d0, 504c1da00, N, 504c5cb40` |
| 1 | 2026-11-25T02:21:02 | `WARN victory_gpu::device: a pipeline creation method that is not implemented: the title goes without this pipeline slot=0x… calls=N description=Some([1d492aae9d0, 504c2ace4, 18c, 504c632` |

## Before the first out-of-memory

```
2026-11-25T02:22:59.045721Z  INFO victory_gpu::pipeline: first bind pipeline=0x1d609783e10 frame=2092 stage="mesh" size=6184 hash=fdd457cb5a6f39a8
2026-11-25T02:22:59.045755Z  INFO victory_gpu::pipeline: first bind pipeline=0x1d609783e10 frame=2092 stage="pixel" size=8836 hash=663d1c4f50206212
2026-11-25T02:22:59.051473Z  INFO victory_gpu::descriptor::host: unordered access view of a depth texture resource=0x1d59966efe0 view=Some([56, 4, 0, 0, 0, 0, 0, 5, 2786065216, 5])
2026-11-25T02:22:59.051519Z  INFO victory_gpu::descriptor::host: unordered access view of a depth texture resource=0x1d59966efe0 view=Some([62, 4, 0, 1, 0, 0, 0, 5, 2786065216, 5])
2026-11-25T02:22:59.055832Z  INFO victory_gpu::descriptor::host: unordered access view of a depth texture resource=0x1d59966efe0 view=Some([56, 4, 0, 0, 0, 0, 0, 5, 2786065216, 5])
2026-11-25T02:22:59.058832Z  INFO victory_gpu::descriptor::host: unordered access view of a depth texture resource=0x1d5bd6d2090 view=Some([41, 4, 0, 0, 13, 0, 0, 5, 2786065216, 5])
2026-11-25T02:22:59.059672Z  INFO victory_gpu::list: a texture to buffer copy (a read back) copies=3 made=Ok(()) source=Some("5x1 f2 mips 1") source_kind=0 destination_kind=1 destination_offset=0 destination_bytes=Some(80)
2026-11-25T02:22:59.060189Z  INFO victory_gpu::list: a texture to buffer copy (a read back) copies=4 made=Ok(()) source=Some("1x1 f2 mips 1") source_kind=0 destination_kind=1 destination_offset=0 destination_bytes=Some(16)
2026-11-25T02:22:59.062702Z  INFO victory_gpu::pipeline: first bind pipeline=0x1d4b81c1800 frame=2092 stage="compute" size=2668 hash=bf59ac28a62312e6
2026-11-25T02:22:59.072587Z  INFO victory_gpu::pipeline: first bind pipeline=0x1d49e290c00 frame=2092 stage="compute" size=1996 hash=93fb87bdc7295d2b
2026-11-25T02:22:59.074319Z  INFO victory_gpu::pipeline: first bind pipeline=0x1d49e042900 frame=2092 stage="compute" size=2428 hash=3a3c2b3f2364d207
2026-11-25T02:22:59.074754Z  INFO victory_gpu::pipeline: first bind pipeline=0x1d49e4d2440 frame=2092 stage="compute" size=1704 hash=ca7946cd9fb13845
2026-11-25T02:22:59.086786Z ERROR victory_gpu::host: a host command list did not close (Not enough memory resources are available to complete this operation. (0x8007000E)): a command recorded into it was invalid; a diagnostics build with VI
VIctory: host GPU failure: a host command list did not close (Not enough memory resources are available to complete this operation. (0x8007000E)): a command recorded into it was invalid; a diagnostics build with VICTORY_D3D12_DEBUG=1 names 
2026-11-25T02:22:59.089525Z  WARN victory_gpu::host: no upload buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=4194304
```

## The first error and what followed

```
2026-11-25T02:22:59.086786Z ERROR victory_gpu::host: a host command list did not close (Not enough memory resources are available to complete this operation. (0x8007000E)): a command recorded into it was invalid; a diagnostics build with VI
VIctory: host GPU failure: a host command list did not close (Not enough memory resources are available to complete this operation. (0x8007000E)): a command recorded into it was invalid; a diagnostics build with VICTORY_D3D12_DEBUG=1 names 
2026-11-25T02:22:59.089525Z  WARN victory_gpu::host: no upload buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=4194304
2026-11-25T02:22:59.090249Z  WARN victory_gpu::host: no upload buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=4194304
2026-11-25T02:22:59.090749Z  WARN victory_gpu::host: no upload buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=4194304
2026-11-25T02:22:59.091425Z  WARN victory_gpu::host: no upload buffer error=Not enough memory resources are available to complete this operation. (0x8007000E) size=4194304
```

## DRED (device removal record)

```
2026-11-25T02:21:07.977788Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 1 title copies 0.0 MB; 1 refreshes 0.0 MB, 0 skipped 0.0 MB; 2 runs 0.0 ms; 0 queries 0.00 ms, moves 0.14 ms, 0.00 MB unmapped; 12177 page faults; 3
2026-11-25T02:21:13.495118Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 4 title copies 0.0 MB; 2 refreshes 1.0 MB, 0 skipped 0.0 MB; 2 runs 0.0 ms; 0 queries 0.00 ms, moves 0.21 ms, 0.00 MB unmapped; 10382 page faults; 3
2026-11-25T02:21:18.628800Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 2.4 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.10 ms, 0.00 MB unmapped; 6838 page faults; 4 
2026-11-25T02:21:23.761685Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.4 MB, 0 skipped 0.0 MB; 1 runs 0.0 ms; 0 queries 0.00 ms, moves 0.08 ms, 0.03 MB unmapped; 7660 page faults; 5 
2026-11-25T02:21:28.860917Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 7 title copies 0.0 MB; 7 refreshes 1.4 MB, 0 skipped 0.0 MB; 0 runs 0.0 ms; 0 queries 0.00 ms, moves 0.10 ms, 0.00 MB unmapped; 7260 page faults; 5 
2026-11-25T02:21:33.927396Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 11 title copies 0.0 MB; 10 refreshes 8.3 MB, 0 skipped 0.0 MB; 3 runs 0.0 ms; 0 queries 0.00 ms, moves 0.09 ms, 0.05 MB unmapped; 5554 page faults; 
2026-11-25T02:21:39.027504Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 11 title copies 0.0 MB; 10 refreshes 9.1 MB, 0 skipped 0.0 MB; 7 runs 0.0 ms; 0 queries 0.00 ms, moves 0.15 ms, 0.00 MB unmapped; 7310 page faults; 
2026-11-25T02:21:44.094230Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 10 title copies 0.0 MB; 9 refreshes 7.9 MB, 0 skipped 0.0 MB; 8 runs 0.0 ms; 0 queries 0.00 ms, moves 0.17 ms, 0.00 MB unmapped; 6388 page faults; 1
2026-11-25T02:21:49.184114Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 11 refreshes 9.4 MB, 0 skipped 0.0 MB; 7 runs 0.0 ms; 0 queries 0.00 ms, moves 0.20 ms, 0.12 MB unmapped; 9045 page faults; 
2026-11-25T02:21:54.294593Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 11 refreshes 8.9 MB, 0 skipped 0.0 MB; 5 runs 0.0 ms; 0 queries 0.00 ms, moves 0.23 ms, 0.48 MB unmapped; 9586 page faults; 
2026-11-25T02:21:59.393063Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 12 title copies 0.0 MB; 10 refreshes 8.2 MB, 0 skipped 0.0 MB; 5 runs 0.0 ms; 0 queries 0.00 ms, moves 0.24 ms, 0.99 MB unmapped; 8577 page faults; 
2026-11-25T02:22:05.147949Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 14 title copies 0.0 MB; 10 refreshes 8.5 MB, 0 skipped 0.0 MB; 7 runs 0.0 ms; 0 queries 0.00 ms, moves 0.25 ms, 0.31 MB unmapped; 9754 page faults; 
2026-11-25T02:22:11.668773Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 18 title copies 0.0 MB; 12 refreshes 10.1 MB, 0 skipped 0.0 MB; 18 runs 0.0 ms; 0 queries 0.00 ms, moves 0.44 ms, 4.50 MB unmapped; 31777 page fault
2026-11-25T02:22:16.737251Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 13 title copies 0.0 MB; 10 refreshes 8.4 MB, 0 skipped 0.0 MB; 5 runs 0.0 ms; 0 queries 0.00 ms, moves 0.30 ms, 0.11 MB unmapped; 4748 page faults; 
2026-11-25T02:22:21.940676Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 16 title copies 0.0 MB; 11 refreshes 8.7 MB, 0 skipped 0.0 MB; 6 runs 0.0 ms; 0 queries 0.00 ms, moves 0.31 ms, 0.10 MB unmapped; 1714 page faults; 
2026-11-25T02:22:29.962244Z  INFO victory_gpu::copystat: copies per frame: 4.0 submissions; 15 title copies 0.0 MB; 12 refreshes 9.2 MB, 0 skipped 0.0 MB; 12 runs 0.0 ms; 0 queries 0.00 ms, moves 0.32 ms, 2.55 MB unmapped; 3819 page faults;
```

## Settings

- `--console series-s`
- `VICTORY_TEXTURE_ROOM_PERCENT=75`
- `VICTORY_TEXTURE_STAGE_MB=96`
- `VICTORY_COMPILE_HEADROOM_MB=2048`
- `VICTORY_WINDOW_KB=64`
- `VICTORY_LATE_WAIT_LISTS=96`
- `VICTORY_RELAY=2`
- `VICTORY_INPLACE=1`
- `VICTORY_SHADER_LOOP_LIMIT=64`
