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
