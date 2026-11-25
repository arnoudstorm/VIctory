# Flicker at the start screen (photosensitivity)

**Status: measured, cause not yet found, not fixed.** Do not watch the
start screen's transitions until it is.

## What was measured

A run on the laptop branch (Radeon 780M, Series S, diagnostics build) saved
every presented frame from 300 to 1100 (`run-laptop.ps1 -Capture
-FlickerFrames 300-1100`); `frame_levels.ps1` took each frame's mean luma
(Rec. 709, every 4th pixel).

- 801 frames, none missing.
- **67 jumps of 20 or more** (of 255) between consecutive frames, in bursts
  at frames ~379, 523, 571, 627 and later: each burst is about five
  alternations, e.g. levels `32, 72, 32, 32, 72, 32, 32, 76, 72, 32, ...`.
- **Up to 24 opposing pairs of jumps in one second** (30 frames). WCAG 2.3.1
  and the Harding test allow 3 flashes a second.
- What alternates: during a transition the picture switches between the
  **previous screen** (dark, a glowing star) and the **next screen** (an
  artwork zooming in) every one to two frames, instead of fading between
  them (frames 379 and 380 side by side).

## What it is not

- **Not a plane the title leaves out.** A log of every change in the
  present's planes (`present planes changed`, new in `queue.rs`): the title
  passes two planes on every present, double buffered, alternating A/B on
  every one of the 600 frames logged, with no repeat and no missing plane.
- **Not the present's copy list being reset while it runs.** With the relay
  the present still goes through `Host::execute`, which waits for everything
  in flight first.
- **Not something the title's planes would explain by themselves:** the
  strobe repeats every ~3 frames, the planes every 2.

## Where to look next

- Which plane texture holds the old screen on the frames that show it: dump
  both planes (`VICTORY_DUMP_TARGETS` with listed frames) at frames 379-393.
- Whether a plane's host copy is stale on those frames (a placed texture
  whose host copy is not brought up to date when the title renders into it).
- A safety guard in the presenter as a stopgap: when a frame would undo a
  large change of the frame before within a few frames, present the newest
  frame once more instead, so that no more than 3 such changes reach the
  screen in a second. It would show only frames the title drew, and be
  logged; it does not replace finding the cause.
