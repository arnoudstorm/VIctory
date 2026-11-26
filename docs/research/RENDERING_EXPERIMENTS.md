# Rendering experiments

Each experiment: one change, a hypothesis, the expected and the actual
result. The runs use the harness (`stability.ps1`, 240 s, `-Capture`,
`VICTORY_DUMP_TARGETS=1;VICTORY_CAPTURE_WORLD_ALSO=10,60,150;VICTORY_PRELUDE_POOL=1`).
Logs and frames are outside the repo (session notes folder).

## E1 (2026-11-25): baseline `fsr-start`

- Build `3f966a8`-era diag build of laptop/780m, no new switches.
- Result: world frames letterboxed and near black (frame 2048 mean 12.0).
  20 "no DXIL part" compute programs, all FSR4. The run ended by itself
  at 190 s with 5 MB of RAM free. 0.9-1.2 fps.

## E2: shader dump `fsr-shaders` (`VICTORY_SHADER_SAVE`)

- Hypothesis: the FSR4 programs are in a container format the
  translator does not know.
- Result: confirmed. 5762 shaders saved; the FSR4 ones have an `ACCC`
  part (FSPX v1) and no `DXIL` part.

## E3: `ffx-builtin` (`VICTORY_FFX_BUILTIN=1`)

- Change: the title's FidelityFX loader is served by our module, and the
  upscaling is a bilinear host pass.
- Hypothesis: the 1080p chain gets its input and the cutscene appears.
- Expected: non-black world frames.
- Actual: lit 3D cutscene on screen (frames 2497-2567, mean brightness
  33-231). The title's dispatch names colour 1280x720 R16G16B16A16_FLOAT
  and output 1920x1080 R9G9B9E5 (unordered access). 1.5-1.6 fps. No
  device removal. The run was stopped by the harness at 240 s.

## E4: `ffx-every` (`VICTORY_FFX_BUILTIN=1`, 420 s)

- Change: none to the code (`1272861`); a longer run.
- Hypothesis: with the upscaler filled, the world keeps rendering.
- Actual: the world ran from frame 1724 (12:02:43) to past 2155
  (12:08:23), when the harness stopped it. The game reached **gameplay**
  (control prompt, camera moved). Black detector: one black frame at the
  cut (1723), none after it. 1.4-1.6 fps in the world (645-717 ms).
  Private memory peaked at 12.5 GB, free RAM at least 7 MB, no device
  removal.
- Note: `-FlickerFrames 2480-2800` saved nothing, because the run never
  reached frame 2480. The capture folder also keeps older runs' frames
  above the new run's last frame; filter by the log's "frame saved"
  lines.

## E6 (2026-11-25): what the 16 syncs per frame and the 400 MB of CPU compares are (run `why-waits`)

- Change: counters only (submissions held for a texture staging buffer
  in flight; why `card_for` keeps a title buffer's copy off the card).
- Result:
  - **Syncs:** 13.9 of the 16 "title memory read" settles per frame were
    holds for texture staging buffers still in flight (about 100
    buffers, 76-93 MB). Every staging buffer a submission named was
    marked in flight, also the unchanged ones it did not copy from.
  - **CPU compares:** 72 copies (401 MB) were kept off the card by the
    cap (`VICTORY_CARD_BUFFERS_MB`, 1,024 MB), and 2 (5 MB) by the
    card's budget.

## E7 (2026-11-25): mark only staging buffers copied from (run `staging-fix`)

- Change: `Staging::take_recorded`. A staging buffer is marked in flight
  only by the submission that recorded a GPU copy from it.
  `VICTORY_STAGING_MARK_ALL=1` restores the old behaviour. Test
  `only_a_staging_buffer_copied_from_is_marked_in_flight`.
- Hypothesis: the 14 holds go away, and with them most of the waiting.
- Actual (`why-waits` -> `staging-fix`, same settings, world samples
  198 / 213):

| | before | after |
|---|---|---|
| holds for staging in flight per frame | 13.9 | 0.0 |
| settles per frame | 16.0 | 5.2 |
| waiting for the card in settles, ms per frame | 342 | 201 |
| frame (overlay), ms | 782 (1.46 fps) | 732 (1.59 fps) |
| CPU, ms | 372 | 409 |
| GPU wait, ms | 339 | 222 |

  The frame time gain (6 %) is inside the run-to-run spread (about
  15 %, E5). The waits clearly fell, but CPU time rose: the CPU is now
  the limit. Picture: unchanged, 63 saved world frames 2476-2899 all lit
  (mean 35.9-228.6), black detector only at the cut (2475), no device
  removal. Free RAM fell to 5 MB in both runs.
- Tests: 544 passed, 0 failed.

## E8 (2026-11-25): where the CPU time goes after E7 (run `staging-fix`, no change)

The submitting thread prepares about 39 submissions per frame, spending
about 136 ms user and 93 ms kernel time per frame inside them. Recording
threads spend 81-88 ms each in parallel, which is not the limit. Per
frame:

| Step | ms |
|---|---|
| moves (kernel copies of title memory into host copies, `copy_mapped`) | 84 |
| snapshots of shader-written ranges | 50 |
| write-watch asks | 49 |
| finding what shaders wrote | 27 |
| guards | 24 |
| page faults | 35,700 per frame |

The levers that look cheap were already pulled by earlier sessions (see
`run-laptop.ps1`): refreshes and compares over threads (`WIDE_REFRESH`,
`WIDE_BACK`), joined copies, windows in place (`VICTORY_INPLACE=1`),
the card cap lowered to 256 MB for memory, and tiled windows. The
reported display mode is ignored by the title (`resolution-hdr-780m.md`).
What is left is structural: stop copying title memory the card can read
in place.

## E12 (2026-11-25): controller input lag (counters in `gameinput.rs`, run `bvh-on`)

- Change: counters only. Per 5 s, the title's calls for the current,
  next, previous and temporal readings, references too old, readings
  made, and the age of the current readings handed out.
- Result: in the world the title asks for the **current** reading every
  frame (36-294 calls per 5 s) and for the next one after it, which is
  never there (`next_none` = half the calls). It never walked old
  readings and never got "reference too old". VIctory polls the pad
  every 8 ms and hands out the newest state, so it adds at most about
  8 ms.
- Conclusion: the lag is the frame time. The title reads the pad once
  per frame, and a world frame takes ~700 ms to render and show, so a
  press shows up after 0.7-1.4 s. Only a higher frame rate fixes it.

## E14 (2026-11-25): where the windows' duplicate memory is (run `window-shapes`, counters only)

Windows outside buffers: 2,031 MB over 1,368 MB of title memory, so
about 660 MB is held twice. By shape: tails 582 MB over 554 MB, grid
windows 1,449 MB over 1,043 MB (about 406 MB grid-on-grid), and about
229 MB where tails and grid windows overlap. Grid windows overlap
because descriptor slots name their window by their own range (`find`):
the pass that refreshes a slot must find the same copy again, so two
slots over overlapping ranges get two windows. Sharing one window would
mean refreshing the copy a slot holds instead of looking it up again
(14 call sites of `shadow::find`). Not done: a wrong refresh means
stale data on screen.

## E15: 4 MiB windows over tiles (`VICTORY_WINDOW_KB=4096`, run `window-4m`)

- Hypothesis: with tiled windows (only copied tiles hold memory), one
  window per 4 MiB cell removes the duplicates at little cost.
- Actual, against `window-shapes`:

| | 256 KiB grid | 4 MiB grid |
|---|---|---|
| tiles mapped (memory held) | 1,522 MB | 1,906 MB |
| GPU memory | 7,189 MB | 7,245 MB |
| moves per frame | 90 ms | 42 ms |
| CPU per frame | 519 ms | 318 ms |
| frame | 806 ms (1.75 fps) | 672 ms (1.90 fps) |

  Faster (17 %, just beyond the 15 % spread), but **more** memory held
  (+384 MB of tiles). It is an option for speed, not for memory. The
  run script keeps 256 KiB.

## E16: the title's own graphics settings lowered (run `low-settings`)

- Change: the title's settings file (`storage/settings/settings.xml` in
  VIctory's data folder, not in the repo; original kept beside it as
  `settings.original.xml`): `lodScale` 1.0 -> 0.6, `grassLod` 1.0 ->
  0.5, `maxTexUpgradesPerFrame` 5 -> 2, motion blur off, alpha-tinted
  and particle shadows off, `vehicleHDCount` 2 -> 1, water reflections
  512x256 -> 256x128, fullscreen 3840x2160 -> 1920x1080. HDR was
  already off (`hdr false`; VIctory also answers SDR).
- Actual: no measurable change. GPU memory 7,190 vs 7,189 MB, window
  tiles 1,498 vs 1,522 MB, draws per frame 4,323 vs 4,281, frame 731 vs
  806 ms (inside the spread), free RAM down to 3 MB. The console build
  most likely uses its fixed Series S preset and ignores these values.
  Saves exist (a 252 KB story save `SARC0015` from 08:01, a profile, the
  settings profile); they were not touched.

## E18 (2026-11-25): slots share windows (`VICTORY_SHARE_WINDOWS=1`, run `share-windows`)

- Change: a buffer view's slot is made on a window that already holds
  its range (`find_any`) instead of one named by its own range, and the
  passes bring up to date the copy each slot was made on
  (`HostHeap::slot_copy`; for all five `readable` callers and the heap
  pass). Nothing is swapped under a slot or a list, unlike E17.
- Actual, against `window-shapes`: **stable** (420 s, no device
  removal, black detector only at the cut), picture right (frame 2769).
  Faster: frame 806 -> 662 ms (1.75 -> 1.86 fps), moves 90 -> 51 ms,
  CPU 519 -> 391 ms. But **not less memory**: window tiles 1,522 ->
  1,939 MB, GPU memory 7,189 -> 7,424 MB, because the longer windows
  slots now take stay alive and fill more tiles. Kept as an opt-in speed
  option, off by default.

## E19 (2026-11-25): skipping the intro (hold A), run `skip-intro`: the title's tile pool is full

- Settings: `VICTORY_FFX_BUILTIN=1 VICTORY_AMBIENT=0.6`, pad A held 4 s every
  15 s from 70 s (`hold_a_keys.txt` in the session notes).
- Actual: the intro was skipped; the loading screen ran at 30 fps (3 draws);
  at 115 s the **title crashed** (access violations reading
  `0xffffffffffffffff` and `0x20` in title code, three threads). Free RAM
  5 MB, private 11.5 GB.
- The title's own crash context (`temp\*.crash.log`): console Lockhart
  (Series S), SRL `int_prolo_int_prolo_int_min` in **Prestream** (upstream
  #1118), **TileHeap used 3,977 of 3,977 MB**, Game Physical 1,289 of
  1,339 MB, 228 streaming requests.
- The TileHeap is the title's own pool inside the physical pages it asked
  for at start (`XMemAllocatePhysicalPages`, 3,977 MB); VIctory does not
  size it. A real Series S has the same pool, so the title asks for more
  tiles here than it would there. The title never calls
  `XMemGetWorkingSetStatistics` (no log line), so VIctory's answer there is
  not the cause.
- Next lead: what the title's texture streaming reads back from the GPU to
  pick mip levels. A read-back that comes back wrong (zeros: "the finest
  level everywhere") would fill the pool. To check: find the streaming
  feedback pass and what it writes; compare what the CPU gets back.

## E20 (2026-11-25): an imported Chapter 4 save, and a temporal upscaler (`VICTORY_FFX_TEMPORAL`)

- Save: a decrypted PS5 story save converted to the Xbox container format
  (format from LilBigJP/ztrx-ps5-save-import), loaded with the opt-in
  `VICTORY_PATCH` (the title's story-save encryption check answers "no";
  the patch checks the original bytes first). Not in the repository.
- Without the temporal pass (run `save-run`, `-Preset low`): the save
  loads, the city loads, the player is in control with the HUD at
  ~1.3 fps, 12.4 GB RAM, VRAM 6.7 of 5.8 GB budget. Visible defects:
  grainy hair and cloth (stochastic/dithered shading the title leaves to
  FSR4 to resolve over frames), black blocks over the sky's clouds (present
  on the loading screen too, so before the upscaler), very dark shadows
  (no ray-traced light).
- The FidelityFX dispatch carries what a temporal upscaler needs: colour
  1280x720 RGBA16F, depth, motion vectors 1280x720 RG16F, motion-vector
  scale `[1, 1]` (render texels), a reset flag. No exposure or masks.
- Temporal pass (`VICTORY_FFX_TEMPORAL=1`, weight 0.1): the spatial result
  blended with our own history pair (2 x 1920x1080 RGBA16F, 33 MB),
  reprojected by the motion vectors, clipped to the colour's 3x3
  neighbourhood in YCoCg, non-finite values dropped.
- Result: **3 of 3 runs failed before gameplay** (runs `taa-run`,
  `taa-run2`, `taa-run3`): twice a host list did not close with
  E_OUTOFMEMORY about 2 minutes after the first temporal upscaling, once a
  device removal after a 60 s submission one second after the first one.
  RAM and VRAM in the title bar were not higher than without it.
- Debug run (`VICTORY_D3D12_DEBUG=1`): no debug layer (0x887A002D, the
  Windows "Graphics Tools" feature is not installed); the same
  E_OUTOFMEMORY, with allocations as small as 256 bytes refused.
- Baseline again without the temporal pass (run `base-run`): **also a
  device removal after a 60 s submission.** So the failures are not shown
  to come from the temporal pass; with this save the 16 GB UMA laptop is
  over its GPU budget and runs fail either way. Kept off by default.

## E21 (2026-11-26): the "missing pipelines" are the title's precompiled cache, not missing

- Run `perf-run` (same settings as E20's baseline): loading screen 30 fps
  at 1.3 GB VRAM; in the world 1.3-1.4 fps (~700 ms: ~240 ms preparing
  copies and textures, ~400 ms waiting for the card), VRAM 6.4 of 5.7 GB,
  the PC's free memory down to 22 MB, then E_OUTOFMEMORY after ~100 s.
- Device slots 0x1c8 and 0x218 (unimplemented, E_NOTIMPL) are called
  about 16,000 and 2,000 times, all in the first second: the title's
  pipeline library trying precompiled (native GFX10) pipelines. The
  disassembly at title+0x247fe25 shows that when 0x218 gives none, the
  title makes the pipeline through 0x1f0 (`create_compute_x`); graphics
  pipelines are made on demand through `create_stream`. So they are not
  the cause of the black cloud blocks; the log text is corrected.

## E22 (2026-11-26): `run-laptop.ps1 -Preset lowmem` (opt-in)

- Change: 'low' with `VICTORY_TEXTURE_BUDGET_MB=768` (1024),
  `VICTORY_TEXTURE_MIN_FREE_MB=3584`, `VICTORY_TEXTURE_STAGE_MB=16`,
  `VICTORY_SHADER_LOOP_LIMIT=32` (64), `VICTORY_KEEP_SNAPSHOTS_MB=64` (256).
- Actual, world medians from the overlay, against E21's `perf-run`:

| Run | Frame | CPU | GPU wait | VRAM peak | RAM peak | Outcome |
|---|---|---|---|---|---|---|
| `perf-run` (low) | 717 ms | 251 ms | 405 ms | 6,445 / 5,734 MB | 11.7 GB | out of memory after ~105 s in the world |
| `lowmem-run` | 718 ms | 313 ms | 332 ms | 6,137 / 5,720 MB | 13.3 GB | out of memory after ~150 s |
| `restart-run` (lowmem) | 860 ms | 354 ms | 422 ms | 6,362 / 5,725 MB | 13.0 GB | still running at the 420 s cut (~110 s in the world) |

- About 300 MB less card memory, no faster, and the run still goes over
  the card's budget. The new loop limit made the first run translate its
  shaders again (load ~4 min instead of ~1). Not a fix: the title's own
  memory (TileHeap 3,977 MB, E19) plus ~3 GB of host copies do not fit
  16 GB. Kept as an opt-in preset; 'low' stays the default.

## E23 (2026-11-26): synthetic upload-before-alias regression and per-list source identity

- Fixture: original packed UINT compute producer, UINT source SRV, aliased R9G9B9E5 SRV, two array layers and three mips. On the old implementation the producer's GPU readback matched but the placed SRV returned zero for expected `[1,1,12]` at mip 0/layer 0/texel 0, after a title-memory upload consumed the generic first-use marker. This is the expected negative control.
- Fix: commit `559dc6101ab7513ffc81036298f86582bf339eab` tracks destination view → last GPU source in `Deferred`. Generic first-use remains for uploads. `texture::alias` now copies when the source identity is new, even if the source's first transition is deferred to the prelude. Existing state transitions still cause another copy after source use/write. Reset clears the tracking map.
- Actual synthetic GPU test passed both default synchronous and `VICTORY_LATE_WAIT=1` submission paths on the available AMD adapter, with exact producer and placed-view readbacks across mips/layers, repeat reads, a second write epoch, reset, and a replacement source at the same title address. It did not release the old COM source before replacement, exhaustively test detach/replay, or establish the runtime sky binding.
- Not a gameplay or visual-fix claim. Exact sky producer/view, descriptor retirement timing and current artifact frame remain unresolved. No title/game was launched.
- Separate CPU diagnostics change in `f15aa66f0c33b9ec5f7f65adf777ef41c4108235` halves one repeated submission fingerprint in its diagnostic path; its isolated synthetic result is recorded in the performance report. `63a247a87af6d15840bbef15789d13cea3317b1d` adds counters only; descriptor owners are not released early.

## E24 (2026-11-26): repeated warm and cache-evicted submission fingerprints

- Test-only comparison: two stable fingerprint reads (the former submission observers) versus one shared read. Each sample used a 128 KiB private source; fingerprinting reads 48 KiB from it.
- Seven alternating-order warm batches of 256 inputs: median 5.838 ms for 512 operations/24 MiB versus 2.880 ms for 256 operations/12 MiB. Operation and byte reductions are exact; timing applies to the synthetic hot-loop only.
- Nine alternating-order cold-cache-approximation samples used a 64 MiB cache-thrashing sweep before each timed case: median 45.5 μs for two hashes/96 KiB versus 34.7 μs for one/48 KiB. This does not flush hardware caches. The old case's second read may be warm. Treat timing confidence as medium-low.
- No title frame time, GPU time or system memory savings were measured. This test-only methodology commit is `015931ce14e7a2e457f946efcdf14c0d1b9b38ea`; the runtime behavior is unchanged from `f15aa66f`.

## E25 (2026-11-26): bounded refresh intervals and descriptor owner telemetry

- Source: `crates/victory-gpu/src/host.rs:6495-6708` adds `VICTORY_REFRESH_INTERVALS` (default 1, preserving prior hull behavior; clamp 1-32), smallest-gap bounded range coalescing, and `Batch::{add,flush_one,flush}` support. It is opt-in; no game workload telemetry yet shows that hull gaps are a material cost.
- Synthetic test: `host::refresh_interval_tests::bounded_ranges_preserve_requested_bytes_and_merge_the_smallest_gaps` uses 32 disjoint 4 KiB requests at 512 KiB spacing, with a 16-range cap. The requested union is 131,072 B, one hull is 16,257,024 B, and bounded spans total 8,454,144 B: 7,802,880 B (48.0%) less requested span than the hull. This measures synthetic range span only, not `Shadow::update` bytes, CPU time, FPS or game memory.
- Two more tests cover overlap/adjacency and smallest-gap cap behavior. Diagnostics ownership summary in `descriptor/host.rs:17-27,44-53,1174-1285` counts resources with no other descriptor-slot owners in that same host heap; external COM owners are not enumerated. Weak shadow links do not own mapped memory.
- `victory-gpu` serial diagnostics tests: 156 passed, 14 ignored, then the new owner-summary targeted test passed. Full workspace serial suite: 565 passed, 0 failed, 21 ignored across 15 test binaries. Initial parallel workspace run exited with STATUS_ACCESS_VIOLATION; serial reruns passed, cause unknown.
- No actual D3D12 alias GPU fixture was run this turn; no 780M or gameplay test. Rendering/sky remains unverified.
- Candidate `VICTORY_REFRESH_INTERVALS=16` should be A/B tested against unset/default (1) with identical title/save/scene/capture settings. Keep default at 1 unless a real workload demonstrates lower copied bytes without added CPU time or rendering regressions.
