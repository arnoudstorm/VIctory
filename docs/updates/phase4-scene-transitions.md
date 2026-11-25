# Phase 4: the start screen to world transition on the Radeon 780M

Run `p5-poolc` (diagnostics build, Series S, no keys, 220 s; the title
starts its world by itself), read with `transitions.py` from the host's
frame-rate lines (every ~5 s). Three things are kept apart: what the
title **submitted** (draws a frame), what VIctory **dropped** before the
GPU (no host pipeline), and what was **presented** (frames a second).
GPU time is the time a frame's work took on the card.

## Timeline (seconds from the first line with 500+ draws a frame)

| t | fps presented | ms/frame | CPU preparing ms | on the GPU ms | draws submitted/frame | draws dropped (since last line) | free RAM MB |
|---|---|---|---|---|---|---|---|
| +0 | 0.3 | 3,056 | 1,852 | 107 | 1,280 | 481 | 476 |
| +7 | 0.2 | 4,048 | 3,344 | 614 | 5,272 | 1,707 | 632 |
| +14 | 1.1 | 935 | 383 | 469 | 4,751 | 92 | 100 |
| +20 | 1.0 | 1,041 | 361 | 610 | 2,017 | 1,199 | 1,272 |
| +31 | 0.8 | 1,332 | 616 | 624 | 2,188 | 193 | 570 |
| +41 | 2.0 | 506 | 231 | 228 | 2,057 | 690 | 542 |
| +51 | 1.0 | 1,045 | 374 | 607 | 5,638 | 408 | 857 |
| +68 | 0.6 | 1,681 | 820 | 763 | 5,000 | 1,965 | 709 |

- The first world frames take 3-4 s, mostly on the CPU (preparing:
  copies of title memory and textures); after ~15 s it settles at 0.6-2
  frames a second with CPU and GPU time about equal.
- Every frame is presented: no frame is lost between submission and the
  screen. What is lost is **draws inside frames**: a few hundred to ~2,000
  per line (lines cover ~5 frames at these rates), i.e. roughly 5-15 % of
  the world's draws are not drawn.

## First observable errors

All at the same moment, 16 s **before** the first world frame (while the
world's pipelines are made):

| Kind | Count | What it is |
|---|---|---|
| `shader does not translate ... no DXIL part` | 20 compute shaders | The containers hold no DXIL. 20 matches the title's FSR4 upscaler passes, whose console-native pipelines also fail to load (`DeserializeComputePipelineStateX FAILED ... falling back to NewBE`, logged by the title). Most likely the same shaders; not matched hash by hash |
| `host mesh pipeline refused (0x80070057)` | mesh `f5e50eb2108c425b` + pixel `ce3f0432a08f1d35` | The driver refuses the pipeline; its draws are dropped every frame |
| `method not implemented` | pipeline state slot 0x58, command list slot 0x1a8 | Two console methods VIctory does not have yet |
| `a shader takes the address of a buffer view that is not there: it reads 0` | - | A shader reads a view slot that holds nothing |
| `a GPU address above 40 bits` | - | A committed resource placed above the console's address range |
| `a black frame after a frame that was not` | 1 at +7 s | See below |

## Black frames vs resource exhaustion

4-7 black frames per run: one ~9 s before the world in this run (free RAM
6.4 GB), one at +7 s (632 MB). Earlier runs had one on the start screen
with 1.5 GB free. **No link to low memory.** Whether the title draws them
(a fade) is not known; the presenter logs the planes' levels around each.

## What this means

- The transition does not fail at one point: it is slow (CPU copies), it
  drops a share of draws (two causes above), and it ends when memory runs
  out (earlier phases).
- Worth fixing first for correctness: the refused mesh pipeline (one
  pipeline, dropped every frame) and the two missing methods.
- Not attempted here: fixing them. Each is its own change.
