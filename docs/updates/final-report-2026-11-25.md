# Final report: Windows stability and rendering session, Radeon 780M (2026-11-25)

Fork `arnoudstorm/VIctory`, branch `laptop/780m`, pull request #1 into the
fork's `main` (it conflicts with `main`: see "Hand-over"). Laptop: Radeon
780M, 16 GB shared RAM (~5.7-5.9 GB D3D12 budget), screen 2880x1800 at
120 Hz, Windows 11. Every game run was automated; no one watched the
screen. Numbers are measured unless marked as calculated.

## 1. What changed (commits on `laptop/780m`)

| Commit | Change | Default |
|---|---|---|
| `7d0ed69` | Start-screen strobe fixed: textures on pages another view also shows are hashed on every check | on |
| `00b6f89` | Lazy host lists (`VICTORY_LAZY_LISTS=1`); pooled preludes (`VICTORY_PRELUDE_POOL=1`) | opt-in; lazy lists on in `run-laptop.ps1` |
| `8b5c8bf` | Tiled windows (`VICTORY_TILED_WINDOWS=1`): windows hold only the tiles copied | opt-in; on in `run-laptop.ps1` |
| `08e9f1b`, `f5f388e` | Probes for tiled windows; owners of big heap blocks (diagnostics) | tests / diagnostics |
| docs | flicker, stability, resolution/HDR, Phase 4, Flamingo, Phase 6, this report, the audit | - |

## 2. Start-screen flicker (`flicker-audit.md`)

Cause confirmed by experiment (guards off: ring repeats 52 -> 1) and fixed:
jumps 67 -> 22, frames showing the old video slot 52 -> 2, most opposing
changes in 30 frames 24 -> 6. Remaining: fades, hard cuts, black frames
at the start of loading. **Not a photosensitivity certification**: only
brightness changes were counted.

## 3. Startup and menu stability (`stability-780m.md`)

3 of 3 startups reach the start screen. The 5- and 10-minute menu
milestones are **not met**: with no input the title loads its world after
55-80 s. Window switching works; "Not responding" for 11-26 s each run
at the world load (paging: free RAM below ~1 GB).

## 4. Resolution, HDR and presentation (`resolution-hdr-780m.md`)

The title ignores the reported display size; HDR, 120 Hz and frame
limits have no path in VIctory. Presentation is ~150 MB, ~2 % of the
card's memory (calculated). The world load is memory bound.

## 5. Scene transitions (`phase4-scene-transitions.md`)

Every frame is presented; what is lost is draws inside frames (roughly
5-15 % in the world), from 20 compute shaders with no DXIL part (most
likely the FSR4 passes the title also fails to load natively) and one
mesh pipeline the driver refuses. First errors 16 s before the first
world frame. Black frames are not tied to low memory.

## 6. World-load memory

| Item | Result |
|---|---|
| Where it goes (peak) | copies of title memory 2.4-3.7 GB (windows 1.8-3.1 GB), render targets 1.0-1.2 GB, textures 0.3-1.2 GB, command lists ~0.9 GB of process memory |
| Lazy host lists | start screen: card -0.5 GB, private -0.7 GB, free RAM +0.8 GB; world peak unchanged |
| Tiled windows | windows hold 1.86-1.99 GB instead of 2.3-2.7 GB; card peak ~0.3 GB lower (6.0-7.0 GB), within noise |
| Page pool as a section | `VICTORY_INPLACE_RUNS` / `VICTORY_POOL_COMMIT`: no gain; the driver refuses 3,180 runs; most windows lie partly outside any view |
| Lifetimes, duplicates | no leak; windows let go as lists stop naming them; the big heap blocks are the title's own (audio 251 MB) and transient |
| RAM vs card vs commit vs paging | commit is not the limit (41-43 of 45 GB); free RAM falls to 20-500 MB; Windows reads 0.4-0.8 GB/s from the page file; the card is 0.5-1.8 GB over budget |

**The world still goes over the budget** in every run (6.0-7.7 GB of
5.7-5.9 GB).

## 7. Tests and benchmarks (`phase6-tests-telemetry-fps.md`)

`cargo test --workspace`: **527 passed, 0 failed** (an earlier "529" in
this session was a miscount; the morning's run had 523). Synthetic
benchmarks are listed in the Phase 6 document.

## 8. Telemetry and frame rates

With lazy lists and tiled windows (`p6-telemetry`): **260 s in the world**
(earlier baseline: 33 s), no out-of-memory, no device removal. World frame
times: mean 1,710 ms, median 1,397 ms, P95 4,080 ms, P99 4,620 ms. 81 % of
the run below 2 fps, 17 % at 25 fps or more (start screen).

## 9. How runs end now

The longest run ended in the **title's own crash handler**: a breakpoint
exception (`0x80000003`) at title `+0x5149aec` on many threads at once
after ~4 minutes in the world. The crash report upload was blocked by
VIctory's network policy. Earlier runs ended out of memory or with device
removal at the world load.

## 10. Project Flamingo (`compare-flamingo-2026-11-25.md`)

676 commits past the common ancestor. Candidates for this laptop, none of
which applies cleanly: `a79befd` (views of title memory in runs: could let
windows be used in place), `d549a33`, `ec9247f` (a stand-in window against
"Not responding"), `d3ebb55` (DXIL and mesh-output fixes: possibly the
refused mesh pipeline). Not ported: each needs your go-ahead to build.

## 11. Not done, limitations

- 5/10-minute menu stability, card under budget, 1-2 GB of free RAM held:
  not reached.
- The title's breakpoint at `+0x5149aec`: not investigated.
- Swap chain at window size (~25 MB), per-pass GPU timestamps: not done.
- The PR's conflicts with the fork's `main` (28 upstream commits, 20+
  files): not resolved.

## Hand-over (where to pick up)

1. Merge the fork's `main` into `laptop/780m`, resolve the conflicts, rerun
   `cargo test --workspace`, so PR #1 can merge.
2. Port Flamingo `a79befd` (+ `d549a33`) and measure the in-place census
   and the card's peak; then `ec9247f` for "Not responding".
3. The breakpoint at title `+0x5149aec`: run `run-laptop.ps1` (it turns on
   lazy lists and tiled windows) with `-Dred`, and read what the title did
   last before it.
4. The refused mesh pipeline (`f5e50eb2108c425b` + `ce3f0432a08f1d35`) and
   the two missing methods (pipeline state slot 0x58, command list slot
   0x1a8).

Tools (in the session notes, outside the repo): `run-laptop.ps1`
(`-Keys`, `-LogName`, `-Set`), `stability.ps1`, `stability_check.py`,
`mem_timeline.py`, `res_compare.py`, `transitions.py`, `fps_audit.py`,
`frame_levels.ps1`, `flash_stats.py`, `flash_realtime.py`.
