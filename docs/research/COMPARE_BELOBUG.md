# Side by side: Belobug/VIctory-Belobug-AMD and laptop/780m (2026-11-25)

Repository: https://github.com/Belobug/VIctory-Belobug-AMD, `main` at
`01aadd19`. Fetched as remote `belobug`; nothing merged.

## Where they stand

| | Belobug (README) | laptop/780m |
|---|---|---|
| Base | upstream `8e79c80` (2026-11-24) + 512 commits | the same base + this branch's work |
| Machine | RX 9070 XT 16 GB, Ryzen 7 9800X3D, 64 GB RAM | Radeon 780M, Ryzen 7 8840HS, 16 GB shared |
| Console profile | Series X | Series S |
| World frame rate | 3-5 fps | 1.5-1.9 fps |
| RAM | ~22 GB, commits ~33 GB after 15 minutes | ~12.5 GB private, free RAM down to a few MB |
| Upscaler (FSR4) | not mentioned | VICTORY_FFX_BUILTIN (ours) |
| Light in shadow | a constant stands in for the ray-traced indirect light (`VICTORY_AMBIENT`) | black (rays do not run) |
| Ray-traced light | `VICTORY_RAYS=1`: builders run, device removed ~25 s later | `VICTORY_BVH_BUILDER=1`: crash or OOM (E10-E13) |

Belobug's memory needs (22 GB+) do not fit this laptop, so a merge is not
an option; single changes are.

## Changes worth taking, by what they fix here

| Commit(s) | Change | Fixes here | Status |
|---|---|---|---|
| `06031237`, `4f45ab88`, `4ffc4766`, `cd9f78ce` | A constant ambient light written after `CS_ScreenProbes_TemporalResolve`, shaded from depth (`VICTORY_AMBIENT`, `_TINT`, `_SHADE`, `_FILE`) | black areas outside direct sun | **ported** (`ambient.rs`, opt-in here) |
| `86292110` (PR #45) | No blending on integer render targets in host pipelines | maybe the one mesh pipeline the driver refuses at every launch | candidate |
| `03629f3f` | Quad reads through helper lanes (lines on car paint, glass, skin) | rendering correctness | candidate |
| `f520772b` | A helper lane's `v_readfirstlane` reads its own value (upstream `0601955`) | rendering correctness | candidate |
| `72e748ea` | Builders' writes ordered around atomics, root buffers coherent | the ray builder's wrong stores (E10) | candidate, for the ray work |
| `a873b241` | GPU use-after-free in texpool/Staging/HostHeap | stability | check whether our code has the same paths |
| `2aad398d`, `788ea467` | Twice the dynamic descriptor slots; failed slot searches not repeated | dropped draws in heavy frames | candidate |
| `918ab038`, `76387c2d` | Hoisted clamps for runs of indirect draws (225 -> 80 ms of GPU time there) | GPU time (not our limit yet: CPU bound) | later |
| `0a32e89c` | Tiles of title memory no longer there given back (`VICTORY_SWEEP_TILES=2`) | memory | candidate, measure |
| `bdecda30`, `cf57ca33` | Asynchronous submissions (`VICTORY_ASYNC_SUBMIT`) | "no faster" there | skip |

## Not useful here

- Series X profile and its memory: more memory, the wrong direction.
- `VICTORY_RAYS`: the same device removal we see.
