# Why the 3D intro reached the screen black: the FSR4 upscaler (2026-11-25)

**Result:** confirmed. The title's FSR4 upscaler is the missing link
between the lit 1280x720 image and the 1920x1080 output. With the opt-in
switch `VICTORY_FFX_BUILTIN=1`, the intro cutscene's 3D image now reaches
the screen through normal presentation. Off by default.

## Evidence

**1. The 20 compute shaders that do not translate are all FSR4 passes.**
Run `fsr-start` (this build, no new switches): every "shader does not
translate ... no DXIL part" warning comes right after a title line
`FFX FSR4 ACO: DeserializeComputePipelineStateX FAILED hr=0x80004001 for
'FFX_FSR4_<pass>' -- falling back to NewBE`, for `FFX_FSR4_PASS_0` to
`_26`, `FFX_FSR4_SPD_AUTOEXPOSURE_PASS` and `FFX_FSR4_RCAS_PASS` (28
passes; shared programs make 20 hashes). There are no other "no DXIL
part" failures.

| Hashes (compute) |
|---|
| 6a6c7b7fde59134f f46117391dd81461 8cbd7f5b9ce7530c 85dea520c893a3d9 32aa8384586e0f12 |
| f5ab5dcf95536794 47d8c225abfcb246 8fb5e4aa9c542c3f 55c46e739fa0796a 17af85f170e66f30 |
| 80a0178d3a9d8d12 477b52e73ddeb6f2 8b437ab61d8008fe 3d147e8190574516 8e614708143ef917 |
| f7fc6c984427fef3 528278df1274cb33 c99e30956be0deaf fdce2a0a9fb1521a ba578b59c8c28a57 |

**2. Why they do not translate.** The NewBE containers (saved with
`VICTORY_SHADER_SAVE`, kept outside the repo) have no `DXIL` part. They
carry an `ACCC` part instead: an `FSPX` v1 module with four sections.
Section 6 is plain, uncompressed GFX10 machine code (ends in `s_endpgm`
and `s_code_end` padding), section 0 holds register settings
(`COMPUTE_PGM_RSRC1/2`-like), and section 7 holds resource-mapping
records. VIctory's translator reads only `DXIL` parts with an `XDAT`/
`XBCH`/`FSSM` program, so the converter stops with "no DXIL part".
`DeserializeComputePipelineStateX` is device slot 0x218, which VIctory
answers with E_NOTIMPL (`device.rs`, `unknown_pipeline`).

**3. They are dropped exactly where the 1080p chain needs its input.**
In `frame-1609-ops.txt` (earlier run `t3d`), commands 9275-9338 are the
20 `DROPPED compute` lines. Their group counts fit FSR: 20x12 (1280x720
in 64-pixel tiles), 120x68 and 60x34 (1920x1080 in 16- and 32-pixel
tiles). Then command 9357 (pixel `3c0e984537499224`) draws the black
1920x1080 target `0x2d1a2863ca0` from, among others, the 1920x1080
R9G9B9E5 texture `0x2d158d61770` (`Srv s0+40`). Command 9362 writes that
texture back from the black target. Nothing else in the frame writes
it, so without FSR's output the loop only ever carries black.

**4. The title's upscale call names exactly those textures.** With the
switch on, the title's `ffxDispatch` reports colour 1280x720 (FFX format
4, host R16G16B16A16_FLOAT) and output 1920x1080 (host R9G9B9E5, flags
RT|UAV, state unordered access), plus depth and motion vectors. There is
no exposure, reactive or transparency mask, and the jitter is 0.

## The change (opt-in)

- `crates/victory-hle/src/modules/fidelityfx.rs`: with
  `VICTORY_FFX_BUILTIN=1`, this module serves
  `amd_fidelityfx_loader_xbox.dll`'s five functions (`ffxCreateContext`,
  `ffxDestroyContext`, `ffxConfigure`, `ffxQuery`, `ffxDispatch`). The
  title's own loader and FSR4 DLL are then not loaded, and none of its
  28 pipelines are made. Queries return the FSR quality ratios and no
  jitter (one phase). Other effects are refused.
- `crates/victory-gpu/src/external.rs`: `upscale` records our compute
  pass into the title's host list. It filters the colour's render-size
  corner bilinearly into the output's upscale-size corner. Texture states
  go through the list's tracking, the output is marked host-written, and
  the list's own heaps and pipeline are set again afterwards
  (`State::host_state_lost`).
- The structure layouts and the `external` shape follow upstream VIctory
  `0b35b7f`, which answers the same calls with AMD's PC FidelityFX DLL
  (`VICTORY_FIDELITYFX_DIR`). That path is not ported here: it needs
  AMD's PC SDK, which is not on this machine.

What this is not: FSR4. There is no temporal reconstruction or
sharpening, so the image is a bilinear 1.5x upscale.

## Before and after (harness, 240 s, same settings plus the switch)

| Run | Switch | World frames on screen | Example frame | Mean brightness (0-255) |
|---|---|---|---|---|
| `fsr-start` | off | letterboxed, flat dark grey/black | 2048 | 12.0 |
| `ffx-builtin` | `VICTORY_FFX_BUILTIN=1` | lit 3D cutscene (prison yard, characters, hair, sunlight) | 2527 / 2567 | 36.4 / 92.2 |

World frames in `ffx-builtin` (2497-2567, saved every 2-8 frames): 203.8,
231.3, 221.1, 195.6, 195.7, 106.2, 36.8, 36.4, 33.0, 217.4, 195.9, 152.7,
128.3, 92.2. The one black frame (2495) is the cut into the world.
Frame rate was 1.5-1.6 fps (613-685 ms), against 0.9-1.2 fps
(845-1056 ms) without the switch, because the 20 dropped passes and the
FSR4 DLL's work are gone.

Still open, not caused by this change: faces under backlight look very
dark (frame 2527), and some shots are washed out to near white (frame
2505). It is not yet known whether these are intended fades or exposure
or lighting defects. Memory still runs out at the world load
(`fsr-start` ended by itself at 190 s with 5 MB of RAM free).

Nothing from the game (frames, shaders) is in the repository.
