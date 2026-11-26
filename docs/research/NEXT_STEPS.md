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
