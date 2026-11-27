# Cycle VI: cross-fork integration, performance and rendering

Started 2026-11-27 on the Radeon 780M laptop (Ryzen 7 8840HS, 16 GB shared,
Windows 11). Labels used below: **measured** (from our run logs),
**reported** (another fork's own figures, on its hardware), **expected**
(our reasoning, untested here), **observed** (seen in captured frames, one
run).

## Phase I: state, repositories and what differs

### Our state at the start

| | |
|---|---|
| Working branch | `experiment/780m-alias-memory` at `2e6645ec`, tree clean, pushed to `mine` |
| Backup | `backup/780m-alias-memory-2e6645ec` (local; pushed with this document) |
| Integration branch | `experiment/cycle-vi-cross-fork`, from `2e6645ec` |
| Protected baselines (untouched) | `backup/laptop-780m-pre-cycle-v` `f52a6102`, `laptop/780m` `f11964e7`, `main` `70564ae0` |
| Last run build | `VIctory-780m-test40-2e6645ec-diag.exe`, SHA-256 `7C5C5886…0784` |
| #38/#39 build | `VIctory-780m-test38-924e2679-diag.exe`, SHA-256 `F88E3CAC…E211` |
| Regression suite at `2e6645ec` | 578 passed, 0 failed, 26 ignored (`cargo test --workspace`, 15 test binaries) |

The brief says 581 passing tests; the last full run on this branch counted
578 (plus 26 ignored). The difference is not explained yet; the next full
run will be recorded with its per-binary counts.

### Repositories (fetched 2026-11-27)

| Repository / branch | Tip | Date | Merge base with ours | Commits not in ours | Ours not in it |
|---|---|---|---|---|---|
| Marvisak `main` (upstream) | `63bcca43` | 11-26 | `ffe310eb` | 479 | 167 |
| arnoudstorm `main` | `70564ae0` | 11-24 | `ffe310eb` | 28 | 167 |
| Belobug `main` (Belobug's own line) | `4f62c2a5` | 11-25 | `8e79c80e` | 550 | 179 |
| Belobug `a-publier` (default branch) | `4eb6cfdf` | 11-26 | `a803c87c` | 611 | 152 |
| Belobug `rx9070xt` | `cd8dfc06` | 11-26 | `a803c87c` | 562 | 152 |
| avatarmanz `main` | `014d52de` | 11-26 | `a803c87c` | 579 | 152 |

Two different lines live in Belobug's repository:

- **`main`**: Belobug's own work on an RX 9070 XT, from upstream
  `8e79c80e`. Compared with ours on 2026-11-25 (`COMPARE_BELOBUG.md`,
  then at `01aadd19`); four changes were taken then
  (`INTEGRATION_RESULTS.md`: integer-target blending, the ambient light).
  36 commits are new since, among them romainhedouin's PRs #24-#34 on the
  CPU cost of a draw.
- **`a-publier`** (the default) and **`rx9070xt`**: the
  fxceless → avatarmanz line on an RX 7900 XTX, which also carries
  Belobug's and other forks' changes. Its fork point `a803c87c`
  (fxceless, 2026-11-24) is already in our branch. `rx9070xt` is an
  ancestor of `a-publier` (49 commits behind). avatarmanz `main` is
  `rx9070xt` plus 17 commits; 14 of them were merged into `a-publier`
  (`1e728610`), and the last three (`84917c34`, `d13e9e0c`, `014d52de`:
  renderer port RT, part 1) are only in avatarmanz.

Of `a-publier`'s ~440 non-merge commits since `a803c87c`, 74 are already
upstream (same patch), 7 are already in ours, and **about 362 are new to
us**: 393 authored as "local" (avatarmanz's Claude Code sessions), the
rest by fxceless, Vyacheslav Venglovsky, gomeaw, feanorsun, fever308,
spacede, ryotdev, sawetco, Duff, quaternion and others. The diff from the
fork point is 218 files, +99,402/−4,175 lines, mostly in `victory-gpu`
(`host.rs` +6,073, `watch.rs` +5,474, `descriptor/host.rs` +4,016, a frame
recorder/replayer, `shadow.rs` +3,295).

All five trees are GPL-3.0-or-later. Every change taken keeps its author
(cherry-pick with `-x`, or "ported from <repo> <commit> (<author>)" in the
message when re-written by hand).

### Their figures and what they mean here

- `a-publier` (README, i9-12900K, 64 GB, RX 7900 XTX, ray tracing off):
  **reported** 45 ms a frame (~22 fps) in a city street with renderer
  patches 1, 2, 5, 6, 7, from 57-75 ms before them and 280 ms three days
  earlier. Their own figure for memory: **about 30 GB private (40-46 GB
  in `docs/findings.md`), 10.5 GB VRAM, 64 GB of RAM recommended.** One
  submitting thread (~40 ms) sets the frame.
- Community reports in that README: RX 9060 XT 16 GB 19.8 fps (Series X
  interior); RTX 3090 545/532 ms a frame (Series S, warm); Arc B580 0.7 fps
  in menus, then device removal.
- `belobug/main` (RX 9070 XT, 9800X3D, 64 GB): 3-5 fps, ~22 GB RAM,
  commits ~33 GB after 15 min (from `COMPARE_BELOBUG.md`).

The laptop has 15.3 GB visible in total and our runs peak at 12-14.7 GB
private with available RAM touching 0 MiB. **Neither line can be run
here as it is**, and a merge of either is ruled out. What can be taken
is single changes that cut CPU time without adding memory, and
correctness fixes.

Most of their gains attack what we pay too: noticing and moving writes
between the title's memory and the card's (write watch, guards, hashes,
copies) and the one submitting thread. On a UMA chip like the 780M part
of that moving is avoidable altogether (the card reads system memory),
which makes their "use it in place" changes (renderer port 1) the most
interesting for this machine; **expected, not measured**.

### Overlap: what each side has

| Topic | Ours | `a-publier` / `main` |
|---|---|---|
| Shared physical tiles (`VICTORY_SHARED_TILES`) | yes (`4e9052e7`) | no |
| Tiled windows (`VICTORY_TILED_WINDOWS`) | yes | no |
| Structurizer memo cap (`dc567b92`, not a switch) | yes | no |
| Compile gate on free RAM (`VICTORY_COMPILE_MIN_AVAIL_MB`) | yes | no |
| Shader cache prewarm (`victory prewarm-shaders`) | yes (`2e6645ec`): translations | `victory warm` (`b7ab9c49`): driver pipelines (PSOs). Complementary |
| Pipeline workers | `VICTORY_PIPELINE_WORKERS` (1-8, `2ded7c88`) | follows the CPU (`ffdde86e`), slow pipelines on a quarter of them (`a30829c4`) |
| Alias-aware texture guard | `VICTORY_ALIAS_GUARD` (`be1ca828`) | "a write through one view is seen at its other views" (fxceless `8dd30ab4`, myokex line) |
| Ambient stand-in for indirect light | older port (`b7a122d`, from Belobug `06031237`…) | follows the sky by default (`705a2d09`, `e48eb83e`, `4f62c2a5`); `VICTORY_AMBIENT_RT` |
| Loop limit (`VICTORY_SHADER_LOOP_LIMIT`) | yes, 32 in our lowmem preset | yes |
| Emergency observer | ours (outside the repo, runner) | none |
| Low-memory preset | `-Preset lowmem` (launcher) | none; 64 GB assumed |

### Which candidates apply to our tree

Every candidate was cherry-picked alone onto `2e6645ec` in a throwaway
worktree (`--no-commit`, then reset). **Clean: 7 of 57.** The rest conflict,
mostly in `host.rs`, `draw.rs`, `shadow.rs` and `watch.rs`, which both
lines rewrote. Integration is therefore a series of hand ports in
dependency order, not merges.

### Comparison table

Order = recommended integration order. "Their effect" is reported on their
hardware; "here" is expected.

| # | Feature | Source, commit(s), author | Files | Ours now | Theirs | Perf. effect | Memory effect | Rendering risk | Depends on | Applies |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | **A/B of our alias guard** | ours `be1ca828` | `texture.rs`, `inplace.rs` | on in #39/#40 | — | **measured**: refreshes 54-101 → 863-1,282 ms/frame with it (association) | none | flicker if off (start screen) | — | ours |
| 1 | Thin LTO, one codegen unit (release) | Belobug `main` `a23d1b46` (Key, PR #96) | `Cargo.toml` | none | release profile | a few % CPU, **expected**; our test builds use `diag`, so it needs a `diag` twin to measure | none | none | — | clean |
| 2 | Re-registered title buffer retires its old copy | Belobug `main` `40b8b9b5` (beloooo) | `shadow.rs` | dropped at once | retired after the lists | none | slightly more held, briefly | fixes a use-after-free class | fences (we have) | clean |
| 3 | Draw path: pipeline looked up by address, heap lookup without the pending lock | `92d353f8`, `b022d860` (romainhedouin, PRs #25, #27) | `draw.rs`, `host.rs`, `descriptor.rs` | QueryInterface ×2 + locks per bind | one lookup | CPU per draw down; 6-7k draws/frame here | none | low | — | clean |
| 4 | Draw path, rest of PRs #24-#34: unaligned root CBV once per list, clamp setup once per list, staging from pooled scratch, write ranges declared once, quick table hash, stack root constants | `84ea9c12`, `ab5ece8f`, `5e08eee0`, `dd799aad`, `b41f3f3f`, `e6b63862` (romainhedouin) | `host.rs`, `draw.rs`, `indirect.rs`, `list.rs` | per-draw | per-list | CPU down (our wall: CPU 270-1,500 ms/frame) | `5e08eee0`: fewer committed upload buffers (down) | low-medium (draw state) | #3 | conflicts (hand port) |
| 5 | Parallel copy refreshes | `a-publier` `05ee4387` (spacede) | `host.rs` | serial | threads | targets our refresh time directly | none | ordering of refreshes | — | clean |
| 6 | Indirect scratch buffer stays readable | `cc11cccc` (Vyacheslav Venglovsky) | `draw.rs`, `host.rs`, `indirect.rs` | barrier pair per execution | stays readable | GPU barriers down | none | low | — | clean |
| 7 | AMD wait instructions patched in place at first fault; clzero loops as stores | `21323661`, `13ffc283` (avatarmanz) | `loader/console_insn.rs` | emulated at every fault | patched once | CPU fault time down on title threads | none | title code patched (byte-checked) | — | `21323661` clean, `13ffc283` conflicts |
| 8 | Shader translation fixes: shift mask in uniformity proof; exact tiled cascade shadows on wave32; level query without array slice | `abbbd6b0` (ryotdev), `4fd34fdd` (sawetco), `41d3184e` (avatarmanz, after upstream `13091f6e`) | `shader/emit/lower.rs`, `ir.rs`, `mod.rs` | — | fixed | none | none | fixes shadows, possibly skin/hair (unproven) | **changes the translator fingerprint: cold cache, re-prewarm (2,575 hashes)** | conflicts (small) |
| 9 | Driver pipelines made ahead (`victory warm`, pipeline book), slow ones on a quarter of workers | `b7ab9c49`, `a30829c4` (avatarmanz) | `apps/victory/warm.rs`, `pipeline/*` | translations prewarmed only | PSOs too | shorter loading, fewer world stalls | compile memory: must respect our gate | none | our compile gate | conflicts |
| 10 | VRAM-scaled budgets | `fffafee5`, `9beff3cd`, `de4ae616`, `010b90a1` (fever308) | `shadow.rs`, `draw.rs`, `resource.rs` | fixed lowmem budgets | scaled to local VRAM | — | **risk up on UMA**: the "VRAM" budget is shared RAM; needs a UMA clamp | none | UMA detection (we log it) | conflicts |
| 11 | Textures shown only after their upload; first uploads on a copy queue; a slot keeps a stand-in instead of black | `3bab23c0`, `350c6864`, `d1c412c5` (avatarmanz) | `book.rs`, `pool.rs`, `descriptor/host.rs`, `draw.rs`, `popwatch.rs` | shown at once | after upload | small | small | fixes black pop-in; they report the 16-min device loss gone (likely, not proven) | their texture book | conflicts (3-6 files) |
| 12 | **Renderer port 1**: the title's constants heap used in place (`OpenExistingHeapFromAddress1`) + indirect ordering fix | `86dc1097`, avatarmanz `631400f5`…`7ab13ed4` | `port.rs` (new), `inplace.rs`, `shadow.rs`, `host.rs` | uploads/fills | in place | large CPU cut on theirs | **down on UMA (no copy of a 174 MB heap)**, expected | flicker until the ordering fix | their `inplace.rs`/`shadow.rs` | conflicts |
| 13 | Renderer ports 2, 5, 6, 7: descriptor stores report their slots; constant buffers read raw in place; guard lock shared; journal split in 64 | `9997d255`, `3920dc26`, `9d723e88`, `673be0ea` (avatarmanz) | `port_*.rs`, `watch.rs`, `shader/emit/*`, `loader/descriptor_sites.rs` | scans/fills/one lock | patched | reported −1 to −5 ms each at 45 ms; here the same work is a bigger share | none to down | port 2 patches title code for build 1.0.2.1 only; port 5 changes shaders (fingerprint) | #12, their `watch.rs` | conflicts (2-8 files) |
| 14 | Asynchronous submission, list calls on our threads, joined parts | `b8b30d1c`, `3a99ee61`, `37332f4d` (avatarmanz) | `queue.rs`, `queue/behind.rs`, `list/thin.rs`, `fence.rs` | title thread submits | 8 threads | reported 103 → 68 ms; Belobug's own line found ASYNC "no faster" | **up**: more in flight | ordering | their queue rewrite; ours on `fix/prestream-1118` (`dfac3b46`, `3a85ae51`) overlaps | conflicts |
| 15 | Textures written by shaders read back to title memory (boats) | `23925a4a` (from fxceless `f5c03f3b`, Geckomacabre) | `texback.rs`, `watch.rs`, … | none | on | small cost | readback copies (up) | needed for later gameplay | — | conflicts |
| 16 | Ambient light follows the sky | Belobug `main` `705a2d09`, `e48eb83e`, `4f62c2a5` (beloooo) | `ambient.rs` | constant | follows sky | none | none | improves our ambient port | our `ambient.rs` | conflicts (small) |
| — | FSR 4.1.2 to 4K | `cd8dfc06` (beloooo, from DexNom `ab34a8b2`) | `fidelityfx.rs` … | `VICTORY_FFX_BUILTIN` | AMD SDK | — | up | — | RDNA4 only; the 780M (RDNA3) falls back to FSR 3.1 | deferred |
| — | Fast cores / thread priorities | `96f2f32e`, `c278c677`, `0d065d35` | — | — | — | — | — | — | the 8840HS has eight equal Zen 4 cores; only priorities could matter | deferred |
| — | Series X profile, ray tracing on | — | — | — | — | — | **up**, beyond 16 GB | — | — | rejected here |

### Integration order

1. **#0 first**, before taking anything: our own alias guard is the
   largest measured slowdown. Both lines' changes are judged against a
   baseline without it, unless the A/B says otherwise.
2. The clean, memory-neutral group: #1-#3, #5, #6, #7 (`21323661`). Build,
   full tests, one run.
3. #4 by hand (romainhedouin's draw path), then #8 with a re-prewarm (cold
   cache otherwise, see Tests #35/#36).
4. #11 and #9, then #12/#13 in their own sub-branch (largest gain, largest
   dependency: `a-publier`'s `inplace.rs`/`shadow.rs`/`watch.rs`).
5. #10 and #14 only with a UMA-aware cap, measured against memory.

## Tests #30-#40 reconciled from the logs

Recomputed from each run's `process.jsonl` (5 s average per sample; a
"world sample" has 3,000 draws or more, which includes black world frames,
so #36 and the first frames of #40 count although black):

| Run | Build | End (from `termination.json`) | World from | World time | World fps median / peak | CPU/frame median | Refresh/frame (last 60) | Peak private |
|---|---|---|---|---|---|---|---|---|
| #32 | `dc567b92` | 600 s cap | 197 s | 402 s | 1.40 / 1.80 | 377 ms | — | 14.67 GiB |
| #33 | same | device removed, 283 s | 166 s | 117 s | 1.50 / 1.80 | 410 ms | — | 14.14 GiB |
| #34 | same | 600 s cap | **90 s** | 510 s | **1.90 / 2.10** | 271 ms | 75 ms | 12.79 GiB |
| #35 | same | emergency (≤64 MiB 2 s), 256 s | 155 s | 101 s | 1.40 / 1.90 | 292 ms | 68 ms | 13.22 GiB |
| #36 | `2ded7c88` | manual stop, 524 s | 176 s | 348 s (3D black) | 1.40 / 1.80 | 233 ms | 54 ms | 12.03 GiB |
| #38 | `924e2679` | exit code 0, 366 s | 279 s | 87 s | 1.50 / 1.80 | 381 ms | 101 ms | 13.31 GiB |
| #39 | `924e2679` | manual stop, 899 s | 272 s | 627 s | 1.10 / 1.40 | 792 ms | 863 ms | 13.56 GiB |
| #40 | `2e6645ec` | exit code 0, 470 s | 284 s | 187 s | 0.60 / 0.70 | 1,504 ms | 1,282 ms | 13.22 GiB |

#30, #31 and #37 never reached the world. The brief's figures hold: best
**measured** 2.10 fps peak and 1.90 fps median (#34), world at 90 s from
launch (Cycle V wrote 85 s; the two cuts differ by one 5 s sample).
Peaks of 2.4-2.6 fps do not appear in any sample (**not confirmed**).

Exact settings, from the launch commands:

| Run | Switches |
|---|---|
| #34 | lowmem, loop limit 32, `SHARED_TILES=1`, compile gate 1,024 MiB, capture, TextureNoWait, WindowBacking, GPU-allocation sampler, memory and thread samples |
| #38 | lowmem only (loop limit 32), observer, 900 s cap |
| #39 | #34's switches + `VICTORY_ALIAS_GUARD=1`, 8 workers, 1,200 s cap |
| #40 | #39 with loop limit default (2^20), cache prewarmed |

Not all of #38-#40 completed: #38 and #40 ended themselves (exit 0, no
record), #39 was stopped by the manual stop file at the 15-minute mark.

## Test #41: current baseline without the alias guard

`2e6645ec` (SHA-256 `7C5C5886…0784`, the #40 executable), #39's switches
without `VICTORY_ALIAS_GUARD`, loop limit default (cache prewarmed),
observer on, 900 s cap. Started with 9.4 GB of RAM available. Stopped by
hand (manual stop file) at 571 s on the user's request; no panic, no
device removal.

| | #40 (alias guard on) | #41 (off) |
|---|---|---|
| World from | 284 s | **168 s** |
| World time | 187 s | 403 s |
| fps median / mean / peak | 0.60 / 0.57 / 0.70 | **1.50 / 1.51 / 2.20** |
| Sample frame time p50 / p90 / p95 / p99 | 1,674 / 2,167 / 3,106 / 4,642 ms | 651 / 950 / 1,089 / 1,673 ms |
| CPU per frame (median) | 1,504 ms | 341 ms |
| GPU wait (median) | 84 ms | 161 ms |
| Texture refreshes, CPU per frame (last 60 lines) | 1,282 ms | **56 ms** |
| Peak private / lowest available | 13.22 GiB / 1 MiB | 14.02 GiB / 0 MiB (dips under 2 s) |
| VRAM | 5,427 of 6,533 MB | up to 7,586 MB against a 6,507 MB budget (over) |

- **The alias guard is the slowdown** (measured, one pair, same build
  and loop limit): texture refreshes 1,282 → 56 ms a frame, fps 0.60 →
  1.50. It stays off; the #42 run with it on is no longer needed for
  this question. Its purpose (fewer hashed bytes) has to be met another
  way.
- **Hair** (observed, frame 3594): with the default loop limit Lucia's
  hair is drawn as curls, not streaks: **the loop limit of 32 caused the
  streaks.** The hair has no colour (white/grey).
- **Skin**: still black with bright speckles: **not** the loop limit.
- The scene stays very dark (no indirect light).
- Peak 2.20 fps is the best single sample so far; the median (1.50) is
  below #34's 1.90 (loop limit 32 there; different scene time, one run each).

## Next tests (proposed)

The runner accepts run numbers up to 40; it needs #41+ added first. No
number is reused after a launch.

| Test | Objective | Settings |
|---|---|---|
| #41 | Current baseline, no alias guard | `2e6645ec`, #39's switches minus `ALIAS_GUARD`, loop limit default (prewarmed), 900 s |
| #42 | Alias guard A/B | #41 + `VICTORY_ALIAS_GUARD=1`, same scene and cache |
| #43 | `a-publier` baseline, as far as 16 GB allows | `4eb6cfdf` diag build, its `lowmem`-less defaults, Series S; observer on; a memory stop is the expected end and still a result |
| #44+ | First integrated group (clean, memory-neutral), then as in the brief | per group |

Desktop tests (RTX 4060, RTX 5070 Ti) need the user on those machines;
their exact CPU and RAM are not verified yet.
## Cycle VI continuation, 27 November

The clean starting state was `8fc7225af7d8d53353286bbe4d3525a011cf86ba` on `experiment/cycle-vi-cross-fork`. The fork's published branch was at `1cf16af6656f1ee843fc75cef07679d6689bdbc4`. The starting integration is preserved at `backup/cycle-vi-integrated-8fc7225a`. The protected `laptop/780m` ref is `f11964e7037089f26c5ca986feb1166b58bdae3e`.

The latest consumed test number in the runtime ledger is 41. The archived file hash verified against its provenance, source `2e6645ec4962ae9c5d73ca39927d0674ba181659`, SHA-256 `7C5C588638DF06951A5DFFE05A1A9EC919457B920C123594BF2ACEDD95660784`. This was an optimized diagnostic build, lowmem with a 256 MiB card cap, texture no-wait, shared tiles on and alias guard off, with a prewarmed shader cache and the default shader-loop limit. The user-reported later interactive breakthrough has no matching numbered log, hash or launch environment in the inspected artifacts. Test #41 recorded world submissions, median 1.50 FPS and peak 2.20 FPS, and must remain attributed to the earlier source. It reached repeated near-zero RAM events despite starting with about 9,339 MiB. Its archived observer used a two-second dwell at 64 MiB. The integrated candidate is preserved as a new independently built diagnostic executable with SHA-256 `A8E005B370E55AD107527EC494959652DAAE4B7186010CBDB71781B3D9425691`. Test #42 remains unused. With only about 5–6 GiB available during this session and no evidence of a large runtime memory reduction, a title run would neither reproduce the earlier conditions nor establish safe world-stage headroom.

### The seven original integrations

| Local / original | Audit and validation scope |
|---|---|
| `d3f03447` / `a23d1b46` | Belobug release thin LTO and one codegen unit apply to both release and inherited diag profiles. Changes code generation and compile memory. There is no runtime toggle or measured title gain. |
| `cbf939ec` / `40b8b9b5` | Belobug defers the prior Shadow on a same-address registration. The registry selects the new owner immediately while the queue retains the old allocation through its submission. Transient memory can increase safely. |
| `e504d944` / `92d353f8` | Belobug/romainhedouin retains the PSO and root once by native PSO address per recording. A retained COM reference prevents ordinary address reuse. A synthetic repeated-binding case with the same PSO and different native root objects would help establish the scope of that identity assumption, particularly with driver cache loads. |
| `1eb4c8e1` / `b022d860` | Belobug/romainhedouin avoids the pending lock after publication. The slow path publishes the OnceLock after replaying buffered descriptors. Concurrent writers still use the existing mirror path. |
| `656605af` / `05ee4387` | The a-publier refresh policy uses the existing worker and ordering machinery. Absent an override it enables on four or more available logical CPUs. Its opt-out is `VICTORY_WIDE_REFRESH=0`. Compare on UMA with identical draws because concurrent refresh can shift memory bandwidth demand. |
| `e715735a` / `cc11cccc` | The indirect scratch state is kept readable between commands and restored on reuse, growth or close. The WARP and host fixtures cover read, clamp and transition cases. Existing path controls can vary indirect workload, but they do not switch off this state fix independently. Validate transitions on both NVIDIA adapters. |
| `8fc7225a` / `21323661` | The avatarmanz wait patch runs on CPU instruction faults and is independent of graphics vendor. Replacement checks byte shape and uses an aligned atomic update with page restoration and an instruction-cache flush. Existing tests cover opcode shape and restored protection. Concurrent inspection of live code and platform failures when restoring protection or flushing deserve a targeted follow-up. Its opt-out is `VICTORY_PATCH_WAITS=0`. |

Exactly two of the seven integrated behaviors have independent runtime overrides. LTO is a build-profile decision. The other fixes are intrinsic to their paths. No additional switches were needed for the continuation changes.

### Warning and parallel-test audit

Normal `cargo check` at the untouched starting revision showed four warnings. The existing unused `Host::execute_own` and `Presenter::ahead` both trace to `578d459c3`. Neither has a caller or reader in either configuration, so the unreachable private implementation and empty field were removed. The existing `placed_filler_twins` from `6d19463de` and `tiles::line` from `8b5c8bfe` are read only by diagnostics and their diagnostic fixture, and are now feature-gated accordingly. The supposed newly introduced warnings were a misattribution. The wait patch's distinct `sites::line` lives inside a diagnostics module and is called. The indirect commit did not introduce the presenter field.

`221823fc` repairs two independent fixture ordering problems. Shared-tile teardown now runs with the real submission mutex held after settling, so unrelated runtime work cannot defer the last drop across the assertion. The tile copy-back fixtures explicitly take the queued snapshots under the same lock before simulating GPU writes. Previously another submission could take a snapshot after the test modified the bytes, explaining a zero-byte copy-back result. The output assertions remain intact. The runtime retirement race is separate. The old fast check could observe a pending submission, then enqueue after that submission completed and drained the retirement queue. The new recheck under the drain lock ensures the object is either dropped after observed completion or included in the drain. The early-drop path still releases outside the queue lock.

`64f21961` adapts only the root-constant portion of romainhedouin's `e6b638622b7cd1469feb976a5a3050aa6ceb869b` from Belobug's line. It keeps up to 64 dwords in stack storage, eliminating the allocation for each supported root-constant stage. Unexpected larger declarations keep the old heap fallback. No table ordering, descriptor identity, root binding or shader translation behavior changed.

### Remaining integration order and hardware

The existing candidate table above is the available conflict ledger. The full ordered list of all 57 trial SHA values was not found. Within the documented conflicts, prefer the allocation-local portions of the draw-path group before moving anything that changes resource ownership. `84ea9c12` needs fixtures for command-list mutation and copy/write invalidation. `b41f3f3f` requires equality checks and collision tests against this fork's descriptor record shapes. `5e08eee0` needs copy-footprint alignment and detached-list/fence lifetime validation before reusing scratch, plus a retained-capacity measurement. The rest of `e6b63862` changes descriptor-table state and remains separate.

The shader fixes `abbbd6b0`, `4fd34fdd` and `41d3184e` need translation fixtures and an explicitly re-prewarmed cache. The upload-order group `3bab23c0` / `350c6864` / `d1c412c5` depends on the other fork's texture book and completion flow. It should be reconstructed as a narrow producer-consumer fixture before porting. PSO prewarm `b7ab9c49` / `a30829c4` must respect the 16 GiB compile headroom gate. VRAM scaling `fffafee5` / `9beff3cd` / `de4ae616` / `010b90a1` and asynchronous submission `b8b30d1c` / `3a99ee61` / `37332f4d` can increase live memory and remain deferred. The in-place constants and renderer ports need exact range ownership, cache coherency and replay fixtures. Preserve existing authorship and GPL headers in any future ports.

On the 780M, keep a guarded lowmem reference with the last recorded scene and matched cache, and measure private commit, available RAM and DXGI budgets independently. On the RTX 4060 and RTX 5070 Ti, start with the same resolution, route, ray tracing off, default GPU compatibility settings and no UMA-only overrides. Log actual driver, local/nonlocal budgets and shader stage capabilities. Compare the two opt-outs one at a time and keep shader caches and number of in-flight submissions comparable. Report per-present world-frame percentiles and distinguish GPU timestamps from fence waiting. Establish visible output and input response separately before calling the result gameplay. No NVIDIA performance or rendering results were obtained.

### Final automated validation and reproduction

The serial workspace finished with 587 passed, zero failed and 27 ignored. GPU diagnostics passed 177/0/20, HLE diagnostics passed 124/0/4, and the loader diagnostic library passed 28/0/0. Separate HLE and loader integration binaries passed four tests total. The isolated retirement fixture also passed. The final GPU library ran 12 times at 16 threads, each with 165 passed, zero failed and 17 ignored. A thirteenth pass used wide refresh off. An intermediate audit run exited with `0xc0000005` and produced no attributable stack or named failure. That crash did not recur in the final stress runs, so its cause remains open. The two assertion failures were reproduced and addressed separately.

```powershell
cargo check --locked -p victory
cargo test --locked --workspace --all-targets -- --test-threads=1
cargo test --locked -p victory-gpu -p victory-hle -p victory-loader --features diagnostics -- --test-threads=1
1..12 | ForEach-Object { cargo test --locked -p victory-gpu --lib -- --test-threads=16; if ($LASTEXITCODE) { throw "GPU suite repetition $_ failed" } }
cargo test --locked --profile diag -p victory-gpu --lib inline_root_words_cost -- --ignored --nocapture --test-threads=1
cargo build --locked --profile diag -p victory --features diagnostics
cargo build --locked --release -p victory
```

On separate shells, set `VICTORY_WIDE_REFRESH=0` for the refresh opt-out and `VICTORY_PATCH_WAITS=0` for the wait-patch opt-out. For the next authorized title baseline, archive the exact candidate, use the ordinary 256 MiB immediate physical guard and retain the independent watchdog. Use the same scene and prewarmed cache only after sufficient headroom is established. Reserve the next number when a title process actually starts. The available logs stop at #41, so there is no comparable live benchmark on the seven-commit binary.

The optimized root-storage benchmark alternated six matched pairs of 200,000 preparations at 8, 16, 32 and 64 dwords. The old heap path took 5.004–5.079 ms per pair arm, and inline storage took 0.657–0.661 ms. The median was 5.034 ms versus 0.661 ms. The isolated case exercises allocation and initialization, without rendering or D3D12 calls. It establishes the removal of one allocation per eligible stage. World frame time, GPU time and peak committed memory remain unmeasured on the new candidate.

For a desktop smoke test, hash the archived executable with `Get-FileHash -Algorithm SHA256` and run its `config` subcommand before launch. In a fresh PowerShell process, leave shared-tile, in-place and AMD upscaler overrides unset, set `VICTORY_RAYS=0`, and pass `run <authorized title executable> --execute` through the independent guarded runner. Keep the scene, resolution and driver fixed for each A/B pair. Start by verifying visible output and control on both NVIDIA cards. A separate telemetry pass can enable diagnostics and per-present timing. These are reproducible starting configurations, without validation on either desktop.

In the existing laptop workspace, this exact Test #42 runner validation succeeded without launching a title. Before a future authorized launch, verify the launcher emits `VICTORY_SHADER_LOOP_LIMIT=0` and the pinned lowmem budgets if matching Test #41. Use shared tiles on and alias guard off only for that matched condition. The stricter ordinary guard can end the run earlier, so termination times cannot be compared directly.

```powershell
& ./outputs/cycle5/claude/claude-runner.ps1 -RunNumber 42 -CardMB 256 -ProvenanceFile outputs/cycle6/baseline-8fc7225a-provenance.json -Preset lowmem -Diagnostic -GpuAllocationSampler -WindowBacking -MemorySamples -TextureNoWait -CompileMinAvailMB 1024 -MaxSeconds 900 -ValidateOnly
```

### Archived executables

Both final binaries were built from `64f21961db8bf27268db8f7d8688b787dd6f7b8c`. The integrated baseline and both final archives passed CLI smoke tests. The logs, manifests and binaries are under the local workspace's `outputs/cycle6`.

| Profile | Executable | SHA-256 |
|---|---|---|
| Integrated baseline, diag | `VIctory-cycle6-8fc7225a-diag.exe` | `A8E005B370E55AD107527EC494959652DAAE4B7186010CBDB71781B3D9425691` |
| Final diag | `VIctory-cycle6-64f21961-diag.exe` | `9968338FA1DA4A032BE6BF5203E26D0135B248469C3711780E2DB373C5CEC227` |
| Final release | `VIctory-cycle6-64f21961-release.exe` | `922DE0403FD78A272663F494F4E30EB234BAD3E993D126A851F25E6F67C4A65E` |
