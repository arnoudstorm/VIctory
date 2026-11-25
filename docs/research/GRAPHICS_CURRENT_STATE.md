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

Milestones (with `VICTORY_FFX_BUILTIN=1`, run `ffx-every`, 420 s, no keys
other than the run script's default Enter presses):

| | Milestone | State | Evidence |
|---|---|---|---|
| M1 | one correct visible 3D frame | met | frames 1867, 2059 |
| M2/M3 | 30 / 100 consecutive frames | met for "not black, not frozen" | presented-frame black detector (checks every present): one black frame at the cut into the world (1723), none after it up to 2155+; 63 saved frames 1724-2155 all mean brightness 17.9-229.6, each one differs from the one before (mean luma change 1.3-206) |
| M4 | animated geometry | met | held shot 1819-2003 changes 1.3-4.0 per sample (people, hair) |
| M5 | camera movement, scene transitions | met | cutscene cuts (changes of 60-206), then **gameplay** with the "Move around / Move the camera" prompt (2059) and a moved camera (2155) |
| M6-M8 | 30 s, 1 min, 5 min | met for continuity | world from 12:02:43 (frame 1724) to 12:08:23, when the harness stopped it: 5 min 40 s, no device removal, 0 "Not responding" samples |

Not yet correct, so the goal is **not** met:

- Lighting: surfaces outside direct sun are near black (silhouettes in
  2059 and 2155). Ambient and indirect light look missing.
- Dark rectangles float above the yard (2059, 2155). They could be
  awnings or misplaced geometry; not checked.
- Clouds show blocky low-resolution patches (1867, left).
- The image is a bilinear upscale, not FSR4.
- Memory: free RAM falls to 7 MB (private 12.5 GB), the margin is gone.
