# Graphics: current state (Radeon 780M, branch laptop/780m)

Updated 2026-11-25.

| Stage | State | Evidence |
|---|---|---|
| Geometry, G-buffers, depth (1280x720) | works | t3d dumps: G-buffers, 1920x1080 and 1280x720 depth, lit R11G11B10 image |
| Lighting | partly | lit image in dumps and on screen; faces under backlight very dark (ffx-builtin frame 2527), unconfirmed whether that is a defect |
| Upscaling 1280x720 to 1920x1080 | **fixed, opt-in** (`VICTORY_FFX_BUILTIN=1`) | FSR4's 20 compute programs do not translate (ACCC/FSPX, no DXIL part); with the switch, our bilinear pass fills the output (`docs/updates/fsr4-black-output.md`) |
| Post-processing, tone mapping, composition | works once fed | 3D cutscene visible through normal presentation with the switch |
| Presentation | works | frames saved from the presented planes |
| Exposure | open | some shots near white (frame 2505); FSR's exposure input is null, so the title owns exposure |
| Memory at the world load | blocks long runs | free RAM falls to 5-7 MB; `fsr-start` ended by itself at 190 s |

Milestones (with `VICTORY_FFX_BUILTIN=1`):

| | Milestone | State |
|---|---|---|
| M1 | one correct visible 3D frame | **met** (ffx-builtin frames 2527, 2567) |
| M2/M3 | 30 / 100 consecutive frames | not measured yet: captures were every 2-8 frames, about 70 world frames per run |
| M4/M5 | animation, camera, scene cuts | cutscene shots change between captures (several cuts seen); not checked frame by frame |
| M6-M8 | 30 s / 1 min / 5 min | the world is reached at about 230 s of a 240 s run; longer runs are limited by memory |
