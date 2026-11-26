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
