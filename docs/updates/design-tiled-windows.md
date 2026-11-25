# Tiled windows: windows that hold only the tiles copied (2026-11-25)

**Status: built, opt-in (`VICTORY_TILED_WINDOWS=1`), on in `run-laptop.ps1`.**

## Why

At the world load on the Radeon 780M the host copies of title memory
outside buffers ("windows") hold 1.8-3.1 GB, but only ~1.4 GB of their
pages are ever copied (`audit-2026-11-25.md`). A window is one committed
CPU-mapped buffer, and **every byte of it is paid when it is made**:
probe 1 below.

## Probes (`host::window_probe_tests`, run by hand on the 780M)

| Probe | Result |
|---|---|
| `probe_untouched_window_pages`: 64 windows of 4 MiB made, then a quarter of each written | Made: card +256 MB, private +256 MB, working set +256 MB. Written: +0. Pages never written cost in full |
| `probe_window_tiles_on_a_cpu_heap`: a 4 MiB reserved buffer, 16 of its 64 KiB tiles mapped onto a 1 MiB CPU-visible heap (custom, write-back, L0) that a mapped placed buffer also covers | Tiled resources tier 3. Charged 1 MiB (the heap), not 4 MiB. A copy on the card through the reserved buffer saw the CPU's write through the placed buffer. No device removal |
| Mapping cost | 16 single-tile `UpdateTileMappings` calls: 500 us (~31 us each) |

## Shape of the change

- A window becomes a reserved buffer of its full size; its tiles are
  mapped on demand onto 64 KiB tiles of pooled CPU-visible heaps (e.g.
  4 MiB heaps, each covered by one mapped placed buffer).
- A tile is mapped before the first copy into it (from the copy paths in
  `shadow.rs`, which already track copied pages per window), and the
  mappings of one submission are sent in one call, ahead of its lists on
  the same queue.
- The CPU side is no longer one pointer: `Shadow::data` becomes a lookup
  from window offset to the tile's heap address. Copies, snapshots,
  `before` and copy-back walk tile by tile.
- Tiles go back to their pool when the window is let go (after its last
  submission, as `retire` does now).
- Opt-in switch first, as with the other experiments.

## Risks to settle before it is on by default

- **A copy into a tile that is not mapped is dropped** (reads of unmapped
  tiles return 0 on tier 3): every write path must map first; a missed
  path is wrong data, not a crash. Needs a test per path.
- Shader writes into an unmapped tile are dropped too: windows the
  shaders write (copy-back) must map the whole written range.
- Asks without a length read to the cell's end: unmapped tiles there read
  0, as the host-zeroed bytes they hold now do.
- Mapping cost at the world load (thousands of tiles): batch per
  submission and measure.

## Expected

Up to the ~1.0-1.1 GB of window pages never copied, off both the card's
budget and RAM; not yet measured in the game.

## Built (`crate::tiles`, `Shadow::read_at` / `write_at` / `spans`)

- Tiles go on pooled 4 MiB heaps (custom, write-back, L0), each covered by
  one mapped placed buffer; a tile is zeroed when taken; a heap whose
  tiles are all free is let go (one spare kept).
- Every CPU access of a window goes through `read_at` (zeros where no
  tile is mapped) or `write_at` (maps the tile first, on the host queue,
  ahead of the next submission). Pieces of 64 KiB of title memory are
  tiles, since windows start on that grid. Declared shader writes map
  their whole range (`Shadow::writing`).
- Tiles of a window let go are given back through `retire`: only after
  every submission running then has finished.
- Probe 4 (`probe_window_tiles_on_opened_heaps`): tiles on heaps opened on
  our own memory remove the device on the 780M too (`0x887A0001`), so the
  CPU side is per tile, not one range.

Tests: `tiles::tests` (chunks; zeros until written; the card sees CPU
writes) and `shadow::tiled_tests` (refresh maps only the tiles written;
declared shader writes map and snapshot; copy-back reaches title memory;
card and CPU side agree; unmapped tiles read 0 on the card). Workspace:
527 passed, 0 failed.

## Measured in the game (150 s, no keys, lazy lists on)

| | Baseline (`p5-owners`) | Tiled, first build (`p5-tiled`) | Tiled, heaps let go (`p5-tiled2`) |
|---|---|---|---|
| Windows, full size | 2.3-2.7 GB | 2.4-2.7 GB | 2.3-2.7 GB |
| Memory they hold | the same | heaps 2.13 GB (never shrank) | **1.86-1.99 GB** of tiles, heaps within 4 MB |
| Mapped for shader writes | - | - | 0 MB |
| The card at the world load | 6.3-7.3 GB | 6.6-7.2 GB | **6.0-7.0 GB** |
| Errors, device removal | none | none | none |
| Frame 600 | right | right (same planes, mean 81 vs 84) | - |

- About **0.5-0.7 GB less** held by windows. Less than the ~1 GB never
  copied: copies are by 4 KiB page, tiles are 64 KiB (Direct3D's
  smallest), so a page copied takes its whole tile.
- The card's peak drops ~0.3 GB, within run-to-run noise, and stays over
  the 5.7-5.9 GB budget: tiled windows alone do not bring the world load
  under it.
- One warning kind was new in `p5-tiled2` (`buffer view the host would
  refuse left null`, a raw view at bytes 8-72 of a 256 KiB window); the
  check does not look at whether the buffer is tiled, and its unordered
  sibling shows in untiled runs. Not tied to tiling; logged once.
