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
