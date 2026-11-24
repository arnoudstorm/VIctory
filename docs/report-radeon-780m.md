# Report: Radeon 780M (integrated) and the Prestream stop (#1118)

This branch is upstream `main` (`70564ae`) plus three open pull requests, put
together to test the Prestream stop (#1118) on a slow host GPU. Nothing from
the game is in this tree (docs/asset-policy.md); offsets and hashes only.

| Commit | From |
|--------|------|
| GPU: run submissions asynchronously on a host fence timeline | #595 |
| GPU: a present waits for the previous frame's submissions, not its own | #1828 (stacked on #595) |
| Loader: opt-in `VICTORY_NO_WATCHDOG` hook for the title's watchdog (build 1.0.2.1) | #454 |

## Machine

- Ryzen 7 8840HS with Radeon 780M (RDNA3, integrated, shares system memory;
  budget reported 7,542 MB), driver 32.0.13074.2001
- 16 GB RAM, Windows 11 Home, page file managed by the system (commit limit
  about 55 GB)
- Title: Series X build 1.0.2.1, BLAKE3 `b99f876b74d4a77f…`

## Runs (scripted menu input with `VICTORY_KEYS`, diagnostics builds)

| Tree | Result |
|------|--------|
| `fx-heapmap` (Xeltax/VIcky, on fxceless `7b0f736`) | menus, sign-in, world loading at 4 fps; a 5.3 s host submission, then the title's watchdog (`+0x23cf683`) after about 1 min, Prestream, Game Physical 95%, TileHeap 3,673/6,236 MB |
| `main` + #454 | device removed in the menus 30 s after sign-in, after a 2.7 s submission |
| `main` + #595 + #1828 + #454 | device removed in the menus, 30 s after sign-in |
| the same + `VICTORY_SHADER_LOOP_LIMIT=4096` | device removed in the menus, 45 s after sign-in |

In all three `main` runs the device removal (`0x887A0004` / `0x887A0007`)
comes 3 s after allocations start failing with `E_OUTOFMEMORY`
(`0x8007000E`): an 11 MB shared buffer and depth twins, while the card itself
reports 1.7 GB of a 7.5 GB budget in use. The title's own crash context in
the watchdog run shows the page file nearly exhausted (1.2 GB of 55 GB free).
On this machine the stop is reached through **system commit running out**,
with other applications holding about 22 GB.

The `VICTORY_NO_WATCHDOG` hooks apply on this build (`applied=2`).

## Tests on this host

`cargo test --workspace` passes on `main` (476 tests). On this branch one
test added by #595 fails on the 780M:

```
host::submissions::a_repeated_list_reads_its_own_upload_memory
between the runs, then after the repeat
  left: [0, 572662306]
 right: [286331153, 572662306]
```

The list between the two runs of a repeated list reads 0 where it should
read the first run's upload bytes.

## Not done yet

- A run with most other applications closed, to tell the memory limit apart
  from the stop itself.
- The world with the async submissions on this GPU.
