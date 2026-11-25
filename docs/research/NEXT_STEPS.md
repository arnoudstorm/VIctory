# Next steps

0. CPU time is now the limit (409 ms per frame after E7). Split it per
   step of a submission (texture refreshes, write-watch asks, snapshots,
   compares) and attack the largest. CPU compares: one run with
   `VICTORY_CARD_BUFFERS_MB=1536`, which would put the 401 MB the cap
   kept off onto the card, watching GPU memory (already over budget) and
   paging.
   Shader cost: the GPU is only ~55 % busy, so shaders are not the limit
   yet; per-pipeline GPU timestamps would be needed first.
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
