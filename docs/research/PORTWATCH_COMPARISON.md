# Trackers compared (2026-11-25)

Trackers are third-party readings of self-reported claims. None of their
scores, stages or frame rates is treated here as independent verification;
what was checked directly is in `REPOSITORY_RANKINGS.md` and
`INTEGRATION_RESULTS.md`.

| Tracker | State when read | Leader it names | Notes |
|---|---|---|---|
| https://portwatch.online/ ("PortWatch VI") | "Partial snapshot, sync unavailable"; newest history point 2026-11-24 | omonkey, 76 % "In-game" (private, no repository) | scores are "third-party estimates", not completion; "Playable builds: 0". VIctory 67 % "Main menu", XCL 62.5 % "Main menu" |
| https://4lwayz.github.io/portwatch-vi/ | not read: answers 301 to another host (`portwatch-vi.xyz`), not followed | - | - |
| https://vuyp.github.io/gta6-pc-race/ ("GTA VI PC Race") | "Checked 15:13 UTC on 25 Nov" | klbf-lab, stage 6 of 7 "In-game" | 32 open-source and 3 private projects; "Playable: no"; about 17 projects at "In-game" |

## Where this branch stands against the trackers' stages

By the race page's ladder this branch is at stage 6 ("In-game"): the
world renders lit through normal presentation with `VICTORY_FFX_BUILTIN=1`
(`docs/updates/fsr4-black-output.md`), the first gameplay scene is reached
with a controllable character (run `ffx-every`, frame 2059), and the world
ran for 5 min 40 s without a black frame. That is measured on this laptop
(Radeon 780M, 16 GB), at 1.4-1.9 fps.

## Claims on the trackers that line up with this branch's own findings

- Shadowed areas black in every VIctory-derived build (#1820, Belobug,
  DexNom): the title's ray-traced indirect light does not run anywhere
  stably; Belobug and now this branch stand a constant in.
- Builders with rays on end in device removal (Belobug ~25 s, this branch
  E10-E13), or run at about 1 fps (DexNom on an RX 7900 XTX, unverified).
- The console FSR4 is not emulated by anyone; XCL's HANDOFF.md decides
  the same as this branch (answer the FFX API with a simple upscale).
