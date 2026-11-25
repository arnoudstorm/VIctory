# Candidate improvements from other repositories (2026-11-25)

Each line: source commit, purpose, whether the defect reproduces on this
laptop, and what was done. "Integrated" means applied, tests passing,
measured in a game run, committed with attribution.

| Source | Commit | Purpose | Defect reproduced here | Outcome |
|---|---|---|---|---|
| Marvisak/VIctory #1729 | `b8dbcba2` (moe-miau) | a stray or unreadable save version folder deleted the save | yes: new synthetic test failed | **integrated** (`511f0be`) |
| Belobug (from fxceless #51 / `d82acef`) | `86292110` (pgm1207) | no blending on integer render targets | yes: mesh pipeline `f5e50eb2108c425b` refused in 10 of 10 runs | **integrated** (`6c5fe3f`), 0 refusals after |
| Belobug | `06031237`, `4f45ab88`, `4ffc4766`, `cd9f78ce` | constant ambient light where ray-traced probes give none | yes: everything outside direct sun near black | **integrated, opt-in** (`b7a122d`, `VICTORY_AMBIENT`) |
| Marvisak/VIctory #993 | `05394a69` (TG0000) | shared texture swizzle tables | texture refresh 52-57 ms median per frame | **integrated**; no in-game change measured (median 54.9 ms) |
| Marvisak/VIctory #1016, #172 | `924457e8`, `7d99f637` | XMA frame reads and IMDCT tables | audio decode on the CPU | **integrated**, bit-identical by their tests |
| Marvisak/VIctory #153 | `aa1bf518` (TG0000) | GPU private data released without scanning the map | lock held over every entry | **integrated** |
| Marvisak/VIctory #617, #1476 | `2fb667bb`, `f73e8e18` | shader translation speed | translation time at first launch | **integrated** |
| Marvisak/VIctory #1635, #126, #1000 | `94856e26`, `c813777b`, `b00c14ac` | indirect clamp skip, perf pack, parallel pipeline loads | - | conflict with this branch; not taken |
| Belobug | `2aad398d` | twice the dynamic descriptor slots | **no**: 0 tables refused per frame in 3 runs (3,300-5,700 asked) | not taken (memory for nothing) |
| Belobug | `03629f3f` | quad reads through helper lanes (lines on paint, glass, skin) on AMD | not shown: needs a GPU pixel test with helper lanes at an edge | candidate; the test first |
| Belobug | `72e748ea` | builders' writes ordered around atomics | the builder's wrong stores (E10) | candidate for the ray work; builder runs are memory-starved here (E13) |
| fxceless/victory-fork | `c96b4f2b` .. `0b0074e0` (8) | depth bounds, frame-rate work, copies in place | - | all conflict (this branch diverged); not taken blind |
| DexNom/VIctory | `bc3cd4b0`, `d71f1979` | ray structures built and traversed (experimental) | rays black here | building that tree needs the user's permission |
| xcl-dev/xcl | - | FSR4: answer the FFX API with a simple upscale | (same decision as here) | GPL-2.0-only: no code can be taken |
