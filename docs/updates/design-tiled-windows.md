# Design: windows that hold only the pages read (started 2026-11-25)

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
