# Startup and menu stability on the Radeon 780M (Phase 3)

Laptop: Radeon 780M, 16 GB shared RAM (~5.9 GB D3D12 budget), Windows 11,
branch `laptop/780m` at `7d0ed69`, diagnostics build, Series S profile,
settings of `run-laptop.ps1` (see `update-4-relay-off.md`).

Runs were automated: a harness started the game, sampled the process every
second (working set, private bytes, free RAM, Windows' "responding",
foreground window, GPU dedicated/shared every 5 s), brought the game to the
front and gave the front back, and stopped the game at a set time. **No key
was pressed** in these runs: the aim was the start screen with no input.
Nobody watched the screen.

## Milestones

| Milestone | Result |
|---|---|
| 3 consecutive successful startups | **Met.** 3 of 3 reached the start-screen video (4-39 s), no error before the title's own world load |
| Menu stable for 5 minutes | **Not met.** With no input the title starts loading the world 55-80 s after launch; memory runs out within ~2 minutes |
| Menu stable for 10 minutes | **Not met** (same cause; not attempted separately: run 1 shows the outcome) |
| Responsive input / window | **Partly.** Responsive on the start screen; Windows reports "not responding" for 11-15 s in every run when the world load starts |
| Window switching | **Met.** Game to front and back worked (both calls succeeded); frames kept coming at ~30 fps with the game in the background |
| No device removal | **Met** in these 3 runs (the earlier capture run ended with device removal after out-of-memory) |
| No out-of-memory | **Not met.** Run 1 (let run to 150 s): 20 out-of-memory errors, the game exited at 116 s |
| No unexplained memory growth | **Open.** On the start screen private bytes grow ~0.7 GB in 50 s (6.5 → 7.2 GB) while texture uploads grow (504 → 4,956 upload buffers); not yet attributed line by line |

## The runs

| Run | Length | Start screen at | World load starts | Not responding | Peak private | Free RAM low | GPU shared peak | End |
|---|---|---|---|---|---|---|---|---|
| start-1 | 150 s planned | 39 s | 94 s (first 500+ draw frame) | 80-91 s | 12.7 GB | 336 MB | 7.4 GB | out of memory, exited at 116 s |
| start-2 | 75 s | 4 s | after 70 s (load running at stop) | 65-75 s | 10.8 GB | 175 MB | 6.1 GB | stopped by harness |
| start-3 | 75 s | 4 s | 73 s | 55-69 s | 12.8 GB | 326 MB | 7.2 GB | stopped by harness |

The start screen came up at 4 s in runs 2 and 3 against 39 s in run 1;
why run 1 was slower was not looked into.

## What the "not responding" is

- In each run it starts exactly when free RAM falls below ~1 GB and Windows
  trims the game's working set (run 3: 7.1 → 2.2 GB in 10 s, free RAM
  887 → 326 MB). The game's private bytes keep rising past the 16 GB of the
  laptop (12.8 GB private plus ~2 GB for Windows and other programs): the
  rest is paged.
- It is **not shader translation**: in runs 2 and 3 all 1,482 and 1,786
  shaders came from the disk cache (`misses=0`). The 17 s without a new frame in run 3 hold 1,866
  `first bind` lines (the world's pipelines being bound) and 4 s with no log
  line at all (64-67 s), when free RAM was lowest.
- So the 5- and 10-minute milestones fail on the same memory limit as the
  world runs before (`update-3-memory-breakdown.md`): the card's share
  goes to 7.1-7.7 GB of a 5.9 GB budget during the load.

## What would move the milestones

- Less memory in the world load (the open problem since update 3): host
  copies of title memory (~3 GB) and render targets (~1 GB) are the largest
  parts.
- A start screen held without the world load is not something VIctory
  controls: the title starts the load itself, with no input.

## Tools (outside the repo, in the session notes)

`stability.ps1` (harness), `stability_check.py` (log and CSV check),
`run-laptop.ps1 -Keys none -LogName <name>.log`.
