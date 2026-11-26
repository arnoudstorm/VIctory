# Cycle V, continued (Claude, 26 November 2026)

## State taken over

- Branches: `laptop/780m` and `backup/laptop-780m-pre-cycle-v` at
  `f52a6102`; `experiment/780m-alias-memory` at `4561e8d5`. Local and
  GitHub matched; working tree clean. The backup exists as a branch (no
  tag).
- Codex's handover was complete as documented: the Test #29 executable
  (`540EA15A…E311A`) hash-matches, and Test #30 was not launched.
- Memory before work: 9,310 MiB available of 15,663 MiB; Codex processes
  gone; the Claude desktop app tree held ~2.4 GB working set (13
  processes).

## The 579 MiB in Test #29, attributed

Replaying Test #29's backing trace (`window_reserved` + `window_tile_bind`,
title page = window start / 64 KiB + tile) reproduces 578.75 MiB of tiles
backing a title page that another window also backed:

| Relation of the later window to the oldest holder of the page | MiB |
|---|---:|
| Later window **contains** the earlier one | 537 |
| Later window inside the earlier one (the mapped-geometry selector's case) | 20 |
| Partial overlap | 19 |

By the later window's size: 462 MiB in 4 MiB windows (asks without a length
run to their 4 MiB cell's end), 96 MiB in 1 MiB, 20 MiB in 256 KiB. Only 6
of 4,043 windows were ever dropped: the earlier windows stay alive, held by
descriptor slots.

That is why the selector saw zero eligible requests: it reuses an older
window that contains a new small ask, while the duplication comes from a new
long window over pages that short windows already hold.

## Change: shared tiles (`4e9052e7`, opt-in `VICTORY_SHARED_TILES=1`)

When a tiled window first maps a tile, it maps the pool tile another window
already holds for the same title page, with a reference count. Correctness
rests on what writes reach a tile:

- outside declared shader-write ranges only title bytes are written
  (`Shadow::copy`): the windows hold the same bytes, or newer;
- copy-back and relay merges write only inside declared ranges, and
  `Tiles::map_range` (called by `Shadow::writing` before the snapshot is
  taken) first gives the window a tile of its own with the same bytes
  (copy-on-write), mapped on the queue ahead of the submission; the shared
  tile's hold is retired after running submissions.

No descriptor, list or recorded view is rebound (unlike E17). A tile goes
back to the pool with its last holder. Known difference: a shader write
outside any declared range now lands in a tile that other windows over the
same page can see.

Validation: new GPU test
`tiles::tests::windows_over_the_same_title_pages_share_tiles_until_shaders_write`
(on the 780M: the card reads through both reserved buffers mapped onto one
heap tile, copy-on-write keeps the bytes and separates later writes, nothing
stays shared after both windows go, device alive). Serial workspace suite:
**578 passed, 0 failed, 26 ignored**.

Diagnostic build: `cargo build --locked --profile diag -p victory --features diagnostics`
at `4e9052e7`, SHA-256
`6E03C1285C7ABEE5B05070A37172B8644D92791F8D0937F95B6B5AD21E70DCCC`
(outside the repository, `outputs/cycle5/claude/`).

## Test #30

Same runner and switches as Test #29 (lowmem, 256 MiB card cap, texture
no-wait, relay 2, window-backing trace, 200 ms sampler, observer with the
64 MiB emergency stop), plus `VICTORY_SHARED_TILES=1`.

| | Test #29 | Test #30 |
|---|---:|---:|
| Available RAM at guard start | 9,205 MiB | 8,903 MiB |
| Runtime | 178.7 s | ~147 s |
| Stage reached | world preparation, 2,261 draws | loading, ~13 draws |
| Minimum available RAM | 4 MiB | 45 MiB |
| Window tile references at the end | 1,498 MiB | 421 MiB |
| Distinct physical tiles at the end | 1,498 MiB | 385 MiB |
| Tiles backing a page another window backs | 578 MiB | **0 MiB** |
| Device removals / texture failures | 0 / 0 | 0 / 0 |

Sharing did what it was built to do for every window the run made (36 MiB
saved at its end). The run was stopped by a private-memory burst during
loading (private bytes ~8.1 -> ~10.2 GB in about 15 s, with 13 draws a
frame and 2 MB of mapped tiles in the last memory line), before world
preparation, where Test #29's duplication arose. Test #29 lived through the
same burst with 4 MiB to spare, from 300 MiB more available RAM. **Test #30
does not measure the change's effect on world preparation.** No gameplay,
no FPS claim.

## Test #31

Same build (`6E03C128…DCCC`) and switches as Test #30, from more available
RAM: 9,695 MiB at guard start (Test #29: 9,205).

| | Test #29 | Test #30 | Test #31 |
|---|---:|---:|---:|
| Available at guard start | 9,205 MiB | 8,903 MiB | 9,695 MiB |
| Runtime | 178.7 s | ~147 s | ~104 s |
| Stage reached | world preparation | loading | loading |
| Minimum available | 4 MiB | 45 MiB | 75 MiB |
| Window tiles at the end | 1,498 MiB | 421 MiB | 3 MiB |
| Working set late in loading | ~3.1-3.4 GB | ~3.7-4.2 GB | ~6.6-7.0 GB |
| Private bytes at the stop | — | ~9.6 GB | ~8.1 GB |

Emergency stop again, in loading, with 3 MiB of window tiles: shared tiles
had nothing to act on. Starting RAM does not predict how far a run gets:
Test #31 started highest and ended earliest. Its working set held twice
Test #29's at the same stage (Windows trimmed Test #29 harder), so the
loading burst met less free memory. Private bytes alone (8.1 GB, below
Test #29's 10.7 GB peak) did not decide it.

Three runs of this configuration stopped before world preparation two times
out of three. A fourth identical launch would repeat that gamble, so no
further launch until the loading burst is reduced.

## The loading burst: the shader structurizer (`dc567b92`)

The 32 MiB+ Rust blocks logged during the loading burst (copystat
backtraces, resolved against the build's PDB) are all
`victory_shader::structure::Bools` tables: the `ite` memo had no bound and
grew to 34-68 MiB per shader, while up to 8 pipeline workers structure
shaders at once. In Test #31 about 440 MiB of such tables appeared within one
second of the burst.

Change (default on, not a switch): the memo is cleared at 2^18 entries (a
pure cache; nodes stay canonical through `unique`), and `memo` and `unique`
are let go once a `Structured` is built (nothing after building reads
them). Test builds use a 64-entry memo, so every translator test runs with
it constantly forgotten: 217 passed.

Offline A/B, `victory translate shaders.rpf --jobs 8` (26,084 programs, the
title's shader archive read locally, nothing kept):

| | Baseline | Capped (run 1 / run 2) |
|---|---:|---:|
| Results (every count) | — | identical |
| Peak private | 1,580 MiB | 970 / 914 MiB |
| Largest table | 113.6 MiB | 61.5 MiB |
| Table bytes allocated in total | 52.0 GB | 27.1 GB |
| Time | 181 s | 172 / 174 s |

The second baseline repeat was stopped for Test #32 and is missing. Side
effect: the translator fingerprint changed, so Test #32 started with a cold
shader cache (0 hits, 493+ misses, translation all through the world).

## Watchdog: ordinary guard observation-only, emergency on sustained floors

Tests #29-#31 were each stopped by the immediate `<=64 MiB` rule on a dip
of under a second; their 100 ms samples show every earlier dip recovering
(Test #29: 15 dips below 512 MiB, each gone within a second). The
Cycle V observer (outside the repository, `outputs/cycle5/claude/`) now
needs 2 s at or below 64 MiB, records every such dip, and keeps the other
emergency rules: under 128 MiB for 8 s, commit at or below 256 MiB, a 2.5 s
scheduling stall under 512 MiB, the manual stop file. Pagefile, memory
compression and GPU timeout recovery untouched. The 2 s rule was reviewed
and compiled, not exercised synthetically.

## Test #32 (`dc567b92`, SHA-256 `451FE11A…7D1A`)

Test #29's switches, plus `VICTORY_SHARED_TILES=1`, the capped structurizer,
the observer above and per-thread samples. Cold shader cache.

| Measurement | Result |
|---|---|
| Available at start | 8,697 MiB |
| Termination | **600 s time cap** (no emergency, no device removal, 0 texture failures) |
| Loading to first world frame | 187 s |
| World rendering | **187-597 s (410 s)**; Cycle V record was 129 s |
| Dips at or below 64 MiB | 10, all recovered, longest 1.8 s (lowest 0 MiB) |
| Peak private / working set | 15.0 GB / 6.9 GB |
| Heavy world frames (>=4,000 draws, n=51) | 1.40 fps; frame 696 / 843 / 2,014 ms (p50/p90/p99); CPU 377 ms, GPU wait 184 ms; 6,620 draws |
| Light world frames (300-4,000 draws, n=208) | 1.40 fps; frame 716 / 903 / 1,979 ms; CPU 318 ms, GPU wait 205 ms; 1,080 draws |
| Video memory late in the world | 7.3 GB used against a budget that fell to 6.6 GB |
| Process CPU in the world | 9.3 of 16 logical cores (about eight threads near 0.8 each) |
| Shared tiles | 3,160 MiB of window tile references on 1,351 MiB of physical tiles; 1,385 MB mapped onto another window's tile; 0 copy-on-write (no declared shader write reached a tiled window); 139 MiB still independently backed |

Seen on screen (user report): the world, a character, the health bar, the
tip message and sound; 3D elements visible, lighting only partly (the dark
shadowed areas already listed in `GRAPHICS_CURRENT_STATE.md`). Earlier in
the world the screen was black. Not checked frame by frame (capture was
off), so no rendering-correctness claim for shared tiles yet.

What it shows: memory pressure in loading and the world is spiky and
recoverable; with dips allowed up to 2 s, the run lived to the cap. The
577 MiB of overlap Test #29 had at world preparation is mostly avoided
(Test #32: 139 MiB independently backed at the end), but Tests #29 and #32
differ in more than shared tiles (structurizer, cold cache, observer), so
this is not an A/B of the 579 MiB claim.

## Test #33 (same build as #32, cache partly warm)

| Measurement | Result |
|---|---|
| Available at start | ~9.9 GB |
| Termination | **device removed** (exit 0x887A0005; reason 0x887A0007, "the GPU will not respond", no breadcrumbs) about 4.8 min in |
| Loading to first world frame | 163 s (Test #32: 187 s) |
| World frames (n=59, ~7,440 draws) | 1.50 fps; frame 663 / 1,059 / 8,160 ms (p50/p90/p99); CPU 401 ms, GPU wait 196 ms |
| Shader cache | 1,918 hits, 973 misses |
| Process CPU in the world | 8.7 of 16 logical cores |
| Dips at or below 64 MiB | 9, all recovered; the last two at 0-1 MiB for 0.6-0.9 s, seconds before the removal |
| Captured frames | none: the launcher only sets capture with its `-CutsceneTest` key script, so the runner's capture switch did nothing (fixed in the runner) |

What preceded the removal: the world moved to a new area (a few seconds at
3 draws), then **64 driver pipeline compiles** (0.1-5.1 s each, up to eight
at once; Test #32 had 6), most because the pipeline library refused its
stored copy (0x80070057, likely entries saved with the previous
translator's output), while available RAM touched 0. The formerly refused
mesh pipeline `f5e50eb2108c425b` was in that batch but ran in Test #32
without trouble, so it is not singled out. Most likely a GPU timeout under
memory exhaustion during the compile burst; not proven, and shared tiles
(on in both #32 and #33) are not excluded.

What it teaches: with the 2 s floor the runtime survives dips, but a burst
of concurrent driver compiles at an area change can still take the last
memory. The runtime already has a gate for that,
`VICTORY_COMPILE_MIN_AVAIL_MB` (driver compiles one at a time while
available RAM is under the floor; off by default, and the runner cleared
it). Next run: that gate at 1,024 MiB, with capture working.

## Test #34 (same build, warm cache, compile gate 1,024 MiB, capture on)

| | |
|---|---|
| Outcome | ran the full 600 s (deadline); no device removal |
| World | from 85 s; 390 overlay samples in the world |
| Frame, light scenes (300-4,000 draws, n=337) | 2.10 fps; p50 474 ms, p90 595 ms; CPU 273 ms, GPU wait 160 ms |
| Frame, heavy scenes (4,000+ draws, n=53) | 1.90 fps; p50 530 ms; CPU 272 ms, GPU wait 194 ms |
| RAM / VRAM in the world (p50) | 12.9 GB / 7,542 of 6,583 MB (over budget) |
| Dips at or below 64 MiB | 9, all recovered, the longest 0.7 s |

Faster than #32/#33 (1.40-1.50 fps); the compile gate and a warm cache
are both in it, so the gain is not attributed to either alone.

What the picture shows (frames 2113-2849, Lucia from the side at a
railing):

- **Hair:** strands spread into horizontal white streaks around the head,
  wider as the shot goes on (2113 narrow, 2657 and 2849 wide). Something
  that carries from frame to frame (the strand simulation's state) goes
  wrong, not one frame's drawing.
- **Skin:** face, arms and legs black with sparse bright speckles, while
  the shirt beside them is lit. Not only the missing ray-traced light:
  the speckles look like a skin pass whose recombination does not happen.
- The ambient pass (`VICTORY_AMBIENT=0.6`) ran (log line at 22:12:20).

Suspect: the lowmem preset's `VICTORY_SHADER_LOOP_LIMIT=32`. Translated
loops leave after 32 iterations per lane; E22 found the limit gave no
speed. A strand simulation or a skin kernel with longer loops would come
out wrong silently. Not proven.

## Test #35 (same, loop limit back to the default 2^20): stopped by the emergency watchdog

The launcher (`work/run-laptop-card256.ps1`, outside the repo) takes
`CLAUDE_SHADER_LOOP_LIMIT` for one run (0 is the default). Cap 900 s,
because the loop limit is part of the shader cache key (a cold cache).

The world's first frame came at 2.5 min (frame 3017), then the title
waited on shaders (2 draws a frame, black screen with sound and the tip
text, as the user saw). At 4.4 min available RAM stayed at 0 MiB for
2 s and the emergency watchdog stopped it (private 13.9 GB, rising
2 GB in 15 s while shaders were translated and the world streamed in).
About 1,100 shaders were translated and cached by then. **No answer on
the hair and skin.**

## Change: `VICTORY_PIPELINE_WORKERS` (opt-in)

The pipeline workers (half the logical processors, 2 to 8; 8 here)
each translate one shader at a time and hold its tables meanwhile. The
setting (1 to 8) lowers the peak of a cold cache at the cost of a longer
wait. It is in `victory-gpu`, outside the translator's fingerprint, so
the shader cache stays warm. Test
`pipeline_workers_follow_the_setting_within_one_to_eight`.

The same commit adds texture-refresh counters (diagnostics builds): why
each texture was hashed (no guard, aliased, the guard saw a write, the
eighth-frame turn, partly mapped), to cut the 110 ms a frame of texture
hashing in #33, which found 0 MB changed.

## Test #36 (`2ded7c88`, loop limit default, `VICTORY_PIPELINE_WORKERS=2`): stopped by hand

| | |
|---|---|
| Outcome | stopped by hand (the manual stop file) at 8.8 min, at the user's request |
| World | first frame at 2.1 min (frame 2587; #35: 2.5 min), then the HUD (health, stamina, focus) and the tip text (the user saw both), 3D black |
| Why black | recording waited about **500 ms a frame for pipelines**: some 600 draws a frame, their pipelines not built yet. Shaders translated: 1,136 by the stop; translation slowed to ~15 a minute late in the run |
| Memory | lowest 1 MiB; 7 dips at or below 64 MiB, all recovered (the longest 0.3 s); no emergency |
| Texture hashing (91 samples) | 91 MB a frame on average; in the world 113-134 MB, **of which 113-134 MB "aliased"** (counters from `2ded7c88`) |

With 2 workers the run held its memory but could not fill the cache in
time to draw the world. No answer yet on hair and skin.

## Change: `VICTORY_ALIAS_GUARD` (`be1ca828`, opt-in)

The counters show nearly all texture hashing is for textures whose bytes
another view shows too (`inplace::aliased`), hashed at every check since
the flicker fix because a write through that other view has no guard of
ours. With `VICTORY_ALIAS_GUARD=1` the other views' address ranges of the
same bytes (`inplace::alias_ranges`, byte for byte) are guarded as well
(`watch::writes(.., true)`), and the texture is hashed only when its own
pages or those ranges were written, or the set of other views changed.
The every-eighth-frame turn still hashes every texture, as a backstop.

Tests: other views are found byte for byte (unit); a write through a
second real view of a section is seen by its guard (real Windows
section, two `MapViewOfFile` views). Suite: 581 passed, 0 failed.
Not yet measured in the title, and the flicker audit must be repeated
with it before it can be a default.

## Runs #30-#37 at a glance

| Run | Build / change | End | Furthest stage | World fps |
|---|---|---|---|---|
| #30 | `4e9052e7`, shared tiles | emergency stop, ~147 s | loading | — |
| #31 | same | emergency stop, ~104 s | loading | — |
| #32 | `dc567b92`, structurizer cap, 2 s floor | **600 s cap** | world 410 s | 1.40 |
| #33 | same, cache partly warm | **device removed**, ~4.8 min | world from 163 s | 1.50 |
| #34 | same, compile gate 1,024 MiB, capture | **600 s cap** | world ~510 s; Lucia: hair streaks, black skin | **2.10** |
| #35 | loop limit default (cold cache) | emergency stop, ~4.4 min | world's first frame, then black (shaders) | — |
| #36 | `2ded7c88`, 2 pipeline workers | stopped by hand, 8.8 min | world: HUD and tip, 3D black (pipelines) | — |
| #37 | `be1ca828`, alias guard, 8 workers, 20 min cap | running | | |

## Next

1. Test #37: loop limit default with the cache now holding #35-#36's
   translations, 8 workers, `VICTORY_ALIAS_GUARD=1`, more world frames
   saved. Answers the hair and skin question if the world draws, and
   measures the alias guard.
2. If the picture is still wrong: a run with `VICTORY_DUMP_TARGETS=1` on
   one Lucia frame, to find the pass.
3. A shared-tiles off/on pair with capture (the 579 MiB claim, and
   correctness).
