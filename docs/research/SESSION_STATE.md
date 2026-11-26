# Session state (checkpoint)

2026-11-25, branch `laptop/780m` (`arnoudstorm/VIctory`), worktree
`C:\Users\Arnou\VIctory-laptop`. Tests: 558 passed, 0 failed.

## What runs now

Best configuration measured (run `all-new`, 360 s, Series S):
`VICTORY_FFX_BUILTIN=1 VICTORY_FFX_FILTER=bicubic VICTORY_AMBIENT=0.3
VICTORY_D3D12_BACKGROUND=0` on top of `run-laptop.ps1`'s settings. World
reached, gameplay prompt at frame ~2743, 1.62 fps (748 ms per frame, CPU
438 ms) over 156 world samples, no crash, no device removal, no refused
pipeline, free RAM down to 9 MB. Several switches changed at once: an
indication, not a measurement of any one of them.

GPU memory architecture as the driver reports it: `uma=true`,
`cache_coherent_uma=false`, `isolated_mmu=true`. One physical pool, but
the CPU's caches are not coherent with the card: a CPU-written texture
path without staging would need write-combined memory, slow for the CPU,
which is the limit here. Not built.

## Later runs

- `bg-off` (`VICTORY_D3D12_BACKGROUND=0` alone): 735 ms per frame, 1.62
  fps, 300 s, no failure, but the log says the switch did **not** apply
  (`ok=false`): `SetBackgroundProcessingMode` needs Windows Developer
  Mode, which is off on this laptop. The frame time is therefore the
  build's (upstream speed PRs included), not the switch's.
- `ambient-06` (`VICTORY_AMBIENT=0.6`): 778 ms per frame, 1.67 fps, 330 s,
  no failure; dark regions +47-53 % over no ambient. 0.6 suits the dusk
  scene better than 0.3.
- Controller: device 0 (XInput slot 0) connects in every run; the pad is
  read every 8 ms with or without focus and the title takes the newest
  state each frame (E12): the lag is the frame time.

## Research done

Trackers and 18 repositories: `PORTWATCH_COMPARISON.md`,
`REPOSITORY_RANKINGS.md`. Candidates and outcomes:
`UPSTREAM_CANDIDATES.md`, `INTEGRATION_RESULTS.md`. Saves:
`SAVE_SYSTEM_AUDIT.md`. Picture: `RENDERING_IMPROVEMENTS.md`.

## Blocked on the user

- Building DexNom/VIctory (experimental rays) side by side: running a
  third-party tree's build needs the user's permission.
- The load after the prison intro (Prestream, upstream #1118): memory;
  other programs hold ~2 GB (Discord, ChatGPT, VS Code).

## 2026-11-26 implementation checkpoint

Current working branch: `experiment/780m-alias-memory`; runtime implementation commit: `63a247a87af6d15840bbef15789d13cea3317b1d`; latest test-only commit: `015931ce14e7a2e457f946efcdf14c0d1b9b38ea`. The expected `laptop/780m` branch remains at the prior `f52a6102d9e31e8588338b948a65e922a64baa34`; no merge or push. The starting working tree contained an alias test hook and new fixture; these were preserved and the fixture is now part of `559dc6101ab7513ffc81036298f86582bf339eab`.

Three focused commits implement: per-recording texture alias source identity; shared diagnostic submission fingerprint; released dynamic-slot reference counters. Synthetic old-path failure/new-path success was observed on the available AMD D3D12 adapter in synchronous and deferred queue modes. Workspace tests: 558 passed, none failed, 21 ignored. Diagnostic profile check and optimized release build pass. The fingerprint-only synthetic benchmark: warm medians: 5,838 μs/512 calls/24 MiB vs 2,880 μs/256 calls/12 MiB; cold-evicted medians: 45.5 μs/2 calls/96 KiB vs 34.7 μs/1 call/48 KiB. This is not gameplay performance. Candidate executable: `target\release\victory.exe`, source commit 015931ce…, standard release profile, SHA-256 F503C0D8506B339ABF75F866B9D017447087FF30CE604C0F0AC0843C9CDF04DD. Not launched; game remains closed.

The current renderer artifact is not confirmed fixed. Next: matched five-minute low-preset gameplay A/B after return; inspect the exact sky producer/view and capture matched raw resources; run diagnostic ownership telemetry and establish unique/sole owners before a cleanup trial. See `NEXT_STEPS.md`. Windows remained locked and settings unchanged; pre-build available RAM was 3,565 MiB.

## 2026-11-26 checkpoint — descriptor telemetry and bounded refresh candidate

- Branch `experiment/780m-alias-memory`; source commits `26a10c2738627763b10c41a9da9495968ee9321a` (diagnostics: unique resource refs and heap-exclusive released-slot ownership) and `3a656f1ba191e86014fa5e0024c04f8ec5ade8a9` (opt-in bounded disjoint refresh intervals). `laptop/780m` remains at `f52a6102d9e31e8588338b948a65e922a64baa34`.
- Released-slot telemetry compares COM resource identity counts across every slot in the same HostHeap. “Heap-exclusive” means the released range accounts for every descriptor slot reference in that heap; it does not mean there are no command-list, Arc, or other COM owners. Live `made_on` weak links count associations but do not retain a Shadow. No resource cleanup was implemented; fence/replay ownership remains unproven for all release paths.
- Refresh policy is runtime opt-in via `VICTORY_REFRESH_INTERVALS=2..32`; default `1` is the old hull. Synthetic 32x4 KiB workload at 512 KiB intervals, cap 16: request union 131,072 B; hull 16,257,024 B; bounded spans 8,454,144 B; synthetic span saved 7,802,880 B (48.0%). No measured runtime copy, CPU, memory or FPS benefit.
- Tests: workspace serial `cargo test --workspace --all-targets -- --test-threads=1`: 565 passed, 0 failed, 21 ignored. GPU diagnostics suite before the owner helper: 156 passed, 0 failed, 14 ignored; owner summary targeted test: 1 passed. Four existing dead-code warnings remain. `cargo fmt --all -- --check` reports extensive pre-existing workspace formatting diffs; no formatter was run. `git diff --check` passed.
- Latest remote startup had no persisted categorized log. Process telemetry showed severe startup private-memory growth and low available RAM, but the categories (staging, pipelines, descriptors, mapped backing, etc.) remain unattributed. Do not call this a gameplay/render regression.
- Normal release candidate: `outputs/VIctory-780m-candidate-3a656f1.exe`, SHA-256 `EF05BB13DB9676C4D73B41772676166F3C9B2B59CAD7D7694D32BACBAB76C96D`; source HEAD at build `3a656f1ba191e86014fa5e0024c04f8ec5ade8a9`, `--locked --release -p victory`. Existing release `target/release/victory.exe` and baseline `target-planes/release/victory.exe` were not overwritten; hashes remain as prior recorded. Candidate was not launched.
- Current exact state: repository clean after docs update commit is expected; do not merge/push. Windows settings unchanged, game closed. Next: collect diagnostic startup category logs on a user-authorized run; only try interval 16 after a matched default run and startup memory guardrail.

Diagnostic executable also prepared from the same source commit with `--profile diag -p victory --features diagnostics`: `outputs/VIctory-780m-diagnostic-candidate-3a656f1.exe`, SHA-256 `1C309CE2366CA951B0153EC45AE657EB425A444865952FB82BF080077754F06B`. It has two existing dead-code warnings and is for guarded startup attribution only; its overhead makes it unsuitable for performance claims. Neither candidate was launched.

## E26 — startup private memory and deferred list objects (26 November 2026)

Source commits: 17a7dd9059fab3ea25ea94ba3bf4aadc56779069 (opt-in lazy object, ownership telemetry, sampler), d2562a06250addbdd82604452698e5552486a00b (lock-free allocation category sampling). No game launched. Source defaults preserved; enable candidate with VICTORY_LAZY_LISTS=1 and VICTORY_LAZY_LIST_OBJECTS=1. It defers creation of ID3D12GraphicsCommandList until first use, while retaining an allocator and unchanged fence/replay ownership. It does not reclaim already-recorded command memory or solve the multi-GiB startup burst.

Three alternating runs of 64 production wrappers: existing private deltas 15888384/15921152/15876096 B, creation 5389/4947/5225 us; deferred deltas 1024000/1007616/1011712 B, creation 972/940/1015 us. Medians: 15,888,384 -> 1,011,712 B and 5225 -> 972 us. ~137 MiB deferred for 620 unused wrappers is an extrapolation only. Existing separate lazy-open fixture: 241 KiB per wrapper made vs 1338 KiB once opened, three repetitions. No gameplay result.

Original isolated fixture separated 64 allocators (~0.98 MB process private, ~0.875 ms) from 64 unopened command lists (~14.89 MB private, ~4.416 ms). Process deltas are not exclusive ownership attribution. Never-reset counts cannot be called never-recorded. Pool is capped at 128 completed lists/256 MiB scratch; detached count includes unsubmitted held recordings and is not exclusively in-flight. No descriptor cleanup: latest run had zero same-heap exclusive released-slot resources. No aggressive pool/allocator retirement.

VICTORY_ALLOCATION_LOG=<absolute file> persists diagnostics independently of presents, 1 s normally / 250 ms below 1 GiB. Timestamps, private bytes, total working set, available memory, remaining commit, live Rust allocation/staging/shadow/host-texture bytes, list counters and detached count. Categories may overlap; not a total private-memory decomposition. External outputs/monitor-victory-memory.ps1 runs GlobalMemoryStatusEx guard independently of slow detail queries, collecting private working set, commit and paging in a worker. A 5 s delayed worker did not delay the guard: sustained-low test gaps 976-1025 ms; immediate-256 test gaps 995-1008 ms. Both simulated threshold tests stopped; no real pressure induced or applications closed.

GPU deferred-object regression verifies first-use copies, two completed-fence replays, reset, second epoch and exact contents; ownership counter resets/drops tested. Opt-in diagnostic GPU suite 160 pass/0 fail/16 ignored. Full serial suite 566 pass/0 fail/23 ignored; final category-only follow-up reruns recorded in outputs/startup-memory-development.md. Existing dead-code warnings retained. The terminal startup burst is still unattributed; title footprints do not prove committed memory. Final 5-second category gap cannot be reconstructed.

Current system optional groups measured private WS: Discord 422 MiB, Claude 275, Steam+web helpers 85, Epic components 73, Teams 29. Suggest manual normal Exit only after saving and confirming idle; hypothetical 0.3-0.6 GiB recoverable, measure before/after. Exclude Codex, security, services and unclear ChatGPT ownership. No causal attribution possible for old 5.1 -> 2.9 GiB available-memory difference without contemporaneous process baselines. Full report, benchmarks and guard results are outside repo in outputs/startup-memory-development.md and JSONL/result files. Keep original executable hashes and earlier 3a656f1 candidates protected.

## Final E26 builds and validation

Both binaries were built from clean runtime source commit d2562a06250addbdd82604452698e5552486a00b (documentation changes only afterward), outside the repository target directory, with --locked and two cargo build jobs. Neither executable was launched. Opt-in lazy-list objects is OFF by default; use VICTORY_LAZY_LISTS=1 and VICTORY_LAZY_LIST_OBJECTS=1 only in the next authorized controlled comparison.

- Diagnostic: outputs/VIctory-780m-startup-diag-d2562a06.exe; SHA-256 808ED844B8EDA0A5C0B3BADF05D338861769F135A7EE3A9142AA226A961CAAD3; --profile diag --features diagnostics. Two existing dead-code warnings.
- Release: outputs/VIctory-780m-startup-release-d2562a06.exe; SHA-256 D487346578EB2F51A8659D92BD1C3870CBEE4821BB61B003D54597F33D6E7670; --release. Four existing dead-code warnings.
- Final full serial workspace suite at d2562a06: 566 passed, 0 failed, 23 ignored. Final opt-in diagnostics GPU suite: 160 passed, 0 failed, 16 ignored. Hardware tests used synthetic data only. No gameplay metrics collected.
- Final guard sustained-low test with 5 s delayed detail worker: 6 samples, gaps 987.72-1013.61 ms, stop at 5001.95 ms. Immediate-threshold test: 4 samples, gaps 995.71-1003.35 ms, stop at 3005.26 ms. The guard now pins the target process handle against PID reuse and ends the authorized VIctory process at timeout/error, rather than leaving it alive unguarded. Runtime category sampler produced 1000 ms-separated samples in the final category fixture; low-RAM cadence tested by policy unit test only.
- Existing target/release and target-planes/release hashes remain F503C0D8506B339ABF75F866B9D017447087FF30CE604C0F0AC0843C9CDF04DD and 03F66A388BC0FA2AEC2AE8C3A6D04F52B0312515ECEBFAEB112FDBEB4CFCCD29. Earlier 3a656f1 candidates also hash-match their saved records.
- Terminal multi-GiB private burst, precise title/driver/pipeline allocation ownership, per-process hard faults, and complete recorded/executing/replay phase accounting remain unresolved. No descriptor cleanup, pool retirement, Windows changes, unrelated process closures or gameplay launches. Bounded refresh remains opt-in and unmeasured in gameplay.

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

## Cycle V allocation sampler and Test #28

Runtime source `3f9ac0ae59bba5e8516c277efa18ad46ef96ef96` on `experiment/780m-alias-memory`. `laptop/780m` remains `f52a6102d9e31e8588338b948a65e922a64baa34`. The diagnostics-only `VICTORY_GPU_ALLOCATION_LOG` worker targets 200 ms independently of presentation, with bounded large-lifetime events and logical-submission/native-fence correlation. Test #28 executable SHA-256 `29C13F047FF5B5A861A930570D34E7835ABE8AEECF702F0A29488C6896D3FCED`.

Test #28 reached 235.801 s before the independent emergency watchdog stopped it. Minimum observed available RAM was 27.88 MiB. Initial available RAM was 8,914 MiB with no app closures. The observer's hypothetical normal stop was 231.814 s. The sampler persisted 1,172 observations with a 200.309 ms median gap, 467.274 ms maximum gap and zero reported producer drops through the final sample. The first-world preparation interval increased private commitment by 3,394.85 MiB, DXGI local usage by 3,043.51 MiB, logical Shadow spans by 2,539.58 MiB, and tiled heap capacity by 944 MiB. DXGI local usage peaked at 4,715.70 MiB against 7,542.02 MiB. The final logged frame had 472 draws. Visible gameplay and character control remained unverified. No repeated launch was performed.

The final frame census measured 1,932 MiB of loose windows over 1,040 MiB of title addresses. Overlap is logical. The next comparison must identify unique mapped backing and compatible owners to test whether those bytes can be reclaimed. Only one of 50 samples below 512 MiB had an incomplete signaled submission. This does not support fence-only backpressure for the captured burst. Scratch buffers, textures, card copy backing and Shadow spans are separate trends with overlapping accounting. The protected branch and baseline executables were preserved.

Serial workspace 576 passed, 0 failed, 26 ignored. GPU diagnostics 168 passed, 0 failed, 18 ignored. HLE diagnostics 124 passed, 0 failed, 4 ignored plus one integration. Counts overlap. Separate sampler persistence and retirement fixtures passed. Full report and provenance under `C:\Users\Arnou\Documents\Codex\2026-11-25\ana\outputs\cycle5\`, with Test #28 raw logs in `outputs/cycle3/test28-20261126-212530`.

## Host-copy window backing after Test #28

Runtime commit `34d79c8abb2443a91cbbdbf8dd9fcda68e71dc72` adds opt-in backing identities and `VICTORY_MAPPED_GEOMETRY_WINDOWS=1`. It keeps existing descriptor, recorded-list and heap ownership, and restricts mapped-window preference to bounded sized vertex and index reads. Test #28 did not record per-title-address heap slots, so the independently backed share of its 892 MiB logical overlap cannot be recovered. Its 953 MiB of occupied tiles against 956 MiB of heap capacity indicates little free pool capacity at the final sample.

Three matched synthetic pairs held the same 32 MiB of logical overlap and reduced occupied backing, heap capacity, resident CPU mappings and DXGI local growth from 64 to 32 MiB. Process private-commitment growth fell by 32.08–32.14 MiB. The copy-back and concurrent GPU-read fixtures passed. Test #29 was not launched. Real workload incidence and sufficient physical headroom remain open. Detailed report and raw fixture traces are in `C:\Users\Arnou\Documents\Codex\2026-11-25\ana\outputs\cycle5\`.

## Cycle V final state

Test #29 reached partial world preparation and ended on the emergency watchdog at 178.705 s. Test #30 was held after the 4.824 MiB minimum and zero eligible geometry requests. See [Cycle V final handover](CYCLE_V_HANDOVER.md) for exact build provenance, attribution, and hardware testing.
