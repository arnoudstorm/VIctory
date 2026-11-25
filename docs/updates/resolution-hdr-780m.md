# Resolution, HDR and presentation overhead on the Radeon 780M

Laptop: Radeon 780M, 16 GB shared RAM, screen 2880x1800 at 120 Hz.
Branch `laptop/780m` at `10deb1f`, diagnostics build, Series S profile,
settings of `run-laptop.ps1`, no keys pressed (the title starts its world
load by itself 55-80 s in, see `stability-780m.md`). Runs were automated;
frame 600 was saved from each run to check the picture.

## Three resolutions, kept apart

| | Who picks it | What it is here |
|---|---|---|
| Internal (3D) | The title, by console type | Not reported directly. Depth targets of 1280x720 and 1920x1080 were seen, and the title loads AMD's FidelityFX upscaler (FSR4), so it likely renders lower and upscales; not confirmed |
| Presentation (planes) | The title | Plane 0 (game) 1920x1080, plane 1 (UI) 3840x2160, each double buffered |
| Display (swap chain) | VIctory: the largest plane | 3840x2160, 2 buffers, RGBA8; Windows scales it to the 2880x1800 window |

## What can be changed, and what was measured

**Display mode reported to the title** (`VICTORY_DISPLAY_MODE`, default
3840x2160) is the only resolution setting VIctory has. Measured, 130 s
runs each:

| Reported mode | Planes at frame 600 | Swap chain | Start screen fps / CPU / GPU wait | Start-screen render targets | World load: peak VRAM | Peak render targets | Peak private | End |
|---|---|---|---|---|---|---|---|---|
| 3840x2160 | 1920x1080 + 3840x2160 | 3840x2160 | 30.0 / 4.6 / 2.8 ms | 246 MB | 7,596 MB | 1,121 MB | 12,764 MB | device removed at 106 s, free RAM 87 MB |
| 1920x1080 | 1920x1080 + 3840x2160 | 3840x2160 | 29.0 / 4.9 / 3.1 ms | 223 MB | 7,683 MB | 1,190 MB | 13,220 MB | device removed at 119 s, free RAM 39 MB |
| 1280x720 | 1920x1080 + 3840x2160 | 3840x2160 | 29.0 / 4.6 / 2.2 ms | 246 MB | 7,729 MB | 1,167 MB | 12,908 MB | device removed at 110 s, free RAM 20 MB |

**The title ignores the reported mode**: it drew the same planes at all
three, so nothing changed beyond run-to-run noise. Lower resolutions can
not be reached through it.

**HDR vs SDR: not measurable.** VIctory answers the title's HDR request
with "disabled (SDR presentation)" (`display.rs`); there is no HDR path to
compare.

**120 Hz vs 60 Hz: not measurable.** The mode list has one mode, at 60 Hz
(`dxgi.rs`); the title paces the start screen at 30 fps (33.3 ms) by its
own frame interval.

**20 / 30 / 60 fps limits: no limiter exists**, and none would help now:
the title already holds 30 fps where it can, and the world load runs at
0.7-0.9 fps.

## How big presentation is (calculated from the sizes above)

- Swap chain: 3840 x 2160 x 4 B x 2 = **66 MB**. At the screen's 2880x1800
  it would be 41 MB, so **~25 MB** could be saved by sizing it to the
  window.
- UI plane: 3840 x 2160 x 4 B x 2 = 66 MB; game plane 1920 x 1080 x 4 B x 2
  = 17 MB. Both are the title's.
- Together ~150 MB, about **2% of the 7.6 GB** the card held at the end of
  the world load. Copying 4K planes at 30 fps moves ~2 GB/s, against tens
  of GB/s of shared-memory bandwidth on this laptop; per-pass GPU time was
  not measured (no GPU timestamps per pass yet).

## Bound by what

- **Start screen:** neither CPU (4.6 ms) nor GPU (2-3 ms wait) is busy;
  the title paces it at 30 fps.
- **World load:** frames of 1-2 s, with 260-790 ms of CPU and 570-1,300 ms
  waiting for the GPU, while free RAM is 20-90 MB and Windows is paging.
  It is **memory bound**. Whether it would then be pixel or geometry bound
  can not be told while it pages.
- So **540p at 20 fps is not realistic now**: VIctory can not lower the
  title's internal resolution, and the cost is not in pixels. Even if
  pixels cost nothing, the 7.6 GB the card held stays (host copies of title
  memory ~3 GB, title render targets ~1.1-1.2 GB, textures).

## What would help, in order

1. Less memory in the world load (host copies of title memory, render
   targets): the same open problem as `update-3-memory-breakdown.md`.
2. A swap chain the size of the window: ~25 MB, small but free of risk.
   Not done in this session.
3. Per-pass GPU timestamps, to measure presentation and compositing time
   directly instead of calculating it.
