# How long the game ran at each frame rate

From the overlay line VIctory logs once a second (`victory_gpu::overlay`), per run. A line covers
about a second, or one frame when a frame takes longer. Radeon 780M, 16 GB, Series S, diagnostics build.

Seconds without any presented frame (long loading stalls) are not in these tables: a
line is written only when frames are presented. In the world the frame rate never rose
above 2.1 fps in any run; the 20-30+ fps rows are the menus and loading screens.

## Per run

| Run | Ran in all | In the world (500+ draws) | World fps min / mean / max | Peak RAM | Peak VRAM |
|---|---|---|---|---|---|
| test 1 | 79 s | 0 s | 0.0 / 0.00 / 0.0 | 9130 MB | 4079 MB |
| test 2 | 149 s | 44 s | 0.3 / 0.89 / 2.1 | 13233 MB | 7705 MB |
| test 3 | 53 s | 5 s | 0.4 / 0.73 / 0.9 | 12604 MB | 7585 MB |
| test 4 | 54 s | 6 s | 0.3 / 0.60 / 0.8 | 12644 MB | 7688 MB |
| in-place runs | 88 s | 4 s | 0.6 / 0.83 / 1.0 | 11836 MB | 7792 MB |
| card copies off | 105 s | 52 s | 0.3 / 0.74 / 1.6 | 13177 MB | 7733 MB |

## Seconds at each frame rate

| fps | test 1 | test 2 | test 3 | test 4 | in-place runs | card copies off | all runs | frames drawn |
|---|---|---|---|---|---|---|---|---|
| 0.0-0.1 | – | – | – | – | – | – | 0 | 0 |
| 0.1-0.2 | – | – | – | – | – | – | 0 | 0 |
| 0.2-0.3 | – | – | – | – | – | – | 0 | 0 |
| 0.3-0.4 | – | 3 | – | 3 | – | 13 | 19 | 6 |
| 0.4-0.5 | – | 2 | 3 | – | – | 10 | 15 | 6 |
| 0.5-0.6 | – | 8 | – | – | – | 10 | 18 | 9 |
| 0.6-0.7 | – | 4 | – | – | 2 | 2 | 7 | 4 |
| 0.7-0.8 | – | 4 | – | 1 | – | 7 | 13 | 9 |
| 0.8-0.9 | – | 7 | – | 1 | – | 3 | 11 | 9 |
| 0.9-1.0 | – | 4 | 2 | – | 1 | 6 | 13 | 12 |
| 1-2 | 2 | 11 | – | – | 2 | 6 | 21 | 26 |
| 2-3 | – | 1 | – | – | – | – | 1 | 2 |
| 3-4 | 1 | – | – | – | – | – | 1 | 3 |
| 4-5 | 2 | – | – | – | – | – | 2 | 9 |
| 5-6 | – | – | – | – | – | – | 0 | 0 |
| 6-7 | – | – | – | – | – | – | 0 | 0 |
| 7-8 | – | 1 | – | – | – | – | 1 | 8 |
| 8-9 | – | – | – | – | 1 | – | 1 | 9 |
| 9-10 | 1 | – | – | – | – | – | 1 | 10 |
| 10-11 | – | – | – | – | – | – | 0 | 0 |
| 11-12 | – | 1 | – | – | 1 | – | 2 | 23 |
| 12-13 | 2 | 1 | – | – | 1 | – | 4 | 50 |
| 13-14 | 1 | – | – | – | – | – | 1 | 14 |
| 14-15 | – | 1 | – | – | – | – | 1 | 14 |
| 15-16 | 1 | 1 | – | – | 1 | – | 3 | 47 |
| 16-17 | 1 | 4 | – | – | 1 | – | 6 | 100 |
| 17-18 | 1 | – | – | – | 1 | 1 | 3 | 53 |
| 18-19 | – | 2 | – | – | – | – | 2 | 36 |
| 19-20 | 1 | 2 | – | – | – | – | 3 | 58 |
| 20-21 | 1 | 2 | 1 | 1 | 2 | 1 | 8 | 163 |
| 21-22 | 3 | 1 | 1 | 1 | 4 | – | 10 | 215 |
| 22-23 | 1 | – | 1 | 2 | 1 | – | 5 | 113 |
| 23-24 | 1 | – | – | – | 1 | 2 | 4 | 95 |
| 24-25 | 5 | 4 | 1 | – | 1 | – | 11 | 269 |
| 25-26 | 4 | 3 | – | 1 | 3 | – | 11 | 280 |
| 26-27 | 5 | 6 | 1 | – | 3 | – | 15 | 396 |
| 27-28 | 2 | 5 | – | – | 2 | – | 9 | 247 |
| 28-29 | 1 | 14 | 2 | 4 | 8 | 3 | 32 | 908 |
| 29-30 | 8 | 11 | 5 | 8 | 10 | 10 | 52 | 1526 |
| 30+ | 35 | 45 | 36 | 31 | 42 | 33 | 222 | 6720 |
