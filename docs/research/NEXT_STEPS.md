# Next steps

Memory (2026-11-25): the duplicate windows are worth ~0.9 GB of GPU
memory (E17), but merging them on every ask removed the device twice.
Next try: list every place that keeps a window (`shadow::find` callers,
root-view caches in `draw.rs`, the relay) and merge once per frame, at
the sweep, not inside `loose`.

Status 2026-11-25 (E9-E13): in-place memory gives little; input lag is
the frame time; the ray-traced lighting (builder) crashes the title,
hangs the GPU, or runs out of memory first on this 16 GB laptop.

0. **Smoothness (CPU bound, about 400 ms per frame):** finish in-place
   runs (`VICTORY_INPLACE_RUNS`, `inplace.rs`). They place ~1 GB of
   windows in place, but asks are still served by copied windows beside
   them (`shadow.rs`, `loose`: `place_for` is asked only when a new window
   is made, so windows already copied keep being used). Route asks inside
   a placed run to the placed buffer and let the copied windows go, then
   measure moves, watches and snapshots (E8) and the card's memory.
   Expected gain: most of the 84 ms of moves and part of the watches.
   Large change; one step at a time.
1. **Lighting (earliest wrong stage now):** surfaces outside direct sun are
   near black. Dump the targets of one gameplay frame (VICTORY_DUMP_TARGETS)
   and follow the lighting pass's inputs: sky light, light probes, ambient
   occlusion, the R9G9B9E5 light textures (`resource::alias_of`). Find the
   first input that is black where it should not be.
2. Floating dark rectangles and blocky clouds: check the same frame's
   geometry and cloud textures.
3. Exposure: frame 2505 of `ffx-builtin` (near white).
4. Memory margin at the world (7 MB free): separate track.
5. Optional: PC FidelityFX path (upstream 0b35b7f) if the user provides
   AMD's SDK DLL, for temporal upscaling.
