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
