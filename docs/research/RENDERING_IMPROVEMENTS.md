# Rendering improvements on this branch (2026-11-25)

All measured on the Radeon 780M laptop, Series S profile, through normal
presentation (not previews). Mean luma and edge strength are 0-255 values
from captured frames; frames of one shot taken at different moments are
compared, so small differences are indicative only.

| Change | Switch (default) | Defect | Evidence |
|---|---|---|---|
| Our upscaler stands in for the title's FSR4 | `VICTORY_FFX_BUILTIN=1` (off) | the 1080p output was black: FSR4's 20 programs are console code (ACCC/FSPX) | world lit through presentation; 5 min 40 s without a black frame (`fsr4-black-output.md`) |
| Bicubic, sharpened upscaling | `VICTORY_FFX_FILTER=bicubic`, `VICTORY_FFX_SHARPEN` (0.25) (bilinear) | bilinear 1.5x is soft | hair edge strength 5.45 / 5.17 (bilinear, two runs) -> 5.76; the window area within the bilinear range; run `bicubic`, no failure |
| Integer render targets do not blend | always | mesh pipeline `f5e50eb2108c425b` refused in every run (first room's walls) | 0 refusals in every run since (`ambient-blend`, `upstream-perf`, `bicubic`) |
| Least ambient light | `VICTORY_AMBIENT=<level>` (off) | everything outside direct sun near black (rays do not run) | at 0.3: left wall 7.3 -> 8.6, whole frame 37.3 -> 39.9 mean luma |
| Staging buffers marked in flight only when copied from | always | 14 needless waits for the card per frame | waits 342 -> 201 ms per frame (E7) |

Not improved yet, and why:
- Ray-traced light: the builder runs (53 pipelines) but crashes the title,
  hangs the GPU or runs out of memory within 2-3 minutes (E10-E13).
- FSR4 itself: an fp8 ML model compiled for the console; XCL decided the
  same (not emulated).
- Quad reads through helper lanes (Belobug `03629f3f`): needs a GPU pixel
  test before it can be shown here.
