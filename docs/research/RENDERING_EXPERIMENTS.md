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

## E4: `ffx-every` (`VICTORY_FFX_BUILTIN=1`, 420 s)

- Change: none to the code (`1272861`); a longer run.
- Hypothesis: with the upscaler filled, the world keeps rendering.
- Actual: the world ran from frame 1724 (12:02:43) to past 2155
  (12:08:23), when the harness stopped it. The game reached **gameplay**
  (control prompt, camera moved). Black detector: one black frame at the
  cut (1723), none after it. 1.4-1.6 fps in the world (645-717 ms).
  Private memory peaked at 12.5 GB, free RAM at least 7 MB, no device
  removal.
- Note: `-FlickerFrames 2480-2800` saved nothing, because the run never
  reached frame 2480. The capture folder also keeps older runs' frames
  above the new run's last frame; filter by the log's "frame saved"
  lines.

## E6 (2026-11-25): what the 16 syncs per frame and the 400 MB of CPU compares are (run `why-waits`)

- Change: counters only (submissions held for a texture staging buffer
  in flight; why `card_for` keeps a title buffer's copy off the card).
- Result:
  - **Syncs:** 13.9 of the 16 "title memory read" settles per frame were
    holds for texture staging buffers still in flight (about 100
    buffers, 76-93 MB). Every staging buffer a submission named was
    marked in flight, also the unchanged ones it did not copy from.
  - **CPU compares:** 72 copies (401 MB) were kept off the card by the
    cap (`VICTORY_CARD_BUFFERS_MB`, 1,024 MB), and 2 (5 MB) by the
    card's budget.

## E7 (2026-11-25): mark only staging buffers copied from (run `staging-fix`)

- Change: `Staging::take_recorded`. A staging buffer is marked in flight
  only by the submission that recorded a GPU copy from it.
  `VICTORY_STAGING_MARK_ALL=1` restores the old behaviour. Test
  `only_a_staging_buffer_copied_from_is_marked_in_flight`.
- Hypothesis: the 14 holds go away, and with them most of the waiting.
- Actual (`why-waits` -> `staging-fix`, same settings, world samples
  198 / 213):

| | before | after |
|---|---|---|
| holds for staging in flight per frame | 13.9 | 0.0 |
| settles per frame | 16.0 | 5.2 |
| waiting for the card in settles, ms per frame | 342 | 201 |
| frame (overlay), ms | 782 (1.46 fps) | 732 (1.59 fps) |
| CPU, ms | 372 | 409 |
| GPU wait, ms | 339 | 222 |

  The frame time gain (6 %) is inside the run-to-run spread (about
  15 %, E5). The waits clearly fell, but CPU time rose: the CPU is now
  the limit. Picture: unchanged, 63 saved world frames 2476-2899 all lit
  (mean 35.9-228.6), black detector only at the cut (2475), no device
  removal. Free RAM fell to 5 MB in both runs.
- Tests: 544 passed, 0 failed.
