# Rendering experiments

Each experiment: one change, a hypothesis, the expected and the actual
result. The runs use the harness (`stability.ps1`, 240 s, `-Capture`,
`VICTORY_DUMP_TARGETS=1;VICTORY_CAPTURE_WORLD_ALSO=10,60,150;VICTORY_PRELUDE_POOL=1`).
Logs and frames are outside the repo (session notes folder).

## E1 (2026-11-25): baseline `fsr-start`

- Build `3f966a8`-era diag build of laptop/780m, no new switches.
- Result: world frames letterboxed and near black (frame 2048 mean 12.0).
  20 "no DXIL part" compute programs, all FSR4. The run ended by itself
  at 190 s with 5 MB of RAM free. 0.9-1.2 fps.

## E2: shader dump `fsr-shaders` (`VICTORY_SHADER_SAVE`)

- Hypothesis: the FSR4 programs are in a container format the
  translator does not know.
- Result: confirmed. 5762 shaders saved; the FSR4 ones have an `ACCC`
  part (FSPX v1) and no `DXIL` part.

## E3: `ffx-builtin` (`VICTORY_FFX_BUILTIN=1`)

- Change: the title's FidelityFX loader is served by our module, and the
  upscaling is a bilinear host pass.
- Hypothesis: the 1080p chain gets its input and the cutscene appears.
- Expected: non-black world frames.
- Actual: lit 3D cutscene on screen (frames 2497-2567, mean brightness
  33-231). The title's dispatch names colour 1280x720 R16G16B16A16_FLOAT
  and output 1920x1080 R9G9B9E5 (unordered access). 1.5-1.6 fps. No
  device removal. The run was stopped by the harness at 240 s.
