# Integration results (2026-11-25)

Every change below: source, defect, tests, measurement, outcome. Test
counts are `cargo test --workspace` on this laptop.

| # | Change | Source and attribution | Tests | Measured | Kept |
|---|---|---|---|---|---|
| 1 | Save versions: stray or unreadable folders never cost the save | upstream #1729, `b8dbcba2` (moe-miau) | +4 (1 new synthetic, 3 upstream); the new one failed before | - | yes, `511f0be` |
| 2 | No blending on integer render targets | Belobug `86292110` (pgm1207, from fxceless #51 `d82acef`) | +2 | refusals 10/10 runs -> 0 | yes, `6c5fe3f` |
| 3 | Least ambient light (opt-in) | Belobug `06031237`, `4f45ab88`, `4ffc4766`, `cd9f78ce` | +1 | darkest regions +18 %, frame +7 % at 0.3 | yes, `b7a122d` |
| 4 | Upstream speed PRs #993, #1016, #172, #153, #617, #1476 | upstream, authors kept (cherry-picked with `-x`) | +7 | texture refresh median 54.9 ms (baseline 54.9-57.2); frame 911 ms (baseline 759-1605) | yes: no regression, gains below the run-to-run spread |
| 5 | Bicubic, sharpened upscaler (opt-in) | ours | pass compiles (existing test) | hair edges +6-11 % | yes |
| 6 | GPU memory architecture in the log | ours | - | see SESSION_STATE | yes |

Rejected or not taken (with reasons): `UPSTREAM_CANDIDATES.md`;
experiments that failed: `FAILED_EXPERIMENTS.md` (E9, E10-E13, E17).

## Stability of the build (ten runs, 240 s each, `VICTORY_FFX_BUILTIN=1`)

| Runs | Ended by | Crashes / device removals / out of memory | World fps | Free RAM |
|---|---|---|---|---|
| 10 | harness 9, the process gone at 221 s 1 (free RAM 1 MB, no error logged) | 0 / 0 / 0 | 0.91-1.65 (mean 1.4) | 1-15 MB at the lowest |

The one silent exit happened with 1 MB of RAM free: memory is the
remaining threat to stability, not the renderer.
