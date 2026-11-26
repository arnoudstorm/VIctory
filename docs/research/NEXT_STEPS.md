# Next steps

0. **Past the prison intro (E19):** the post-intro load dies with the title's TileHeap full (3,977/3,977 MB). Find the texture-streaming feedback the title reads back from the GPU and check it is right; a wrong read-back asking for the finest mips everywhere would fill the pool (and lower-detail textures would follow from fixing it).

1. **Background recompiles off** needs Windows Developer Mode (`SetBackgroundProcessingMode` returned an error without it); with it on, rerun `bg-off` and check `ok=true`.
1b. **Isolate the combined run's gain** (748 ms per frame in `all-new`):
   one run with only `VICTORY_D3D12_BACKGROUND=0` added to the baseline,
   against the ten-run baseline (mean 1,073 ms, range 759-1,605).
2. **The remaining waits** (5 per frame, 170-250 ms): 2-2.5 are "another
   copy" (a refresh of one window while a flight writes an overlapping
   one): the duplicate windows again (E14, E17). Fewer duplicates means
   fewer waits; a safe merge must go through every holder of a window.
3. **Ambient level**: 0.6 measured (+47-53 % in dark regions); next a warmer tint for dusk and the shade strength (`VICTORY_AMBIENT_SHADE`).
4. **Skipping the intro** with held A (`hold_a_keys.txt` in the session
   notes), with other programs closed, to see whether the Prestream load
   fits in memory now.
5. With permission: build DexNom side by side and run it once normally,
   once with `--rt` settings, as Series X, with its own data folder.
6. Belobug `03629f3f` (quad reads, AMD helper lanes) after a GPU pixel
   test that shows the defect.

## Current implementation checkpoint (2026-11-26)

The alias synchronization candidate, synthetic failure/pass fixture, shared diagnostic fingerprint and descriptor-reference telemetry are implemented in three local commits on `experiment/780m-alias-memory`; `laptop/780m` is not changed. Exact commit IDs, test totals, synthetic metrics and unresolved conditions are recorded in `SESSION_STATE.md` and `RENDERING_EXPERIMENTS.md`.

The next useful step is a matched, capture-off five-minute low-preset gameplay A/B with identical title binary/save/scene/input and at least three repeated segments, followed by matched captures of the exact reported sky if it remains defective. Stay under the previous abort thresholds. Keep A/B runs separate from diagnostics and do not add performance assertions to the synthetic GPU fixture. The read-only Windows preview script and existing Windows checklist can be used to inform manual, reversible app exits, but do not close active work or alter services/settings. Before any dynamic-descriptor cleanup, gather unique resource identity and all list/flight/retired owners; the current counters alone are insufficient.

## Benchmark and release follow-up

A fourth commit, `015931ce14e7a2e457f946efcdf14c0d1b9b38ea`, changes only the ignored diagnostics benchmark to alternate seven warm batches and nine 64 MiB cache-evicted approximation samples. The final measured medians and limitations are in `RENDERING_EXPERIMENTS.md` E24 and the performance report. The complete 558-pass workspace suite preceded this test-only harness refinement; the updated ignored harness itself was compiled and passed. The runtime source commits remain the first three listed above.

Final optimized release rebuilt at HEAD `015931ce14e7a2e457f946efcdf14c0d1b9b38ea`: `target\release\victory.exe`, SHA-256 `F503C0D8506B339ABF75F866B9D017447087FF30CE604C0F0AC0843C9CDF04DD`. The binary was never launched. The four existing dead-code warnings remain.

## Next implementation and return-to-laptop checklist

1. Keep using `experiment/780m-alias-memory`. Do not merge or push. Keep the current release candidate and baseline hashes from `SESSION_STATE.md` as the protected references.
2. On return, do one user-authorized baseline run first with `VICTORY_REFRESH_INTERVALS` unset (default hull) and existing `run-laptop.ps1 -Preset low -Safe -NoCapture`. Capture categorized diagnostics to a persistent log and sample process/system memory at 1 s; abort at the previously documented guardrails. Stop if startup pressure repeats before input/gameplay.
3. Only if baseline is safe and produces comparable actual gameplay, repeat the same scene/save/input/settings/capture mode with `$env:VICTORY_REFRESH_INTERVALS='16'`. Compare refresh requested bytes, range span/copy traffic, CPU preparation time, GPU wait and frame time; reset environment between runs. A capture-on run is not a performance run.
4. Do not release descriptor resources yet. Review the new per-second diagnostic counts first; ensure reset, detached lists, replay, multiple same-heap owners and all queue/fence paths before a cleanup change.
5. Keep Windows preparation reversible: use `outputs/preview-optional-app-memory.ps1 -DryRun` (or its documented preview invocation) to inspect optional apps; manually close only apps you recognize and can reopen after saving work. Do not close Codex, security tools, services or system components. No settings were changed in this session.

Candidate executable: `outputs/VIctory-780m-candidate-3a656f1.exe`; SHA-256 `EF05BB13DB9676C4D73B41772676166F3C9B2B59CAD7D7694D32BACBAB76C96D`; source commit `3a656f1ba191e86014fa5e0024c04f8ec5ade8a`. Candidate has the new interval option, default off. It was built but never launched.

## Exact guarded commands for the next authorized session

Run the diagnostic binary first only when the user has returned and authorized launch. Set `$gameDir` to the installed folder that contains the title's `game_*.exe`. Keep the laptop plugged in and use the same low preset and save. The log is written to the local Victory data folder. Start with interval mode 1 (the unchanged hull):

```powershell
$repo = 'C:\Users\Arnou\VIctory-laptop'
$gameDir = Read-Host 'Folder containing game_*.exe'
$diagExe = 'C:\Users\Arnou\Documents\Codex\2026-11-25\ana\outputs\VIctory-780m-diagnostic-candidate-3a656f1.exe'
Get-FileHash -Algorithm SHA256 $diagExe
$env:VICTORY_LOG = 'victory_gpu=info'
$env:VICTORY_REFRESH_INTERVALS = '1'
$log = Join-Path $env:LOCALAPPDATA 'victory\diagnostic-startup-default.log'
& (Join-Path $repo 'run-laptop.ps1') -GameDir $gameDir -BuildPath $diagExe -Preset low -Safe -NoCapture 2>&1 | Tee-Object -FilePath $log
```

If startup stays above the memory guardrail and a comparable scene is reached, run the **same binary** with `$env:VICTORY_REFRESH_INTERVALS = '16'` and a different log filename. For frame-time A/B, use the normal release candidate for both runs, with the same command and environment values, no capture; change only the interval variable between runs. Collect at least 1 s memory samples externally and stop at the existing guardrail. Do not use a diagnostic run for performance comparisons. The commands are prepared only; nothing was launched in this session.

## E26 next authorized startup comparison

Keep game closed until explicitly authorized. Use new d2562a06 candidates with the one-second external outputs/monitor-victory-memory.ps1 guard, not the old counter-based loop. Set VICTORY_ALLOCATION_LOG to an absolute persistent file and persist console stdout/stderr. VICTORY_LAZY_LISTS=1; compare VICTORY_LAZY_LIST_OBJECTS=0 vs 1; keep VICTORY_REFRESH_INTERVALS=1, low/safe preset, same title/save/cache/power. Do not combine refresh-interval experimentation with lazy-list startup A/B. Avoid the former memory-map walk for the lightweight attribution run (VICTORY_MEMORY_MAP=0 in both arms); if needed, run a separate explicit map trace and label its overhead. Benchmark logs are in workspace work/lazy-object-ab.log, unopened-objects.log, and category-sampler-tests.log. Two guard self-tests and optional app snapshots are in outputs. Full measured limitations and source references: outputs/startup-memory-development.md. The burst remains unknown; do not claim startup fixed.

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

## E30 current next steps — supersedes older launch queues

Run #6 consumed the authorized launch and ended on the memory guard at72.711s. Do not restart automatically. Await another controlled-test authorization. Current experimental runtime sourceff3d88a7399dde2ede1ade64edd8b5e7b1ef1463; final external release/diagnostic candidates and hashes in phase6-final-build-provenance.json. Release identical to f69d42f Run6; final logger correction affects diagnostics only. Baselines preserved, laptop/780m unchanged.

1. Before another test, save work and manually confirm idle optional apps. Fresh privateWS:Discord430.8MiB,Claude173.4,Steam90.3,Epic25.0,Teams25.2. Combined744.7MiB is accounting/planning only; hypothesized0.25-0.5GiB physical recovery is unmeasured. Keep Codex/security/drivers/services; no auto closures.
2. Use corrected coarse VM logger with5s region interval in a separately labeled diagnostic run, not all expensive diagnostics together. Correlate complete37-55s snapshots, title VM flags/callers and CPU mapped address allocation bases. Four large title arenas remain semantically unchanged; burst remainder not assigned to driver.
3. Add bounded low-frequency DirectStorage per-role allocation/retained/high-water counters and sum texture scratch lifetimes before capacity policy. Pool reuse synthetic savings are allocation traffic only, defaultoff; memory footprint can rise16MiB.
4. Investigate texture refresh/remap lock duration using synthetic concurrent mapping/replacement/stable-read tests. Do not remove guard or repeat E17 window merging.
5. Resume actual input/movement/world verification first, then matched release reuse0/1 or refresh1/16, one flag per A/B, same caches/settings/power/route and100ms safety. Need per-present timings for percentiles. Previous5:40gameplay record unchanged; Run6gameplay0.

Exact next-run preview and guarded launch instructions are outside repo in outputs/phase6-next-test.ps1 (default preview only; -Run only after next authorization). Full findings/limitations: outputs/phase6-development-and-run6.md. No further gameplay milestone claimed.

## Current queue after Phases 7–10

Final runtime source f43ae7fbaeb604679f022f3052930f0eaee5161b. Run7/8stopped on immediate memory guard,0gameplay. Runs9/10withheld under repeated-danger condition; no Run11. Preserve final unlaunched candidates/provenance and all prior binaries. Protected laptop/780m unchanged. Current dataset includes unknown Run1–4 identifiers instead of mapping historical tests by guess; no missing values plotted as zero.

Next work: establish actual safe physical headroom (normal optional-app Exit only if idle verified; no forced closure); close native terminal-allocation coverage/complete-snapshot gap; correlate second174MiBregion by identity, not size; keep arena semantics and GPU/replay/fence lifetimes. Existing DS retained capacity is small and mutex acquisition negligible in Run7; do not enlarge caps or claim scratch pooling fixes startup. Keep copied-read unlock defaultoff until actual-workload incidence and correctness measured. Add targeted replacement/replay/error fixtures before enabling. Resume input/world proof before gameplay A/B; no per-present FPS percentile currently available. Historical340s/2.1FPS remain historical. Full audit/roadmap outside repository.
