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

## Next

1. The loading burst (E27's unattributed ~1.8 GB, here ~2 GB) decides
   whether a run reaches world preparation at all; it is the first gate.
2. Test #31: the same configuration from more available RAM (Test #29's
   start or higher), to measure shared tiles over world preparation.
3. Only after a measured world-preparation result: consider shared tiles
   for `run-laptop.ps1`; it stays off by default until then.
