# Failed or rejected experiments

## Porting upstream's PC FidelityFX path (0b35b7f): not attempted

It needs AMD's PC FidelityFX SDK DLL (`amd_fidelityfx_upscaler_dx12.dll`,
MIT). It is not on this machine, and downloading it needs the user's OK.
It is kept as an option: it would give temporal upscaling (FSR 3.1 on
RDNA3) instead of bilinear.

## Translating FSR4's ACCC/FSPX programs: not attempted

It would mean a new container reader plus whatever instructions the ML
upscaler uses (likely dot4 and packed math). Flamingo `f219769` only
parses the header. Effort is large and the gain over E3 is image
quality, not correctness of the rest of the frame.

## E5 (2026-11-25): `VICTORY_CARD_BUFFERS_MIN_MB=1`, run `card-buffers`, no effect

- Hypothesis: the log line "VICTORY_CARD_BUFFERS_MIN_MB=1 would have the
  card look there" means that moving the shader-write search for buffers
  of 1 MiB or more to the card would cut the 403 MB per frame compared
  on the CPU.
- Actual: 1 MiB is **already the default** (`shadow.rs`, `card_limits`;
  it was 16 MiB), so the run changed nothing. The CPU still compared
  416.5 MB in 408 ranges per frame. The log hint is out of date.
  Why those copies stay on the CPU is not known yet; the 1,024 MB cap
  (`VICTORY_CARD_BUFFERS_MB`) may be full.
- Useful by-product, **run-to-run spread**: two runs with identical
  settings (`ffx-every`, `card-buffers`) averaged 848 and 715 ms per
  world frame (1.32 and 1.50 fps; CPU 379 and 306 ms, GPU wait 378 and
  359 ms; 226 and 261 samples). A frame-time change has to beat about
  15 % to count from one run each. Neither run had a black frame after
  the cut or a device removal, and free RAM fell to 7 MB in both.

## E9 (2026-11-25): in-place runs (`VICTORY_INPLACE_RUNS=1`), run `inplace-runs`: not worth building on

- Hypothesis (E8): routing asks to in-place runs removes most of the 84
  ms of moves per frame.
- Census of the windows asked for over the run (`inplace.rs`):

| How the memory lies | Asks | MB | Can be in place |
|---|---|---|---|
| one run of one section | 1,865 | 636 | yes |
| scattered over the page pool | 3,551 | 1,416 | no |
| partly outside the runtime's views | 3,939 | 4,554 | no |

  The page pool is mapped 64 KiB at a time (85,500 views alive, 9.5 GB),
  so a window over it is never one range of anything the card can open.
  At most about a tenth of the asked memory could be in place. 317 MB
  was in place at the end; 1,206 windows were let go because the title
  remapped their pages.
- Measured against `staging-fix`: 696 vs 732 ms per frame (inside the
  15 % spread), but moves went **up** (84 -> 115 ms), and free RAM fell
  to 2 MB. Off, as in `run-laptop.ps1`.
- Conclusion: copying title memory is structural, because the console's
  page mapping has no host equivalent. Cutting it needs the copies to be
  skipped where nothing changed (the write watch already does this), not
  in-place memory.

## E10 (2026-11-25): ray-tracing structure builder on (`VICTORY_BVH_BUILDER=1`), run `bvh-on`: the title crashes

- Hypothesis: with the builder running, the title's ray-traced indirect
  light and shadows work, and shadowed areas stop being black.
- Actual: the builder now runs on this card ("all of its programs have
  host pipelines", 53 pipelines; `fork-ray-tracing.md` said some were
  refused). The first world frame took 11 s, and 22 s after the builder
  started the title crashed in its own code: access violation reading
  through `rcx = 0x500150010000d000`, a bounds-like bit pattern, not a
  pointer. The title's crash reporter tried to upload (blocked by
  VIctory's network policy). Same signature as run 98: builder stores land
  on the title's own objects.

## E11: builder on with its stores kept off title memory (`VICTORY_BVH_QUARANTINE=1`), run `bvh-quarantine`: device removed

- Actual: no title crash, but the **host GPU device was removed** at the
  first world frame (1 submission in flight, no breadcrumbs). Private
  memory peaked at 14.1 GB, free RAM 5 MB.
- Conclusion: the ray-traced lighting is blocked by the builder and
  traversal emulation itself: wrong stores without quarantine, a GPU
  fault with it. This is the work `fork-ray-tracing.md` lists as known
  gaps. It needs a synthetic test of one builder pass against known
  input on this card before any further game runs. Not a quick fix.

## E13 (2026-11-25): builder checks offline and with DRED

- Offline (`victory translate --validate --pipelines --list` over the
  title's `XboxBvhBuilder.dll`, locally, nothing kept in the repo): all
  52 builder programs translate, pass the DXIL validator, and get host
  pipelines on the 780M. The defect is in what they compute (or in the
  memory around them), not in translation or validation.
- In-game with `-Dred` (`bvh-dred`, builder plus quarantine): **out of
  memory** 48 s after the builder started (`0x8007000E`: no heap for
  window tiles, then a host command list did not close), at 112 s.
  E11's device removal came at the same point with private memory at
  14.1 GB of 16 GB. So the builder runs cannot yet be told apart from
  memory exhaustion on this laptop.
- Next for lighting: either a GPU test of one builder stage on synthetic
  input (needs the builder's input layout written down first: spilled
  user-data table, root views, node format in `bvh.rs`), or first
  lowering memory enough that a builder run is not starved.
