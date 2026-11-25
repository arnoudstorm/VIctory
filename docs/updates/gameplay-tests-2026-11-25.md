# Gameplay tests on the Radeon 780M (2026-11-25)

Goal: reach the first gameplay. Branch `laptop/780m` at `46795c5`, laptop
settings (lazy lists and tiled windows on), diagnostics build, Series S.
Each test changed one setting. Enter (pad A) every 5 s from 15 to 240 s;
world frames saved at fixed points and checked for content. The series
was stopped after test 4 at the user's request. A Word version of the
whole session audit was made outside the repo (the asset policy keeps
binary files out).

| Test | Setting | Ended | World from | Furthest frame | World fps median | Frame time median / P95 | Peak card / budget | Least free RAM | Not responding |
|---|---|---|---|---|---|---|---|---|---|
| 1 | baseline | stopped at 330 s | 70 s | 1600 | 0.7 | 1,513 / 3,800 ms | 7,728 / 5,845 MB | 20 MB | 0 s |
| 2 | `VICTORY_RELAY=0` | stopped at 330 s | 79 s | 1608 | 0.9 | 1,151 / 2,323 ms | 7,568 / 5,839 MB | 4 MB | 0 s |
| 3 | `VICTORY_TEXTURE_ROOM_PERCENT=60` | device removed at 269 s | 90 s | 1692 | 0.8 | 1,331 / 3,824 ms | 7,776 / 5,877 MB | 7 MB | 0 s |
| 4 | `VICTORY_PRELUDE_POOL=1` | stopped at 330 s | 90 s | 1959 | 0.9 | 1,113 / 2,413 ms | 7,273 / 5,837 MB | 2 MB | 0 s |
| 5 | best combination + hold A to skip | not run | | | | | | | |

- **None reached gameplay.** World frames show the intro sequence: opening
  credits and artwork transitions on black (most frames fully black or
  credits on black). At under 1 fps the game advances a few seconds of
  its own time per minute.
- Pooled preludes went furthest (about 520 world frames against about 160)
  with the lowest peak and, with relay off, the best frame times. One run
  each: an indication, not proof.
- A 60 % texture share ended in a device removal.
- Next: hold A for several seconds to skip the intro, with pooled preludes
  and relay off combined.
