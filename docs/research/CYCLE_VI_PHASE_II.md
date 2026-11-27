# Cycle VI, Phase II: pipelines, memory and Tests #43–#50 (Radeon 780M, 16 GB)

Branch `experiment/cycle-vi-cross-fork`. Every run uses the legally installed
title on this laptop; no game files, captures or driver caches are committed.
Tests #43 and #44 were Codex's; #45–#50 are this audit's. Test numbers follow
the launch ledger (`outputs/cycle3/cycle3-launch-ledger.jsonl`); none was reused.

## 1. Why 64f21961 loaded ~618 pipelines where 2e6645ec loaded ~2,300

Phase I's integration build (64f21961, Test #43) drew the world with most of its
pipelines missing, while the older 2e6645ec (Test #44) had loaded 1,748 of them
15 s after start. The causes checked, in order:

| Suspect | Evidence | Verdict |
|---|---|---|
| Pipeline library incompatible | same library file both runs; names hash the translated bytes, and translation is deterministic (new test `the_same_program_translates_to_the_same_bytes_every_time`) | no |
| Shader translation cache cold | #43: hits 438, misses 0 | no |
| Compile gate head-of-line blocking | #43 held its first compile at 2.5 min; 508 pipelines loaded before it | not the start-up cause (fixed anyway, §2) |
| Wait-instruction patch (8fc7225a) spinning | #45 = #43 with `VICTORY_PATCH_WAITS=0`: same crawl, same CPU | no |
| Wide refresh (656605af) | the launcher had forced it on for #44 too | no |
| Title threads starving the workers | #46, thread samples with names: the 8 `victory-pipelines-*` workers used 720 of ~950 CPU-seconds | no: the workers were busy |
| **AMD's per-executable driver cache** | `%LOCALAPPDATA%\AMD\DxcCache` has one file set per executable; 2e6645ec's was 576 MB and warm, 64f21961's started with #43. The library is 1.26 MB for ~2,300 pipelines (~550 B each): it holds references into the driver's cache, not code. #49's counters: a library load took **1,038 ms** (graphics) and **3,194 ms** (compute) against **214 ms** for a plain create | **yes** |

So every new build's first runs recompile every pipeline inside "loads" from
the library, on the workers' CPU. Each later run of the same executable is
warmer (#43 → #48 on 64f21961: 618 → 2,930 pipelines).

## 2. Code changes (this phase)

| Commit | Change |
|---|---|
| bd8e3bd4 | Test: the same program translates to the same bytes every time |
| feac4c74 | A compile held back by the compile gate no longer holds a worker: a worker that finds another thread at the gate puts its pipeline aside (`gated` queue) and loads others; pipelines put aside go one at a time when the gate is free. Library loads and creates are counted and timed per kind (graphics/compute), with misses, pipelines put aside and holds, on the library's "saved" line |
| 098c7c36 | avatarmanz `b7ab9c49` ported: the pipeline book (`VICTORY_PIPELINE_BOOK=1` writes `known-pipelines.bin` and `known-shaders.bin` beside the library) and `victory warm`, which makes the book's pipelines in idle-priority child processes of 200 each (a hung or crashed compile costs one pipeline and is listed in `warm-bad.txt`; progress resumes from `warm-done.txt`, keyed by translator and driver fingerprint). Adapted: `Host::driver`, the shader cache key kept as here, and a **memory floor** (`VICTORY_WARM_MIN_AVAIL_MB`, 2,048): no child starts while available RAM is below it |

Not ported: avatarmanz `a30829c4` (slow pipelines on a quarter of the workers).
With a warm driver cache the slow tail is small (§4) and it changes worker
scheduling again; left for a later A/B.

Test suite at 098c7c36: 591 passed, 0 failed (`cargo test --workspace --locked`).

Runner and launcher (outside the repository): `-NoWaitPatch`, thread samples
with thread names (`GetThreadDescription`), and `-Overrides 'VICTORY_X=..;..'`
applied by the launcher after the preset and recorded in the preflight and the
ledger.

## 3. Memory: where 13.3 GB went (#48, world) and what was cut (#49)

#48 at its end, 13,315 MB private:

| Holder | MB |
|---|---|
| title pages (the title's own memory) | ~4,460 |
| host copies of title memory (windows 2,376 over 1,390 of title memory: ~990 copied twice or more) | 2,965 |
| host textures (1,101 render targets, 738 filled by the title) | 1,841 |
| tiled windows: tiles mapped / heaps | 1,732 / 2,032 |
| our own small blocks (978k allocations) | 1,102 |
| shader containers + translations kept | 189 + 188 |
| card in use / budget | 6,513 / 6,535 |

Cut for #49 (configuration only): `VICTORY_SHARED_TILES=1`, texture budget
768 MB, kept snapshots 64 MB, on the `low` preset (loop limit 64; lowmem's 32 is
what caused the black sky blocks and hair streaks, so it was not used).

At the same point of the run (300–330 s, world):

| | #48 | #49 | Change |
|---|---|---|---|
| private | 13,304–13,315 MB | 11,836–12,051 MB | **−1.25 to −1.48 GB** |
| card in use | 6,513–6,534 MB | 5,412–5,438 MB | **−1.1 GB** |
| tile heaps | 2,032 MB | 1,160 MB | −872 MB (1,722 MB of tiles mapped onto another window's) |

This meets the brief's ≥1.3 GiB GPU-headroom target at that point. It did not
hold: #49 lived longer, and between 330 and 450 s host textures went from 1.74
to 3.2 GB (the title-filled copies to 1.9 GB; the 768 MB budget does not limit
them), the card to 7.4–7.8 GB against a 6.5 GB budget.

## 4. Tests #45–#50

| # | Build | Settings (all diag, 256 MB card buffers) | End | World | Pipelines | Notes |
|---|---|---|---|---|---|---|
| 43 | 64f21961 | lowmem, observer only | manual stop, 320 s | 0.60 fps (2 samples) | 618 cached / 16 built | Codex |
| 44 | 2e6645ec | lowmem, observer only | device removed, 234 s | 1.80 median, 2.20 peak | 2,310 / 265 | Codex; warm driver cache |
| 45 | 64f21961 | as #43, `VICTORY_PATCH_WAITS=0`, **ordinary guard on** | guard stop (memory), ~70 s | — | 411 | rules out the wait patch |
| 46 | 64f21961 | as #43, guard on, named thread samples | guard stop (memory), ~120 s | — | 537 | workers 720 CPU-s |
| 47 | 64f21961 | `low`, no ordinary guard | 600 s cap | 1.40 median, 1.90 peak | 987 / 2 | shader cache cold for loop limit 64 (469 misses); free RAM reached 0 |
| 48 | 64f21961 | `low`, no guard, preloaded: 3,246 shaders prewarmed offline, compile gate off, 8 workers, early textures 48 ms, catch-up | device removed, 338 s | 1.70 median, 2.00 peak | **2,930 / 8** | 988 hits, 0 misses; 50 MB free before the end |
| 49 | 098c7c36 | as #48 + pipeline book + memory cuts (§3) | device removed, 687 s | 1.60 median, 1.80 peak | 2,015 / 44 | new executable: cold driver cache; crash §5 |
| 50 | 098c7c36 | as #49, driver cache warmed by `victory warm`, compile gate back on (3,072 MB headroom, 1,024 MB available floor) | device removed, 259 s (a command list's Close failed with E_OUTOFMEMORY) | 1.70 median, **2.20 peak**; p90 877 ms (best tail) | **2,940 / 2**; loads 1.0 ms (graphics), 0.7 ms (compute) | world from **155 s**, the earliest; 2,260 shader hits, 0 misses; free RAM 1 MiB |

World figures count only samples with ≥3,000 draws (no menu or loading frames).
"No ordinary guard" kept the independent emergency watchdog (64 MiB for 2 s,
128 MiB for 8 s, 256 MiB commit headroom) and Windows' pagefile, compression and
TDR as they are; no system setting was changed.

### `victory warm` before #50

`victory warm --used --jobs 4` (loop limit 64, F16 nearest, as the launcher)
made the 2,764 pipelines earlier runs drew with, out of 83,544 in #49's book,
in 85 s: none failed, no process ended, median 3.5 ms, p99 1.7 s; it never
waited for RAM. The new executable's file set in `DxcCache` grew to 633 MB
and was last written during the warm, so the warm fills the same cache the
game reads. In #50 a library load took 1.0 ms (graphics) and 0.7 ms (compute),
against 1,038 and 3,194 ms in #49: **the pipeline problem is solved for a
warmed executable**. World speed was not: 1.7 fps median, as in every run
since #44, with 298 ms of CPU a frame.

## 5. Why #49 and #50 crashed

At 687 s the device was removed with the driver's "GPU will not respond …
some other application submitted invalid commands" (a reset). At that moment
nothing of ours ran on the card; the submitting thread was preparing a
submission; eight workers were inside the driver (one compile 4.3 s in, library
loads 0.5–1.7 s in); the card held 7,778 MB against a 6,548 MB budget, and the
PC had ~500 MB available with 15.5 GB private. On a UMA laptop the card's
memory is the PC's: 1.2 GB over budget with no RAM left for the video memory
manager to page through, plus driver compiles allowed with no gate, ended in a
reset. #44 and #48 ended the same way (`DXGI_ERROR_DEVICE_REMOVED`, RAM
exhausted).

#50 ended earlier, at 259 s: a host command list's `Close` failed with
`E_OUTOFMEMORY`, then the device was removed; available RAM was 1 MiB and the
process 12.8 GiB private. With every pipeline ready the world came 80 s
sooner, and so did its memory peak. Two minutes in, still in the menus, the
process held 10.8 GB private with 2.8 GB of it resident, Windows' memory
compression 3.4 GB, and the PC under 500 MB free.

### Sound

The audio glitches heard in #50 come from the same pressure: the title's
memory was being compressed and paged back in, so its audio thread stalled on
missing pages and its buffers ran dry; at 1–2 fps the emulated title also
falls behind its own audio schedule. No audio warning or error was logged.

## 6. Shaders

No shader failed to translate in #47–#49 (0 failures; #49: 408 hits, 36
misses for shaders never seen before). One dispatch was dropped for want of a
pipeline in #49. What looked like "shaders not loading" was pipelines waiting
for driver compiles (§1).

## 7. Conclusions

1. Pipelines: the ~618-pipeline stall was AMD's per-executable driver cache,
   not an incompatible library. Shader prewarm, the pipeline book and
   `victory warm` fix it; every new build needs one warm (85 s here).
2. Memory is now what ends every run on this 16 GB UMA laptop. The config cuts
   save 1.25–1.48 GB private and 1.1 GB of card memory in the same scene, but
   texture copies the title fills are unbounded and later scenes outgrow it.
3. World speed (1.4–1.8 fps) did not change with pipelines ready: it is the
   CPU (~300 ms a frame) and paging, not missing pipelines.

## 8. What remains

- Texture copies the title fills are not under any budget; that is the next
  memory to bound (they tripled in #49's later scene).
- Warm every new build once before play (`victory warm --used`, with the
  launcher's translation settings), after a run with `VICTORY_PIPELINE_BOOK=1`.
- Desktop configurations, the rendering report and avatarmanz `a30829c4` are
  not done.
- No interactive-gameplay claim: the runs used scripted Enter presses only.
