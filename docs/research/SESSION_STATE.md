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
