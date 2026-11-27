# VIctory, Cycle VI (work in progress)

Research fork of [VIctory](https://github.com/Marvisak/VIctory), a
compatibility layer that runs an Xbox Series title natively on Windows from
files you supply yourself. This repository holds the Cycle VI work: pipeline
(PSO) prewarming, memory residency on shared-memory GPUs, and tests on a
Radeon 780M laptop (Ryzen 7 8840HS, 16 GB RAM). The findings are in
[docs/research/CYCLE_VI_PHASE_II.md](docs/research/CYCLE_VI_PHASE_II.md).

**This repository contains no game files, code or assets, and never will.**
You need your own legally obtained copy of the game. It is not affiliated
with or endorsed by the game's developer or publisher.

**Not playable.** On the 780M the world renders at about 1.7 frames per
second, and every run so far has ended with the GPU device lost once memory
runs out, about 3 to 4 minutes in. Expect that.

## Before you run it

1. **Hardware and Windows.** Windows 10 or 11 x64, a CPU with AVX2, and a
   Direct3D 12 GPU with shader model 6.6 and mesh shaders. 16 GB of RAM is
   the minimum tested; 32 GB is much better.
2. **Memory.** Leave the Windows paging file on (system managed is fine) and
   close other programs, browsers especially. The title needs several GB of
   Windows commit at start-up: check **Committed** under
   **Task Manager → Performance → Memory**. If `crashes.log` reports Windows
   error 1455, free memory or enlarge the paging file and try again. Do not
   turn off the paging file, memory compression or the GPU timeout recovery
   (TDR): they are what lets Windows recover when a run runs out of memory.
3. **Your game files.** Put your own copy of the game in a folder **outside**
   this repository. The tools refuse paths inside it.
4. **Rust.** Install [rustup](https://rustup.rs) and the Visual Studio C++
   build tools. The exact toolchain (1.98.1, MSVC) is pinned in
   `rust-toolchain.toml` and installs itself on the first build.
5. **Set up the checkout:**

   ```powershell
   cargo xtask install-hooks                # asset-policy check on every commit
   copy victory.example.toml victory.toml   # then set game_dir (outside the repo)
   cargo run -p victory -- config           # checks your paths
   ```

6. **Build:**

   ```powershell
   cargo build --release -p victory                                # target\release\victory.exe
   cargo build --profile diag -p victory --features diagnostics    # target\diag\victory.exe, with logging and captures
   ```

7. **Warm the pipeline cache (strongly recommended, once per build).** AMD's
   driver keeps compiled pipelines per executable, so the first run of a new
   build otherwise spends minutes compiling pipelines inside the world. Run
   the title once with `VICTORY_PIPELINE_BOOK=1` set, which records the
   pipelines it uses. Then warm them with the same shader settings as your
   runs (these are the launcher's `low` preset):

   ```powershell
   $env:VICTORY_SHADER_LOOP_LIMIT = '64'
   $env:VICTORY_F16_NEAREST = '1'
   target\release\victory.exe warm --used --jobs 4
   ```

   A cold warm takes about 10 minutes on the 780M. After a rebuild, run it
   again with `--again`. It pauses whenever free memory drops under 2 GB
   (`VICTORY_WARM_MIN_AVAIL_MB`).

## Running

On a 16 GB laptop like the 780M, use the launcher. It picks the newest build
and sets the measured settings:

```powershell
.\run-laptop.ps1 -GameDir "D:\path\to\your\game"
```

Or run the executable directly with your own settings:

```powershell
target\release\victory.exe run "D:\path\to\your\game\<the title's exe>" --execute
```

Saves, settings and `crashes.log` go to `%LOCALAPPDATA%\victory\data`;
shader and pipeline caches go to `%LOCALAPPDATA%\victory\cache`. Controls are
listed under *Getting started* further down.

## Automated tests

```powershell
cargo test --workspace
```

The tests need no game data and no GPU. CI runs on every pull request and on
every push to `main` ([.github/workflows/ci.yml](.github/workflows/ci.yml)):
Clippy with warnings as errors on the normal build and on the diagnostics
build, then all tests. A pull request has to pass them.

What the roughly 650 tests cover:

| Crate | Tests | What they check |
|-------|------:|-----------------|
| `victory-shader` | ~220 | The console shader recompiler: decoding GPU instructions, lowering to DXIL, the clean-up passes, loop limits, and that translations are deterministic (the same shader always gives the same output, so caches stay valid) |
| `victory-gpu` | ~200 | The Direct3D 12 back end in isolation: descriptor and resource bookkeeping, texture layouts and detiling, the pipeline queue and compile gate, the pipeline book used by `warm`, and the memory limits on texture copies (for example, draws stop making new copies while the card is over budget) |
| `victory-hle` | ~130 | The runtime services the title calls: users, licences, packages, saves that survive being killed at any step, input, audio decoding |
| `victory-formats` | ~30 | Archive formats, zlib and BCPack decompression |
| `victory-loader` | ~30 | Loading the title's executable and DLLs, imports, crash reports |
| `victory-core`, `victory-pe`, `victory-pipeline`, `victory-game`, `apps/victory` | ~30 | Shared helpers, the PE parser, pipeline packs, title-specific glue, command-line parsing |
| `xtask` | 4 | The asset policy: fails if the repository contains game files, executables or other forbidden binaries |

To run one crate's tests, use `cargo test -p victory-gpu`. To run tests by
name, use `cargo test -p victory-gpu draws_wait`.

Game runs on real hardware are not automated, because they need your own
game files. The research notes record each numbered test run by hand.

## Changing the settings

Settings are environment variables named `VICTORY_*`. Each one is described
in a comment in the code where it is read (search for its name).

**With the launcher**, pick a preset:

| Preset | For | Texture copy budget | Shader loop limit | Upscale filter |
|--------|-----|--------------------:|------------------:|----------------|
| `auto` (default) | `low` on 16 GB or less, otherwise `balanced` | | | |
| `low` | 16 GB laptops | 1,024 MB | 64 | bilinear |
| `lowmem` | 16 GB laptops that run out of memory in the world | 768 MB | 32 | bilinear |
| `balanced` | 32 GB PCs | 2,048 MB | 256 | bicubic |
| `quality` | 32 GB or more, dedicated GPU | 4,096 MB | 1,024 | bicubic |

```powershell
.\run-laptop.ps1 -GameDir "D:\path\to\your\game" -Preset lowmem
```

Other launcher switches: `-Safe` (submission relay off), `-Bilinear`,
`-NoAmbient` (no ambient-light fallback), `-AutoStart` (presses A through
the first screens), `-CutsceneTest`, `-CaptureDir <folder>`, `-NoCapture`,
`-BuildPath <exe>`, and `-DryRun` (prints the settings without starting).

The launcher sets its variables itself and replaces any you set before. To
change a single value, edit the preset table near the top of
`run-laptop.ps1`, or start `victory.exe` directly with your own variables:

```powershell
$env:VICTORY_TEXTURE_BUDGET_MB = '768'
$env:VICTORY_SHADER_LOOP_LIMIT = '64'
target\release\victory.exe run "D:\path\to\your\game\<the title's exe>" --execute
```

The settings you are most likely to change:

| Variable | Effect |
|----------|--------|
| `VICTORY_TEXTURE_BUDGET_MB` | The most memory host texture copies may use |
| `VICTORY_TEXTURE_MIN_FREE_MB` | Optional texture uploads stop below this much free RAM |
| `VICTORY_DRAW_TEXTURE_BOUND=0` | Draws make texture copies even while memory is short (the bound is on by default) |
| `VICTORY_KEEP_SNAPSHOTS_MB` | Memory for kept snapshots of title buffers |
| `VICTORY_SHADER_LOOP_LIMIT` | Translated shader loops stop after this many iterations (lower is faster but less exact). It is part of the shader cache key, so warm again after changing it |
| `VICTORY_F16_NEAREST` | Shader rounding mode; also part of the shader cache key |
| `VICTORY_RELAY=0` | Submission relay off (slower, safer) |
| `VICTORY_FFX_FILTER` | `bilinear` or `bicubic` upscaling |
| `VICTORY_AMBIENT` | Strength of the ambient-light fallback, or `off` |
| `VICTORY_PIPELINE_BOOK=1` | Records the pipelines the title uses, for `victory warm` |
| `VICTORY_WARM_MIN_AVAIL_MB` | `warm` pauses while free RAM is under this (default 2048) |
| `VICTORY_LOG=victory_gpu=debug,info` | More logging (diagnostics build) |

`victory.toml` holds only paths (game, cache and data folders), and
environment variables override it. More debugging variables are listed under
*Debugging* below.

## Contributing

Fork the repository and open a pull request against `main`. Only the owner
can push or merge. Never commit game files, executables, recordings, keys or
anything copyrighted: the asset-policy hook and CI refuse them.

## Credits

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors is the base of everything here: the loader, the runtime
  services, the Direct3D 12 back end and the shader recompiler. Licensed
  GPL-3.0-or-later; so is this fork. Upstream's history is kept intact below
  the single commit that carries this fork's changes.
- **[XCL](https://github.com/ivall/xcl)** by ivall and its contributors
  (GPL-2.0-only) was used as a reference for how the same title behaves under
  another layer. Fixes inspired by it were written anew in VIctory's code.
- This fork's changes were developed with [Claude Code](https://claude.com/claude-code).

## Earlier status of this fork on an RTX 4070 (2026-11-25)

**Not playable yet: 2.5 to 4 frames per second, and shadowed areas are black.**
Tested on one machine: Windows 11, Intel i7-14700KF, NVIDIA RTX 4070 12 GB,
32 GB RAM. The Series X profile is used throughout.

| Stage | State in this fork |
|-------|--------------------|
| Boot, menus, start screen | works, 30 fps (as upstream) |
| Opening cutscene | plays from start to finish, in order, lit: title cards, credits cards, night and dusk shots |
| Saves | the game autosaves a checkpoint and loads it again |
| First gameplay scene (the prologue) | renders: characters, textures, sky, sunlit exterior; HUD, pause menu and pad input work |
| Light in shadow | missing: the game's ray-traced bounce light and ray-traced lamp and sun shadows do not run yet, so anything the sun does not hit directly is black |
| Frame rate in the world | 2-4 fps: 420-450 ms per frame by default, 240-350 ms with the newer switches (`VICTORY_RELAY=2`, or `VICTORY_LATE_WAIT=3 VICTORY_WIDE_BACK=1 VICTORY_CARD_BUFFERS_MIN_MB=1 VICTORY_CARD_BUFFERS_MB=1024`); it started at 3-6 seconds per frame |
| Stability | no driver hang or device loss in normal runs (a hang introduced by the wave64 update was found and fixed: its barrier form for 64-lane programs is now off unless `VICTORY_WAVE_WIDE=1`) |

### What this fork changed (summary)

Rendering:
- Meshlet culling read its scene buffers as the wrong kind of view, so the world drew almost nothing; fixed.
- Per-draw descriptor tables could run out, after which draws used slots never made for them and the card hung; fixed.
- Light packed by the game's compute shaders was read back through empty copies (sky, probes, fog); fixed for every mip level and layer.
- The fog reconstruction, shadow-mask, hair, blend-shape skinning, fire and several material programs did not translate; fixed.
- Gathers at a named mip level (with and without an offset) read level 0; fixed.
- Implicit-level samples outside pixel shaders produced invalid DXIL in 730 compute programs; fixed.
- Rectangle lists, depth and stencil twins for depth textures written by compute, texture read-backs.
- Ray tracing, in progress and off by default: the game's own structure-builder programs translate, title memory can be read by raw address from shaders, the hardware ray/box and ray/triangle test is emulated in DXIL (checked against a reference on the GPU), and 59 of the 64 trace programs translate. Nothing is lit by it yet. See [docs/fork-ray-tracing.md](docs/fork-ray-tracing.md).

Speed and memory:
- Waits for memory values, write-watch queries and page queries no longer walk page tables per call.
- Large title buffers get a copy in card memory, with shader writes found by hashing on the card.
- Submissions can be sent ahead of the wait for the card (`VICTORY_LATE_WAIT`), with a relay thread that lands results as the card finishes (`VICTORY_RELAY`): in a first five-minute run the card's idle time behind the submitting thread fell from about 136 to 30 ms per frame.
- Title memory can be opened as a graphics heap so that buffers are used in place instead of copied (`VICTORY_INPLACE`, first stage only; a probe confirmed the approach on the test GPU).
- The bindless texture queue no longer uploads stale views; the view table no longer grows without bound; the pipeline library is mapped instead of read into memory.
- An intermittent device loss was traced to the pipeline library file being replaced while mapped; fixed.

- Translated shaders get clean-up passes over the finished module (dead code, constant folding, reuse of values, branch facts); the pack's host instruction count falls by about 43 %, worth about 20 ms of GPU time per frame on the test machine (`VICTORY_SHADER_OPT=0` turns them off).

Most of the newer speed changes sit behind environment switches that are off
by default (`VICTORY_LATE_WAIT`, `VICTORY_RELAY`, `VICTORY_WIDE_BACK`,
`VICTORY_CARD_BUFFERS_MIN_MB`, `VICTORY_INPLACE`, `VICTORY_BVH_BUILDER`, ...).
Each is described in a comment where it is read.

### Known problems

- Everything in shadow is black (see above).
- The interior of the first gameplay room is sometimes not drawn after the cutscene hands over to gameplay.
- Around 24 GB of memory is in use during play.
- A few style lints are switched off for this fork in the workspace `Cargo.toml` until the newer code is tidied.
- Many changes here are specific to this title and this GPU, and some are marked `shortcut:` in the code where they approximate what the console does.

Everything below this line is upstream VIctory's own README, unchanged; its
status section describes upstream, not this fork.

---

# VIctory

Runs the title (Xbox Series X|S) natively on Windows, from files
the user supplies.

The title's x86-64 code runs unchanged inside our process; VIctory supplies
the platform underneath it and converts the title's data, ahead of time where
it can and on first use where it must (shaders). See
[docs/architecture.md](docs/architecture.md).

## Status (2026-11-23)

**Not playable yet.** The title boots, runs its menus at its own 30 fps and
loads the world, where rendering fails.

| Stage | State |
|-------|-------|
| Boot, first-boot screens (accessibility, language, settings, legal) | works, 30 fps |
| Intro video with sound | works |
| Start screen, profile and saves | works |
| Loading into the world (about 50 s) | works |
| The world (first mission's intro) | reached and runs with indirect draws; its title card renders, the scene after it is still dark; 0.5-1.8 s per frame |

Tested on one machine: Windows 11, AMD RX 6700 XT.

> **Warning.** Entering the world on a real GPU can currently hang it, and
> Windows then resets the display driver (every monitor goes black for a
> moment). To look further, use a diagnostics build with
> `VICTORY_D3D12_WARP=1`, which renders on the software adapter.

What each part does today and what it still needs:

| Subsystem | Today | Needs |
|-----------|-------|-------|
| Loader | Runs the title's executable and its DLLs in our process; crash reports name the faulting code and are kept in `crashes.log` | – |
| Runtime services | One signed-in offline user, licences, packages, saves that survive a kill at any step, text entry dialog | Speech is silent; the dialog and relaunch are untested live |
| Network | Offline by design: the title cannot reach the network | – |
| Files and streaming | Archives and streamed data read, zlib and BCPack decoded in parallel | Loading is limited by writes into title memory |
| Input | XInput pads, keyboard, keyboard stand-in pad | Trigger rumble |
| Audio | Menu, video and loading sound; XMA voices and Opus streams decode | Reverb is silent; Opus results unverified against the title |
| GPU: menus | Draws, compute and presentation on host D3D12; only changed title memory is copied | Frame pacing drifts on one screen in release builds |
| GPU: world | Mesh pipelines, shader-written buffers, descriptor lifetimes, indirect execution (arguments clamped), pipelines made by worker threads and kept on disk | Most of the scene is still missing (untranslated shaders drop draws); frames take seconds; real queries, depth bounds, HDR and scaling are missing |
| Shaders | Console GPU code recompiled to DXIL at first use, cached on disk. Menus: all. World: mesh 8,191 of 8,506, vertex/pixel 10,617 of 12,346, compute 1,643 of 2,539 | 64-bit image atomics, merged geometry stages, the ray-tracing builder's shaders; some compute programs need a GPU with 64-lane waves |
| Textures | Console tiled layouts and detiling for the modes the title uses | Render-target metadata planes |
| Series S mode | Runs through the menus to the loading screen | The world, as above |

## Getting started

Requirements: Windows 10/11 x64, a CPU with AVX2, a Direct3D 12 GPU with
shader model 6.6 (and mesh shaders for the world), Rust (pinned in
`rust-toolchain.toml`).

The title also needs several GiB of available Windows commit at startup.
Check **Committed** under **Task Manager → Performance → Memory**. If
`crashes.log` reports Windows error 1455, close other applications or increase
the Windows paging file size, then retry.

```powershell
cargo xtask install-hooks                # asset-policy check on every commit
copy victory.example.toml victory.toml   # then set your paths (outside the repo)
cargo test --workspace                   # needs no game data
cargo run -p victory -- config           # check your paths
```

Run the title:

```powershell
cargo build --release -p victory
target\release\victory.exe run <game dir>\<the title's exe> --execute
```

On the 16 GB Radeon 780M laptop, `run-laptop.ps1` selects the newest build
in `target-planes` or `target`, enables the measured Series S memory profile,
the built-in upscaler and the ambient-light fallback:

```powershell
.\run-laptop.ps1 -GameDir "D:\path\to\your\game"
```

The default `auto` preset selects conservative settings on systems with 16 GB
of RAM. Use `-Preset low|balanced|quality` to choose explicitly. The low preset
uses the cheaper bilinear filter, caps optional host texture copies at 1 GB,
and stops optional uploads below 3 GB of free physical memory. `-AutoStart`
uses the diagnostics build and presses pad A through the first screens. Use
`-Safe` to turn the submission relay off, `-Bilinear` for the cheaper filter,
or `-NoAmbient` to inspect the title's lighting without the fallback. The
launcher warns on 16 GB systems because the current world can still page
heavily after boot even with the low preset.

To enter the opening cutscene without continuing to press A during it, capture
its rendered frames, and log the connected XInput pad's button changes, run:

```powershell
.\run-laptop.ps1 -GameDir "D:\path\to\your\game" -CutsceneTest
```

The capture folder is printed at launch. Pass `-CaptureDir <folder>` to choose
one, or add `-NoCapture` for a faster full-cutscene run. Press a controller
button during the run to see its state in the console. The balanced and quality
presets also use progressively less aggressive shader-loop limits (256 and
1,024) for better visual correctness than the low preset's performance-first
limit of 64.

The window caption shows the framerate. Saves, settings and `crashes.log`
are kept under the data folder (default `%LOCALAPPDATA%\victory\data`), derived
data under the cache folder.

Input: XInput pads. With none connected the keyboard is a pad while the
title's window has focus:

| Keys | Pad |
|------|-----|
| W A S D / I J K L | left / right stick |
| arrows | D-pad |
| Enter, Backspace, X, Y | A, B, X, Y |
| Q / E, Z / C | shoulders, triggers |
| F / H | stick clicks |
| Esc, Tab | Menu, View |

## Commands

```
victory inspect <exe|dll> [--imports] [--exports]
victory census [paths...] [--missing] [--strict]
victory lib-exports <import.lib> [--list]
victory shader-info [paths...] [--verbose] [--disasm]
victory translate [paths...] [--validate] [--pipelines] [--list] [--jobs N]
victory run <exe> [--execute] [--unimplemented trap|zero] [--strict]
            [--console series-x|series-s] [-- <title arguments>]
victory config
```

Logging: `VICTORY_LOG=victory_loader=trace,info` (a release build logs
info and above).

## Debugging

A release build has no debugging aids. Build them in with

```powershell
cargo build --profile diag -p victory --features diagnostics   # target\diag\victory.exe
```

| Variable | Effect |
|----------|--------|
| `VICTORY_LOG=<target>=debug,...` | debug and trace output |
| `VICTORY_STACKS=20,60` | every thread's stack at those seconds |
| `VICTORY_WATCH=<rva>,...` | logs a title u32 when it changes |
| `VICTORY_KEYS=30:Enter,35:X` | presses keys at those seconds, focus or not |
| `VICTORY_INPUT_LOG=1` | logs XInput connection and button/trigger changes |
| `VICTORY_CRASH_SAVE=<rva>:<len>,...` | saves title memory to the cache folder on a crash |
| `VICTORY_CAPTURE=<dir>` | saves presented frames (powers of two, or `VICTORY_CAPTURE_FRAMES=<list>`) |
| `VICTORY_DUMP_TARGETS=1` | with the two above, saves every render target of those frames |
| `VICTORY_D3D12_DEBUG=1` | host debug layer |
| `VICTORY_D3D12_GBV=1` | with the above, GPU-based validation (bad descriptor use reported, not faulted; slow) |
| `VICTORY_D3D12_DRED=1` | on a GPU failure, the allocation hit and the commands in flight |
| `VICTORY_PIPELINE_WAIT_MS=24` | reduce the per-frame wait for background pipeline creation from the default of 500 ms. A draw can be omitted until its pipeline is ready |
| `VICTORY_SKIP_PIPELINES=<hash>,...` | omit draws and dispatches, including indirect ones, whose pipeline contains a listed 16-digit shader hash |
| `VICTORY_GPU_TIMING=1` | with the above, each list of a submission run and timed on its own; slow lists logged with their slowest commands |
| `VICTORY_D3D12_WARP=1` | host device on the software adapter (slow; cannot reset the driver) |
| `VICTORY_SHADER_LOOP_LIMIT=<n>` | translated shader loops stop after `n` iterations per lane instead of 2^20 (own shader cache entries); if a lower `n` makes world frames much faster, loops hitting the limit are what makes them slow |
| `VICTORY_AUDIO_CAPTURE=<file.wav>` | records what the title plays (our process only) |
| `VICTORY_XMA_DUMP=<dir>` | saves the first 32 XMA voices' context and input |

Every build appends crashes and host GPU failures to `crashes.log` in the
title's data folder.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
