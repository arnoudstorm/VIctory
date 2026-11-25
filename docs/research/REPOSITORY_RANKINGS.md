# Repositories checked (2026-11-25)

Availability, ownership and the head commit were read from GitHub; the
common ancestor from this repository's history (remotes added with
`--filter=blob:none`). "Claimed" is the repository's own README or a
tracker; "seen here" is what was verified in this branch.

| Repository | Head | Licence | Relation to this branch | Claimed | Seen here |
|---|---|---|---|---|---|
| Marvisak/VIctory (upstream) | `940d32f69b` | GPL-3.0 | base `ffe310e`; 319 commits since | world, 1-2 s per frame | 6 perf PRs and 1 save fix applied cleanly and pass tests |
| JMAN730/project-flamingo | `73625c4fb3` | GPL-3.0 | fork of upstream, 780 commits past ours | - | `f219769` parses FSPX (diagnostic only) |
| xcl-dev/xcl | `082944135c` | **GPL-2.0-only** | independent code base; code cannot be copied into this GPL-3.0-or-later project | menus 30 fps, ~5 fps world on RTX 5090 | HANDOFF.md: same FSR4 decision and FSPX layout as this branch |
| Belobug/VIctory-Belobug-AMD | `01aadd193d` | GPL-3.0 | upstream `8e79c80` + 512 | 3-5 fps, RX 9070 XT, 64 GB RAM | ambient light and integer-target blend ported and validated |
| DexNom/VIctory | `8cca4d2285` | GPL-3.0 | fork of upstream, 11 own commits past it | RT experimental, ~1 fps on RX 7900 XTX | not built: building a third-party tree needs the user's permission |
| fxceless/victory-fork (was victory-gta6) | `0b0074e0b2` | GPL-3.0 | this branch's lineage, base `a803c87`; 8 commits not here | 7-12 fps (tracker) | all 8 conflict with this branch (diverged); integer-target blend came via Belobug |
| PastiiJay/klbf-lab | `092a824aaa` | GPL-3.0 | no common history (imported) | race leader, 5-9 fps on RX 9070 XT | not compared further |
| realblastex/flamingo | `1f70cacba8` | GPL-3.0 | no common history | 9-14 fps, unconfirmed | - |
| LooperSalty/AnyXboxX | `79f115b787` | GPL-3.0 | no common history | 3-6.9 fps | - |
| Zettrox/ztrx | `90b529f878` | GPL-3.0 | upstream `ffe310e` + 13 | 1-3 fps, RTX 3050 | - |
| vitutu1/victory-perf | `ffed22db45` | GPL-3.0 | upstream `ffe310e` + 35 | 4.5-14 fps, unconfirmed | its own last commit backs out its VRAM policy changes |
| ka1qux/BRVIctory-ka1qux | `3d706c9c24` | GPL-3.0 | upstream `ffe310e` + 199 | 2-6.9 fps | - |
| uiliannn/VItoriaBr | `d0af132b6a` | GPL-3.0 | upstream `ffe310e` + 110 | 3.5-6 fps | - |
| myokex/victory-amd | `070df424b0` | GPL-3.0 | fork of fxceless/victory-fork, + 21 | 2-3 fps | - |
| voidopcode/vilocity, ZigZagBoi1/ScarlettPort, R0Y00/Xbox2Windows | `c8b59e5a63`, `d8eee74c0d`, `7092683a9a` | none / NOASSERTION / MIT | not VIctory-derived | - | not usable here (licence or base) |

Ranking for this branch's purposes (what can be taken and verified on a
16 GB Radeon 780M): upstream PRs first (clean, tested, attributed), then
Belobug (closest problems, AMD), then the fxceless line (same lineage but
diverged), then DexNom (rays, needs a build permission). Frame-rate
claims of 5-14 fps come from 16-24 GB discrete cards with 32-64 GB of
RAM; none was reproduced here.
