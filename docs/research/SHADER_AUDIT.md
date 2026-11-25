# Shader audit

Updated 2026-11-25.

## Containers the translator reads

DXBC containers whose `DXIL` part holds an `XDAT` index with an `XBCH`
(zlib) blob, which unpacks to `FSSM` v0x101 records per hardware stage
(`crates/victory-shader/src/native.rs`). Vertex, pixel, compute and mesh
programs are translated to host DXIL.

## Containers it does not read

| Part | Who | Count | Contents | Status |
|---|---|---|---|---|
| `ACCC` (`FSPX` v1) | the title's FSR4 upscaler (NewBE fallback, after `DeserializeComputePipelineStateX`, device slot 0x218, returns E_NOTIMPL) | 20 compute programs, 28 passes | section 0: register settings (two dwords like `COMPUTE_PGM_RSRC1/2`); section 1: 20 bytes; section 6: plain GFX10 code; section 7: resource-mapping records | not translated. The passes are avoided by `VICTORY_FFX_BUILTIN=1`, which answers the upscaler API instead |

Hashes: see `docs/updates/fsr4-black-output.md`.

Other forks: Project Flamingo `f219769` (2026-11-25) parses FSPX modules
for thread group, wave size and code size, and reports them as
unsupported. It does not translate them either.

## Other known failures (from the logs)

- 1 mesh pipeline refused by the driver ("The parameter is incorrect"),
  `mesh f5e50eb2108c425b`, `pixel ce3f0432a08f1d35` (0x80070057). Not
  investigated yet.
- `pipeline state` slot 0x58 and `command list` slot 0x1a8
  (`ClearUnorderedAccessViewUint`) are not implemented. Both were called
  by the FSR4 DLL, which no longer loads with the switch. Upstream
  `0b35b7f` implements 0x1a8/0x1b0.
