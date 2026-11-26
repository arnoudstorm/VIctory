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

## E26 — startup private memory and deferred list objects (26 November 2026)

Source commits: 17a7dd9059fab3ea25ea94ba3bf4aadc56779069 (opt-in lazy object, ownership telemetry, sampler), d2562a06250addbdd82604452698e5552486a00b (lock-free allocation category sampling). No game launched. Source defaults preserved; enable candidate with VICTORY_LAZY_LISTS=1 and VICTORY_LAZY_LIST_OBJECTS=1. It defers creation of ID3D12GraphicsCommandList until first use, while retaining an allocator and unchanged fence/replay ownership. It does not reclaim already-recorded command memory or solve the multi-GiB startup burst.

Three alternating runs of 64 production wrappers: existing private deltas 15888384/15921152/15876096 B, creation 5389/4947/5225 us; deferred deltas 1024000/1007616/1011712 B, creation 972/940/1015 us. Medians: 15,888,384 -> 1,011,712 B and 5225 -> 972 us. ~137 MiB deferred for 620 unused wrappers is an extrapolation only. Existing separate lazy-open fixture: 241 KiB per wrapper made vs 1338 KiB once opened, three repetitions. No gameplay result.

Original isolated fixture separated 64 allocators (~0.98 MB process private, ~0.875 ms) from 64 unopened command lists (~14.89 MB private, ~4.416 ms). Process deltas are not exclusive ownership attribution. Never-reset counts cannot be called never-recorded. Pool is capped at 128 completed lists/256 MiB scratch; detached count includes unsubmitted held recordings and is not exclusively in-flight. No descriptor cleanup: latest run had zero same-heap exclusive released-slot resources. No aggressive pool/allocator retirement.

VICTORY_ALLOCATION_LOG=<absolute file> persists diagnostics independently of presents, 1 s normally / 250 ms below 1 GiB. Timestamps, private bytes, total working set, available memory, remaining commit, live Rust allocation/staging/shadow/host-texture bytes, list counters and detached count. Categories may overlap; not a total private-memory decomposition. External outputs/monitor-victory-memory.ps1 runs GlobalMemoryStatusEx guard independently of slow detail queries, collecting private working set, commit and paging in a worker. A 5 s delayed worker did not delay the guard: sustained-low test gaps 976-1025 ms; immediate-256 test gaps 995-1008 ms. Both simulated threshold tests stopped; no real pressure induced or applications closed.

GPU deferred-object regression verifies first-use copies, two completed-fence replays, reset, second epoch and exact contents; ownership counter resets/drops tested. Opt-in diagnostic GPU suite 160 pass/0 fail/16 ignored. Full serial suite 566 pass/0 fail/23 ignored; final category-only follow-up reruns recorded in outputs/startup-memory-development.md. Existing dead-code warnings retained. The terminal startup burst is still unattributed; title footprints do not prove committed memory. Final 5-second category gap cannot be reconstructed.

Current system optional groups measured private WS: Discord 422 MiB, Claude 275, Steam+web helpers 85, Epic components 73, Teams 29. Suggest manual normal Exit only after saving and confirming idle; hypothetical 0.3-0.6 GiB recoverable, measure before/after. Exclude Codex, security, services and unclear ChatGPT ownership. No causal attribution possible for old 5.1 -> 2.9 GiB available-memory difference without contemporaneous process baselines. Full report, benchmarks and guard results are outside repo in outputs/startup-memory-development.md and JSONL/result files. Keep original executable hashes and earlier 3a656f1 candidates protected.

## E27 — authorized deferred-object startup test, 26 November 2026

Source d2562a06 diagnostic candidate, SHA-256 808ED844B8EDA0A5C0B3BADF05D338861769F135A7EE3A9142AA226A961CAAD3 verified. Windows unlocked; prelaunch available RAM ~5.37 GiB. Low/safe, refresh intervals=1, lazy lists and lazy list objects=1, memory-map walk/capture off, persistent allocation log. Guard stopped at ~50 s after sustained <512 MiB. 51 guard samples, gaps 711.6-1292.6 ms; faster runtime sampler saw a transient ~95.9 MiB available just before the guard stopped. No guaranteed transient threshold prevention by one-second guard.

Observed loading artwork, no gameplay/world. Window capture succeeded; Enter UI helper failed 'foreground window did not report a process id', so no verified responsiveness. Do not claim gameplay FPS or rendering correctness. Between 37.042 and 49.567 s private memory rose 5080223744 -> 7005270016 B (~1835.9 MiB), tracked Rust live grew ~49.4 MiB; staging decreased, shadows stayed 182452224 B, host textures +~14.9 MiB, list count stayed632, lazy opens stayed15. Burst is not explained by those tracked counters; no arbitrary driver residual attribution. Last category snapshot: pool5 lists/0 scratch, shaders116 MB/translations9 MB, card778/7542 MB. System paging peak6722 reads/s,56772 pages input/s among slower samples; not per-process attribution.

No matched A/B: prior run had less initial RAM, different map/cache conditions. No further launch or release refresh test. Full evidence outside repo: outputs/deferred-startup-20261126-141744/result.md and sibling JSONL/logs. Game closed, RAM recovered ~5.4 GiB. Next: instrument non-Rust/title/driver private commitment around37-50 s; preserve current candidates/defaults. Require next controlled test authorization.

## E28 — private-commit attribution instrumentation (26 November 2026)

Game remained closed; no title allocation policy or Windows settings changed. Existing ~1835.87 MiB E27 burst remains unattributed. Stable Rust live bytes do not exclude allocation/free churn and native heap retained capacity. No driver ownership is inferred from the unobserved remainder.

Commits: 774798c7 observes title Windows/Rtl heap and VM import boundaries, XMem virtual/physical operations, raw stacks for large calls, independent VirtualQueryEx allocation-base deltas and module metadata. c8f16786 validates extended VM signatures. a04657e0 adds cumulative Rust layout allocation/free traffic, distinct from live bytes and OS commit. ed67d75e enforces external output paths for event/region/allocation logs. 68d324bd preserves native dynamic export availability. 270814c5 adds offline tools/analyze-memory-burst.ps1. All observation is diagnostics-only and opt-in; deferred-list optimization remains separately controlled and unchanged.

Coverage limits: no global API detours; internal driver/native DLL allocations, direct Nt VM calls and other bypasses are not captured. Events are requests/results, not unique committed bytes. Region types are neutral; aliases and copy-on-write prevent equating their totals with PrivateUsage. Workers are independent of the external safety guard; output can be partial after forced exit. Raw stack/module/RVA matches alone do not establish ownership. Detailed schema, limitations and source references are in outputs/startup-burst-attribution.md outside this repository.

Synthetic evidence: reserve/commit/recommit/decommit/release classification passes; anonymous mapping vs image classification passes; native and Rtl heap reallocation preserves data and failure last-error behavior; dynamic lookup availability matches native exports. Controlled heap after freeing every block: 0 allocated bytes but 3526656 committed /24584192 reserved, demonstrating retained capacity only in the fixture. Three paired small-heap trace batches: median 4.683 ms off vs6.282 ms on (8192 calls); no reported drops. Large-stack and region-walk overhead were not benchmarked in a title workload. Parsed synthetic traces:4657 event rows,8 stack-bearing,0 reported drops;108 region/module rows,1 completed scan,2358 us scan duration. Rust allocator churn test allocates and frees a real vector and observes cumulative traffic.

Faster external guard: outputs/monitor-victory-fast.ps1 plus victory-fast-guard.cs uses an independent .NET thread with compensated100 ms polling, no file/CIM/region work on safety thread, bounded sample queue and pinned authorized process. Simulated96 MiB transient from3500-3800 ms stopped at3509.06 ms despite a5 s blocked consumer. Sustained-low test stopped at3208.88 ms; recovery test did not falsely trigger. Max gaps109.38/109.75/113.79 ms. No real memory pressure induced; no real-time guarantee. Existing thresholds unchanged.

Offline correlator self-test passes: two16 MiB overlapping commit requests remain32 MiB of requests, actual region growth16 MiB plus8 MiB unobserved=24 MiB; incomplete64 MiB scan ignored; unobserved allocation stays unattributed. The script reads JSONL and writes stdout only, never operates on a process. Source-root log-path rejection tested independently for all3 outputs;4 core policy tests passed.

Final suites: default full serial workspace 566 passed/0 failed/23 ignored; diagnostics GPU 161/0/16 ignored and HLE 119/0/1 ignored. Standalone correlator self-test passed; ignored native heap benchmark explicitly ran6 A/B repetitions. Existing2 diagnostic/4 default GPU dead-code warnings remain; git diff --check passed.

Latest isolated diagnostic candidate: source 270814c5da6f98ca5b77cea849595b1ca625a191; outputs/VIctory-780m-memory-trace-270814c5.exe; SHA-256 556CCA0BF624D89C670A327331D1887192ADED6FC059FE3E093B584C3A41C99F. Built --locked --profile diag -p victory --features diagnostics; not launched. Older candidates and protected baseline/release hashes verified unchanged. laptop/780m stays f52a6102. No push/merge.

Next authorized test: use latest trace candidate with 100 ms external guard, persistent MEMORY_EVENTS/MEMORY_REGIONS/ALLOCATION_LOG files outside repo, fixed low/safe settings and refresh intervals1. Disable old MEMORY_MAP walker to avoid overlapping scans. Keep deferred-list flag isolated/fixed. Correlate complete region scans bracketing 37-50 s, cumulative Rust traffic, API result ranges/flags and raw stacks/module bases; never sum commit requests or label unmatched growth as driver. No further game launch until authorized.

## E29 — Run #5, authorized memory-attribution startup (26 November 2026)

One launch of diagnostic source270814c5, executable SHA-256556CCA0BF624D89C670A327331D1887192ADED6FC059FE3E093B584C3A41C99F, verified before/after. Low/safe, BVH/rays0, refresh1, deferred list objects1, stack threshold16 MiB, event/region/allocation logging, old MEMORY_MAP off. Guard preinitialized with3600 s maximum and100 ms target polling. Initial launch available RAM4.60 GiB, AC100%; Windows unlocked. Baselines/source/saves not manually changed; save mtimes preceded run. No restart.

Guard stopped after38.903 s at220549120 B available (210.3 MiB), immediate <=256 MiB threshold.390 guard samples, mean100.007 ms, max513.463 ms,0 dropped guard samples. Peak private5598175232 B (5.214 GiB), total WS3613093888 B (3.365 GiB), peak sampled private WS2938544128 B; local GPU663/7542 MiB observed. Available-RAM transient extrema between samples remain unknown. Temperatures unavailable. Paging peak7975 reads/s and17735 pages input/s system-wide, not process-attributed. No device-removal report before guard stop.

Verified interactive gameplay0 s; no milestones. Startup overlay windows7.4-37 FPS,1-11 draws, not gameplay/frame-percentile measurements. Activation failed 'failed to activate captured window'; recovery found process already stopped. No successful input or Run5 screenshot, no sky/geometry/HUD verdict. Do not infer a rendering regression or release FPS from this run.

Allocation origin evidence: four initial XMemVirtualAlloc calls with0x203000 normalized flags and matching private regions:1277,1162,258,194 MiB (2891 MiB/2.823 GiB total), title caller RVAs0x1e1d4db/0x1e1d186/0x1e1d28d/0x1e1d36e. Additional matched20 MiB region. Initial commitment is identified; necessity/duplication is not. Two16 GiB placeholders are reserve-only, not32 GiB RAM. Section-map offset overlap shows aliases/repeated coverage, not duplicate physical backing.

Trace89829 timestamped events,4891 reported drops,24 stack-bearing events. Physical map/commit events>99% of timestamped records.17 complete region scans up to53898 regions, max384100 us per scan: diagnostic overhead is material. No HeapAlloc/Rtl records observed; coverage/loss prevent concluding native heaps absent. Rust sampler window allocated6.55 GiB/freed5.91 GiB, last live679461170 B (~648 MiB); churn may coexist with retained capacity.632 wrappers,15 cumulative lazy openings,5 pooled completed lists/0 pooled scratch: no evidence for descriptor or aggressive list cleanup.

Prior37.042-49.567 s burst was not covered completely: last allocation sample37.807 s, last complete region scan36.108 s. Offline tools/analyze-memory-burst.ps1 ran for covered terminal interval29.200-36.108 s: private+72241152 B (~68.9 MiB), mapped+4 MiB, image0. Largest terminal private growth regions had no observed matching events and remain unattributed. Old burst root cause unresolved. Conditions differ from earlier runs (RAM/cache/instrumentation); no improvement/regression claim.

Full report/logs outside repo: outputs/run5-20261126-150044/run5-report.md. Native/VM stacks and region/event identity match evidence: region-event-stack-correlation.json; correlator output terminal-correlation.json. Report includes all ten requested sections, metrics, provenance, log index, preservation limits and targeted engineering proposals.

Next development priorities, no automatic relaunch: batch/coalesce high-volume physical-map/commit diagnostic traffic while prioritizing coarse VM events; validate loss/tail accounting. Tag Rust allocation traffic by DirectStorage/decode/shader/scratch owner before bounded buffer reuse. Investigate matching174 MiB regions by resource identity, not size alone. Do not alter title arena semantics, enable descriptor cleanup, repeat E17, or combine refresh A/B until safe startup/input exist. Keep game closed until another controlled test is authorized.

## E30 — Phase 6 and Run #6 (26 November 2026)

Implemented source commits 0ab77fd1 (batched/prioritized allocation diagnostics, optional region interval5s), 27c618f (default-off VICTORY_DSTORAGE_REUSE=1 bounded CPU request scratch: max4MiB/buffer,16MiB global), f69d42f (large mapped/resource owner and scratch growth tags; optional normal startup foreground request), ff3d88a7 (critical events wait through short memory-only queue drains). No arena semantics, descriptor cleanup, GPU ownership/fence behavior or Windows settings changed.

Synthetic: 20,000 ordered JSON lines/1,328,894B, flush calls20,001->79, times54.648-58.347ms->3.503-3.855ms. Optimized scratch incl mutex:2,048 allocations/301,989,888B requested->3/458,752B;30.964-53.110ms->2.195-2.378ms, three alternating-order trials. Retained448KiB in fixture and up to16MiB runtime; not lower peak RAM or gameplay gains. Real heap trace after correction captures4,096alloc+4,096free,0drops/order errors. First nonblocking critical producer lost849 events; corrected, not hidden. Debug pool timing regressed substantially; optimized evidence differs.

Final serial workspace567pass/0fail/24ignored; diagnostics GPU161/0/16, HLE121/0/3 plus1integrationpass. Explicit ignored benchmarks run separately. Same pre-existing4default/2diag warnings. Final source ff3d88a7399dde2ede1ade64edd8b5e7b1ef1463. Isolated external target and separate final release/diag executables, baseline hashes unchanged; laptop/780m remainsf52a6102. Final release hash353ABDAC3604120B92510262E79DD610A025C47B21975A58236604B299C75A76; diag486978F192B279F695428AC66CF60E75371F10585B160D4DFD7EE4FE5FB852CE.

Run #6 used f69d42f release (same release hash), one launch. Unlocked/session1/Default verified. Low/safe,ray/BVH0,refresh1,lazy flags1,scratch reuse1; heavy event/region/map diagnostics off. Initial RAM5.049GiB. Guard stopped72.711s at212.848MiB available, immediate threshold;728samples,mean100.015ms,max308.918ms,no drops/errors. Peak sampled private6.612GiB,totalWS4.064GiB,privateWS3.259GiB,GPU824/7542MiB. System paging up to2420reads/s,48639pagesinput/s. Zero verified gameplay. Loading artwork then black loading frame observed; normal Enter helper succeeded and game window focused, but title/controller responsiveness, sky and gameplay remain unverified. Do not treat loading9.9-35FPS as gameplay. No restart.

Run5 vs6 is not matched: diagnostic vs release, different initialRAM/cache and tracing. No measured startup memory reduction or gameplay optimization gain claimed. Large174MiB Shadow/Card mapping now has distinct resource/staging identity tags, but neither prior174MiB private region is conclusively assigned. Shader containers already Arc-shared by content hash. No large texture-scratch growth observed; no summed capacity or shrinking policy. End-of-run loading logs attribute much texture-refresh time to waiting for title memory remapping; preserve memory-stability guards.

Full report/evidence outside repository: C:\Users\Arnou\Documents\Codex\2026-11-25\ana\outputs\phase6-development-and-run6.md; run6-20261126-153647; phase6-final-build-provenance.json; phase6-validation. Game closed, no push/merge, unrelated apps untouched.

## E31–E33 — Phases 7–10 checkpoint, 26 November 2026

Starting clean HEAD3d90239b, experimental branch preserved. Runtime source commits: eda05157709cc516165fec15b3d97fb15026d265 (diagnostic per-role DirectStorage capacity/counter worker); 5124706d77da14feeb40b66848da9d2861adb41a (consume compressed input before later destination/retry work); 063bd97b457eae098bde79f15682fc5d7162fec7 and f43ae7fbaeb604679f022f3052930f0eaee5161b (default-off VICTORY_COPIED_READ_UNLOCK=1 and shared kernel snapshot helper). Runtime diff217additions/17deletions across3files. No title-arena, descriptor cleanup, aggressive window merge or lifetime/fence removal.

Run7diagnostic eda05157 SHA3CC740DA422C88920A64B209390E3F800C7E21A26869CBFEE76675A3D514AB88:47.9125954s,0verified gameplay; initial available5.808GiB; peak sampled private6.588GiB,totalWS4.754GiB,privateWS3.929GiB,GPU816/7542MiB; immediate guard at232.086MiB.480samples,mean100.026ms,max390.032ms,no guard errors/drops. Critical event drops0; physical-observation drops2460; tail/scan coverage incomplete. DirectStorage source requested345911970B,fresh106825227B,peakactive23585941B,retained4047598B,539allocations/12654reuses; intermediate/fallback0. One174MiBprivate allocation base2055208960000 exactly matches CPU staging mapping; second2055391608832unknown. No unobserved native remainder assigned to driver. Completed late scans end14:54:45.904UTC, before47.9s stop.

Run8release5124706d SHAF696D7313659BFD559B9DA56590BDE6A9BB8B03308DC9709F75AA1852E5617BE:55.0100256s,0verified gameplay;initialavailable5.876GiB;peakprivate6.629GiB,totalWS4.960GiB,GPU817/7542MiB;immediateguard224.898MiB.551samples,mean100.018ms,max373.564ms,no errors/drops. Both unlocked/Default/session1;normalEnterdelivery/focusaccepted,loadingart/logo only. Different instrumentation/cache/background means no matched savings/FPS comparison. User approved optional normal exits if idle, but background-only apps could not be verified idle; none closed. No OS setting or title/save edit. Save metadata12files unchanged in length/timestamp; contents not hash-compared.

Runs9/10 WERE NOT EXECUTED under user's repeated-unresolved-critical-memory safety condition. Only Run7/8actual launches consumed in outputs/phase7-10-launch-ledger.jsonl. No automatic restart or Run11. Do not resume launch sequence without a substantiated safe headroom remedy; no gameplay recovery/endurance milestone claimed. Synthetic work and comprehensive audit continued after the stop; this is not an after-Run10 success report.

Optimized decode lifetime fixture:peakowned3146059->2097483B,32requests;baseline29.661-32.042ms,new25.443-27.484ms. Relevant combined-codec path absent inRun7, so no startup claim. Copied-read guard fixture:64KiBcopy,actualdecommit/remap,successful GPUfence/readback;3pairedoptimizedwriterwait12.20-12.56ms->47-67us,callback11.9-12.7msapproximatelyunchanged,copy/hashworkequal. Flagdefaultoff,actualworkload untested. Native64MiBsectionaliasfixture showsWSdouble-counting aliases whileprivatecommitbarelychanges; not unique physicalRAM.

Final sourcef43ae7fb:serialworkspace569pass/0fail/26ignored;GPUdiagnostics162/0/17;HLE123/0/4plus1integration. Suitesoverlap; not a unique sum. Guard3fixtures pass;target100ms not real-time. Four default/two diagnostic existingwarnings. Final external releaseSHA B144D16E22BFC6D62BF2C1D5B5F9D4EC4DB6B138785D430CFD00B15D5FA556A8;diagnostic B7A7583BCC2DCAF6D2EC73EBAD266667BB3DB4C9086E57376AC35F85812FE9F6;neitherlaunched. Baselines and laptop/780m f52a6102unchanged. No push/merge/publication.

Deliverables outside repo: C:\Users\Arnou\Documents\Codex\2026-11-25\ana\outputs\phase7-memory-investigation.md;phase8-memory-optimization.md;phase9-gameplay-recovery.md;phase10-final-candidate.md;phase7-10-experiment-ledger.md;victory-780m-comprehensive-audit.md;performance-history CSV/JSON (126evidenceentries,86archivedreports; duplicates/missingvaluesexplicit);final-build-provenance JSON;remaining-issues/next-roadmap;localRedditupdate7draft;performance-graphs (12PNG/SVGpairs). Reports include exact references, limitations, historical/current distinctions and unsafe-run decision. Graph generation dependencies are confined to workspace work/plot-libs, no Windows/global package settings changed.

## Cycle V tests 29 and 30

Test #29 measured 578.75 MiB of observed independently backed tile overlap. Zero geometry requests qualified for tile savings. The emergency watchdog stopped the run before a sampled full-world frame. Test #30 was held. See the [Cycle V handover](CYCLE_V_HANDOVER.md) for the decision and test evidence.
