# Flicker at the start screen (photosensitivity)

**Status: cause found and fixed in the emulator; residual changes remain.**
This is not a statement that the start screen is safe for people with
photosensitive epilepsy: brightness levels and flash counts are measured
below, but no photosensitivity test (such as a Harding analysis of
red flashes and patterns) was run. Do not watch the start screen's
transitions on the strength of this file.

## Cause (confirmed 2026-11-25)

- The start screen is a video. The title decodes it on the CPU into a
  **ring of three** YUV texture sets (3840x2160 luma, two 1920x1080 chroma),
  and pixel shader `3bfe73…` turns slot N mod 3 into the UI plane each frame.
- Every jump repeated the frame **3 frames before** (52 of 67 jumps): a slot
  showed the image from its previous ring cycle.
- Why: VIctory notices that a title texture changed through a guard on its
  pages. The decoder writes the ring through **another view of the same
  pages**, and a write through that view does not trip the guard. The
  texture was then thought unchanged and only re-hashed every 8th frame, so
  most frames showed the old slot.
- The test that proved it: with the guards off (`VICTORY_TEXTURE_GUARD=0`,
  every texture hashed on every check) the ring repeats fell from 52 to 1.
  The earlier "decoder writes after submission" reading was wrong: the new
  read-timing log (`title textures written after this frame read them`)
  fired on only 3-4 frames.

## Fix

- `inplace.rs` counts how many views show each page of title memory;
  `inplace::aliased(at, len)` says whether any page of a range is shown by
  more than one.
- `texture.rs` (`refresh_timed`): a texture whose guard was not tripped is
  skipped as unchanged **only if none of its pages is aliased**. Aliased
  textures are hashed on every check, as with the guards off; all others
  keep the guard.
- Unit test `inplace::alias_tests` (a page shown by two views is aliased,
  and stops being when one view goes).
- Diagnostics builds also log a texture that changed but was skipped as
  unchanged; it fired on every ring slot before the fix and **not once**
  in the run after it.

## What was measured

Runs on the laptop branch (Radeon 780M, Series S, diagnostics build) saved
every presented frame (`run-laptop.ps1 -Capture -FlickerFrames 300-1100`);
`frame_levels.ps1` took each frame's mean luma (Rec. 709, every 4th pixel);
`flash_stats.py` counts jumps of 20+ (of 255) between frames, opposing pairs
of them, and the most pairs in any 30 frames.

| Run | Frames | Jumps 20+ | Opposing pairs | Most in 30 frames | Jumps repeating frame-3 |
|---|---|---|---|---|---|
| Before | 801 | 67 | 60 | 24 | 52 |
| Upload in the same submission (`8d2591e`) | 801 | 71 | 65 | 29 | 57 |
| Guards off (experiment) | 760 | 21 | 10 | 4 | 1 |
| **Alias fix** | 668 | **22** | **12** | **6** | **2** |

- The alias-fix run stopped at frame 967 (out of memory loading the world,
  the known memory limit, not this change), so it covers 300-967.
- **Real time:** capturing slows the game to 3.4 frames a second, so 30
  frames are not one second here. Timed by the log, the alias-fix run had
  **one** flash (two opposing jumps within 0.5 s) in 194 s
  (`flash_realtime.py`). The 30-frame column is the cautious figure for a
  game running at 30 frames a second.
- What the remaining jumps are: fades counted over 2-3 frames (316-322,
  568-593), single cuts back to the dark star screen about every 50 frames
  (768, 822, 868, 918), and fully black frames at 720-724 and 931-965
  (after Enter, while the world loads). The 6 in 30 frames come from those
  black frames. Whether the title draws them black too is not known.
- Cost: texture refreshes took 3-6 ms a frame on the start screen in the
  alias-fix run (all threads); no global GPU wait was added.

## What it is not

- **Not a plane the title leaves out.** The present's planes alternate A/B
  on every frame logged, with no repeat and no missing plane.
- **Not the present's copy list being reset while it runs.**
- **Not the decoder writing after the GPU read** (see above).

## Still open

- The black frames at the start of loading: dump both planes there
  (`VICTORY_DUMP_TARGETS`) to see whether the title draws them.
- A presentation guard as a safety net (present the newest frame again when
  a frame would undo a large change within a few frames, logged) was not
  built; the cause is fixed instead.
