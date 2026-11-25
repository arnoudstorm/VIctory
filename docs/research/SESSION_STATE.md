# Session state (checkpoint)

2026-11-25, branch `laptop/780m`, worktree `C:\Users\Arnou\VIctory-laptop`.

- Done: FSR4 confirmed as the cause of the black 1080p output; opt-in fix
  `VICTORY_FFX_BUILTIN=1` (HLE FidelityFX loader plus a host bilinear
  upscaling pass); M1 met.
- Runs: `fsr-start` (baseline), `fsr-shaders` (shader dump),
  `ffx-builtin` (switch on). Their logs and CSVs are in the stability
  notes folder, and their frames are in this session's notes folder.
- Tests: 543 passed, 0 failed (536 before, plus 7 for the new module and pass).
