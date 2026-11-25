# Playing on AMD RDNA3 (RX 7000 series)

This branch of the fork collects the open fixes that matter on AMD RDNA3
cards and a launcher with settings that were tested on one. It is not
playable yet in the normal sense: expect about **2 frames per second** in
the world, and dark scenes. Read **What to expect** before you start.

**This repository contains no game files.** You need your own unpacked copy
of the title.

## What was tested

| | |
|---|---|
| GPU | AMD Radeon RX 7600 XT (16 GB, RDNA3), driver 32.0.21025.10016 |
| CPU / RAM | 32 GB RAM, Windows 11 Pro |
| Build | Rust stable, `x86_64-pc-windows-gnu` toolchain, release |
| Profile | Series X (the default) |

| Run | Result |
|---|---|
| Menus | 30 fps |
| Opening cutscene and first world scenes, default settings | 1.0–1.4 fps at ~6,800 draws per frame |
| Same scenes with `run-rdna3.ps1` (relay on) | **1.9–2.5 fps**, one 3.5-minute run without a device loss |
| Relay on, *before* the relay fixes below were merged | device removed after ~50 s (the AMD driver reset) |

One machine and a few runs: treat the numbers as a first data point, not a
promise.

## What is merged on top of the fork

| PR | What it fixes |
|---|---|
| [#134](https://github.com/fxceless/victory-fork/pull/134) | View-heap indices clamped on every GPU: AMD removed the device on the first world frame. Adds `v_cvt_f32_f64` and `v_permlane16/x16` |
| [#135](https://github.com/fxceless/victory-fork/pull/135) | 64-bit integer compares done on their dwords: AMD's driver ordered some wrong |
| [#203](https://github.com/fxceless/victory-fork/pull/203) | 46 more shaders translate (16-bit wave sums with lanes gone, which needs a 64-lane GPU such as AMD; unused descriptor sizes; paired f16 interpolation halves) |
| [#141](https://github.com/fxceless/victory-fork/pull/141) | With the relay: a texture's upload buffer is no longer overwritten while the card still reads it |
| [#191](https://github.com/fxceless/victory-fork/pull/191) | With the relay: reads wait for in-flight writes from other copies, read-backs and late writes |
| [#206](https://github.com/fxceless/victory-fork/pull/206) | With the relay: pages never copied are no longer skipped under pending writes |

All credit goes to the authors of these PRs and of the fork; this branch
only puts them together.

## Requirements

- Windows 11 x64 (Windows 10 is untested here), a CPU with AVX2.
- An AMD RDNA3 card (RX 7600 / 7700 / 7800 / 7900 series) with a current driver.
- **32 GB of RAM.** The title uses about 16–18 GB in the world. Close other
  programs, and leave Windows' page file on "system managed".
- [Rust](https://rustup.rs/).
- Your own unpacked copy of the title (the folder with the title's `game_*.exe`).

## Build

```powershell
git clone https://github.com/AndreRese/victory-fork
cd victory-fork
cargo build --release -p victory
```

The repository pins the MSVC toolchain (`rust-toolchain.toml`), which needs
Visual Studio's C++ tools and the Windows SDK. Without the SDK, the GNU
toolchain builds it too (that is what was tested here):

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
cargo +stable-x86_64-pc-windows-gnu build --release -p victory
```

## Play

```powershell
.\run-rdna3.ps1 -GameDir "D:\path\to\your\game"
```

The script sets:

| Variable | Why |
|---|---|
| `VICTORY_F16_NEAREST=1` | 16-bit float results round to nearest even (AMD rounded some toward zero) |
| `VICTORY_RELAY=2` | sends submissions to the card ahead and lands them on a thread of their own |
| `VICTORY_WIDE_BACK=1`, `VICTORY_CARD_BUFFERS_MIN_MB=1`, `VICTORY_CARD_BUFFERS_MB=1024` | keep big title buffers in the card's memory instead of comparing them on the CPU every frame |

**If the screen goes black and the AMD driver resets**, run with `-Safe`:
it leaves the relay off. Slower, but the relay was what removed the device
before the fixes above.

```powershell
.\run-rdna3.ps1 -GameDir "D:\path\to\your\game" -Safe
```

Saves and settings are kept in `%LOCALAPPDATA%\victory\data`; GPU failures are
appended to `crashes.log` there.

## What to expect

- **The first run is slow.** Shaders are translated and pipelines built the
  first time they are used. The next runs reuse the cache
  (`%LOCALAPPDATA%\victory\cache`).
- **About 2 fps in the world**, with stalls of a few seconds when a new scene
  loads.
- **Dark scenes.** The title's ray-traced lighting does not run yet, so
  everything the sun or a lamp does not hit directly is black, and night
  shots are close to black. This is the same on every GPU, and no fix
  exists yet.
- **About a minute of plain black** between the opening night shot and the
  next scene is the title's own loading screen; wait for it.
- A few shaders still do not translate and some draws are dropped, so some
  objects can be missing.

## Reporting

Test results on other RDNA3 cards help. Include your card, driver, RAM, the
commit you built, whether you used `-Safe`, the frame rate from the window
title, and `crashes.log` if the device was removed. Leave out game files and
screenshots of game content.
