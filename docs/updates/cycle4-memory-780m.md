# Cycle IV, Phase 1: where the memory goes on the Radeon 780M (16 GB)

Measured on 26 Nov 2026, Ryzen 7 8840HS / Radeon 780M, 15.3 GB visible RAM. Release build of `d82d0f39`, `low` and
`lowmem` presets with relay 2 and texture no-wait. Tests #22–#25 of the laptop log. Samples every ~5 s of the Windows
`Memory` and `GPU Process Memory` counters, plus the new `title pages N MB` field of the status line. No game data
is quoted here.

## The game's physical pages are not "private bytes"

They are pagefile-backed section pages (`XMemAllocatePhysicalPages`):

- **4.2 GB is committed within the first ~2 s, and 4.5 GB by the end.**
- **Only 0.03–2.2 GB of it is resident at any time.**
- **Commit charge is not the limit:** 32.7 of 48.6 GB at worst.

## Loading: the first screen lasts about 3 minutes

The first loading screen runs at 30 fps with 20 draws or fewer, and lasts ≈3¼ minutes with 1,300 pipelines cached and
≈2¾ minutes with 2,500.

The guarded runs (stop at ≤256 MiB available, or 2 s under 512 MiB) all ended at ≈60 s, so they never left it. At that
point (#22) the game's resident memory is ≈7.1 GB:

| Category | Resident |
|---|---|
| Private (the game's other allocations + the runtime) | 5.1 GB |
| Shared (game pages, images) | 0.8 GB |
| GPU shared (system RAM the GPU maps) | 1.3 GB |
| GPU dedicated (carve-out) | 14–286 MB |

## The world: the footprint roughly doubles

Unguarded, the `lowmem` preset gets through loading twice (#24, #25). World frames then render behind the black
gameplay loading screen with a tip box: 6,700–7,450 draws per frame at 0.8–1.9 fps.

| Measurement | Loading | World |
|---|---|---|
| Private commit | 7.6 GB | 12.7–14.6 GB |
| GPU shared | 1.5–1.7 GB | 6.1–7.6 GB |
| GPU budget (DXGI local) | 7.5 GB | 6.4–6.5 GB |

The GPU budget shrinks as RAM runs out. Once GPU shared memory passes it, D3D12 allocations fail: 439–1,119 host
textures, 25–39 shared buffers, then a command list that cannot close. The device is removed at 243–386 s.

With card buffers at 128 MB and the `low` preset (#23), the run died earlier, still in loading (171 s), after a 61 s
paging stall.

## What this means

- **The hard ceiling in the world is the GPU budget (≈6.4 GB here), not the commit limit or the game's pages.**
- **The gap is ≈1–1.5 GB of GPU memory**, plus the paging the doubled private memory causes.
- **Next:** split the 6–7.5 GB of GPU shared memory into textures, host copies of buffers, upload/staging and card
  buffers, and split the private memory into the game's and the runtime's. Both are runtime counters, measured before
  any change.
