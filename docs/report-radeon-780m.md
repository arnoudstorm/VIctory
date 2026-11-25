# Report: Radeon 780M (integrated, 16 GB shared)

Branch `laptop/780m`: fxceless/victory-gta6 main, plus upstream #454
(`VICTORY_NO_WATCHDOG`), the Series S multi-plane present (1984742628's
fork, `24a387d`) and an opt-in page pool used in place
(`VICTORY_POOL_COMMIT`). Nothing from the game is in this tree
(docs/asset-policy.md).

## Machine

- Ryzen 7 8840HS, Radeon 780M (RDNA3, integrated): 512 MB carved out,
  the rest shared with the PC; Direct3D budget about 6 GB
- 16 GB RAM, Windows 11, `TdrDelay` raised to 60 s
- Title: Series X|S build 1.0.2.1 (BLAKE3 `b99f876b74d4a77f…`), run in
  Series S mode

## Settings that run here

```
--console series-s
VICTORY_NO_WATCHDOG=1
VICTORY_TEXTURE_BUDGET_MB=1536  VICTORY_CARD_BUFFERS_MB=256
VICTORY_KEEP_SNAPSHOTS_MB=64    VICTORY_SHADER_LOOP_LIMIT=64
VICTORY_INPLACE=2  VICTORY_INPLACE_MB=6144  VICTORY_POOL_COMMIT=1
```

## What each step changed

| Run | Result |
|-----|--------|
| upstream `main` (+ #595, #1828, #454), Series X | out of shared GPU memory in the menus (4K targets) |
| the same, Series S | menus, then out of memory ~1 min later |
| this branch, budgets, Series S | menus at 30 fps, world reached in ~2.5 min at 0.1-1.3 fps, then Windows' 2 s GPU timeout (submissions of 1.5-2.4 s) |
| + `TdrDelay` 60 s | timeouts gone; out of memory ~2 min into the world (card 7.7 GB of a 6.0 GB budget) |
| + multi-plane present | menus and loading screens drawn (before: all frames black in Series S) |
| + `VICTORY_INPLACE=2` | 0.5 GB of title memory used in place; page-pool copies (3.3 GB) remain |
| + `VICTORY_POOL_COMMIT` | pool runs placed by address where one view holds them; most of the pool is scattered over views (586 turned down, 96 refused), so 3.2 GB of copies remain; still out of memory in the world |

Opening a whole committed section as one heap is not done: it counts all
6.1 GiB against the card's budget at once and every allocation after it
fails.

## Where the memory goes in the world

- host copies of title memory (shared buffers): about 3.2 GB
- textures the host renders to: about 1.1 GB
- textures the title fills: about 0.4 GB

## Next

- The texture mip drop of upstream #1296/#1617 (about 1.1 GB on 6-8 GB
  cards) does not cherry-pick: it creates host textures eagerly, where
  this base makes them on first use; it needs porting.
- Page-pool views mapped page by page: placing a buffer over several
  views would need one heap per view and a buffer per heap.
- More memory for the card: 32 GB of RAM doubles the shared budget.
