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
