# Cycle V final handover

## Decision and build

Test #30 was held. Test #29 exhausted physical RAM despite starting with 9,479 MiB available, and the instrumented baseline found zero eligible tile-saving requests for the synthetic geometry optimization. A subsequent snapshot showed about 10,038 MiB available. That additional headroom does not establish safe completion of the first world frame. The last sample still had 2,261 draws, far short of the historical 7,350–7,550-draw world frames. The normal guard would have stopped at 171.013 seconds. Keep the existing independent emergency thresholds.

The diagnostic candidate is source commit `cacbb3597fc65b01c1146f678117a5ef519f2b2f`, built with `cargo build --locked --profile diag -p victory --features diagnostics`. Executable SHA-256 is `540EA15A6378445EABD13C23969A5AA41004D9A5AD949D72A70E360BCA6E311A`. Earlier build hashes, including the Test #28 control, are retained in the local build provenance reports. The mapped-geometry selector and detailed attribution remain opt-in. Keep `laptop/780m` at `f52a6102d9e31e8588338b948a65e922a64baa34`. Its backup is `backup/laptop-780m-pre-cycle-v`. Cycle V provides diagnostic improvements and a synthetic win, with no validated production improvement to integrate.

## Test #29

| Measurement | Result |
|---|---|
| Configuration | Lowmem, ray tracing off, relay 2, texture no-wait, 256 MiB card cap, warm pipeline cache, selector off, 200 ms sampler and backing trace |
| Startup / first-world preparation | 161.362 s to the first sample above 1 GiB of logical Shadow spans |
| Runtime / minimum available RAM | 178.705 s / 4.824 MiB |
| Peak private commitment / system commitment | 10.73 GiB / 28.23 GiB |
| Peak DXGI local usage and budget at peak | 5.31 GiB / 6.39 GiB |
| Late tile occupancy / allocated capacity | 1,498.25 MiB / 1,500 MiB |
| Logical overlap / observed independently backed tile overlap | 1,077.25 MiB / 578.75 MiB, sampled maxima |
| Geometry activation / eligible tile-saving requests | 0 / 0 |
| Peak recorded incomplete submissions | 15 in a 200 ms sample |
| Peak sampled outstanding copy footprints | 29.86 MiB of texture staging and 16.75 MiB of card uploads, separately sampled |
| Peak observed tile backing awaiting return | 0.19 MiB |
| Peak texture preparation | 68.3 ms in the terminal samples |
| Last sampled CPU preparation / GPU wait | 1,346.6 ms / 309.0 ms at 2,261 draws |
| World frame / verified gameplay | No sampled full-world draw frame. Zero verified gameplay. Gameplay FPS and frame-time percentiles unavailable. |
| Event loss and termination | 5 sampler drops, 3 unpaired events. Emergency physical-memory stop. Zero logged texture allocation failures or device errors before stop. |

Of 37,776 geometry decisions, 28,616 found containing windows with no tiled candidate and 4,558 resolved to registered buffers. The recorded independently backed overlap is credible in scale and incomplete in precision. It is unrelated to the 32 MiB synthetic A/B result unless the same owner and read-range conditions are established. The pool was close to full occupancy. The dominant measured growing backing was titled Shadow windows and their mapped tiles. Render-target and UAV textures also rose. These allocations, DXGI usage and private commitment describe different accounting layers and can overlap. The terminal condition was physical RAM exhaustion while DXGI still reported budget headroom.

There is little evidence that evicting empty pools or retiring completed submissions would recover hundreds of MiB. Avoid E17 live rebinding and E18 broad descriptor-window reuse for this pressure case. The next evidence-supported investigation is to attribute duplicate tiles by descriptor access, title-memory generation, bound window and recording owner. In a synthetic fixture, select only a compatible already-mapped resource at creation of a new read-only descriptor. Preserve captured descriptor ownership, alias refresh, replay and writable isolation. Proceed to an authorized runtime comparison only after showing that the measured hot path is eligible and that the expected improvement can materially change available physical headroom.

## Historical comparison and engineering audit

Cycle V started from `61746ff70e980bee6a2752796bed179813662d4a`. The six commits through `cacbb359` changed 12 files with 885 additions and 31 deletions. Ten are Rust source files, including fixtures. The other two are research checkpoints. No new merge or upstream import was made in this range. Other local branches and worktrees were preserved. The final handover documentation follows the tested source commit.

| Test | Runtime | Outcome |
|---|---:|---|
| 1–4 | Unknown | Historical numbered mapping remains unresolved |
| 5–8 | 38.903 / 72.711 / 47.913 / 55.010 s | Startup only, from the earlier audit |
| 9–10 | See Cycle II | Later card-cap A/B and default test. Distinct from the earlier runs withheld with those labels |
| 11–22 | See numbered data | Loading-stage tests. 18 and 20 failed before launch |
| 23 | 171.7 s | Device removal during loading |
| 24 | 386 s | First world-draw samples at approximately 257 s. About 129 s of world submissions, then GPU OOM |
| 25 | 243 s | First world-draw samples at approximately 172 s. GPU OOM |
| 26–28 | 53.8 / 103.3 / 235.8 s | Physical-memory safety termination. No verified world control |
| 29 | 178.7 s | Partial-world preparation and emergency stop |
| 30 | No launch | Headroom and eligible savings unproven |

An unnumbered earlier run achieved 2.2 FPS in sampled world submissions and started the world at approximately 167 seconds. Visible control for that run was unverified. The earlier E4 record reports 340 seconds of verified interaction in a 420-second runtime. The historical playable peak sampled across earlier records is 2.1 FPS. Cycle V did not surpass the verified gameplay milestone. The existing lowmem profile is the most evidenced world-loading configuration. Current candidates remain short of repeatable visible gameplay. The black-world, lighting, sky and final-composition questions remain open. On the 780M the D3D12 capability query succeeded for mesh and shader model 6.6. The runtime handles translated triangle mesh pipelines and does not accept amplification stages in the host pipeline stage filter. This alone does not identify the black-screen cause.

Validation on the final runtime source was 577 passed, zero failed and 26 ignored for the serial workspace, with 169/0/19 GPU diagnostics and 124/0/4 HLE diagnostics plus one integration. Suites overlap. Earlier separate fixtures verified buffer transitions, overlapping reads and writes, copy-back and retirement. Three optimized synthetic pairs reduced physical tiles, resident CPU mappings and DXGI local usage growth from 64 to 32 MiB without changing the 32 MiB logical overlap. These are different measurements of the same backing and cannot be added. No live memory or FPS improvement was validated.

The initial Windows audit found 16.00 GiB installed, 15.30 GiB usable, 5.98 GiB in use and 9.31 GiB available. An estimated 2.28 GiB was attributable to Windows processes and resident kernel and driver counters. Codex's observed private resident tree used 0.92 GiB. Claude was absent. The idle DXGI probe had a 7.37 GiB local budget and zero usage in the probe process. No services, pagefile, compression or other Windows settings were changed.

## Tomorrow's controlled comparison

Use only an authorized local workload. Archive the exact executable, SHA-256, source revision, runtime switches, GPU driver, Windows version, resolution, save and route identifiers, power state and cache state. Build the same source on all three machines. Use `cargo test --locked --workspace --all-targets -- --test-threads=1` once and verify both the baseline and diagnostic executable hashes. Reuse an independent runner that samples physical availability every 100 ms, records termination and enforces the established thresholds. The laptop workspace already has the Cycle V guarded runner.

| Hardware | Baseline for the comparison |
|---|---|
| Ryzen 7 8840HS / Radeon 780M / 16 GiB | Existing lowmem, 256 MiB card cap and ray tracing off. Keep the geometry experiment disabled. Preserve non-cache-coherent UMA synchronization. Record system RAM and DXGI independently. |
| 12th-gen Core i5 / RTX 4060 8 GiB / 32 GiB | Same low graphics, ray tracing off and matched workload. Verify the actual local budget and nonlocal spill. Keep UMA-specific overrides disabled for an independent discrete baseline. |
| Ryzen 7 9800X3D / RTX 5070 Ti 16 GiB / 32 GiB DDR5-6000 | Same scene and settings first. Measure CPU submission separately from GPU execution. A later quality or resolution comparison changes one setting at a time. |

Use one warm-up and at least three comparable measured repetitions per supported condition. Record cold and warm results separately. Verify present output, world visibility and actual input response. Mark title loading, first-world preparation, first world frame and time of successful control independently. Sample physical RAM, private commitment, working set, paging, local and nonlocal DXGI usage and budgets, and outstanding GPU work. Use lightweight per-present timings for frame-time percentiles and report GPU fence waiting separately from timestamped execution. Run the detailed allocation trace as a distinct interval if its overhead is material. No desktop performance is measured yet.

The workspace report set contains the numbered CSV and JSON, graphs, initial Windows audit, raw Test #29 trace summaries and executable provenance. Only source and handover documentation belong in Git.
