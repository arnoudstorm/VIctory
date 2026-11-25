# Project Flamingo, compared again (2026-11-25)

`https://github.com/JMAN730/project-flamingo`, `main` at `83f6767`, against
`laptop/780m`. Analytical only: nothing merged, built or run.

- Common ancestor: `ffe310e` (2026-11-24, unchanged).
- **Flamingo-only commits: 676** (551 at the last comparison). Ours only: 63.

## Commits that bear on what was measured on the 780M

| Commit | What it does | Our measurement it touches | Applies cleanly here |
|---|---|---|---|
| `a79befd` HLE: map a page scatter in runs, not one page at a time | Fewer, larger views of title memory | The census: ~100,000 views of single pages and **0 asks inside one view**: why title memory can not be used in place (page-pool redesign) | No |
| `d549a33` HLE: keep scatter-mapped pages independently replaceable on Windows | Same code (`xmem.rs`), later than `a79befd` | Same | No |
| `ec9247f` HLE: show title windows through a stand-in that always reads its messages | The visible window is pumped by a thread of VIctory's own; the title's is message-only | Phase 3: 11-26 s of "Not responding" in every run at the world load | No |
| `d3ebb55` Shaders: fix DXIL signature mismatches and prune oversized mesh outputs | Shader translation fixes | Phase 4: a mesh pipeline refused by the driver (`0x80070057`), dropped every frame | No |
| `e411b71` GPU: mesh pipelines load from the pipeline library | Mesh pipelines cached | Load-time pipeline creation | not checked |
| `#559` proton sparse world copies, `#579` resident upload | Wine/Proton world copies and uploads | Copies of title memory | not checked |

## What it would take

Each of the four needs adapting by hand (our branch and Flamingo have
diverged since `ffe310e`), then the suite and an automated game run.
`a79befd` + `d549a33` are the ones that could change the memory picture:
with views in runs, windows could lie inside one view and be opened in
place instead of copied. `ec9247f` addresses the "Not responding" milestone
directly.
