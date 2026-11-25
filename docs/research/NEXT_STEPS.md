# Next steps

0. CPU-side write checks (audit #2): find why 370 MB of copies of 1 MiB
   or more stay on the CPU although the threshold is 1 MiB: log the card
   bytes alive against `VICTORY_CARD_BUFFERS_MB` (1,024 MB), then one run
   with a higher cap. Compare against the ~15 % run-to-run spread (E5).

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
