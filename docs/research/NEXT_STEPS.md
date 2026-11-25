# Next steps

1. M2/M3: run with `-FlickerFrames <world range>` and
   `VICTORY_FFX_BUILTIN=1` to save every world frame, then count
   consecutive non-black frames (frame_levels.ps1).
2. Exposure and dark faces: compare the 1280x720 colour going into the
   upscaler (dump) with the presented frame for shots 2505 and 2527, to
   tell whether they are intended or defects.
3. Longer runs (M6-M8): limited by memory at the world load; take that
   up separately (lazy lists and tiled windows are already on).
4. Optional: the PC FidelityFX path from upstream 0b35b7f, if the user
   provides AMD's SDK DLL.
