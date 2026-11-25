//! Host work recorded into a title's command list in place of a console
//! library's (the title's upscaler, see `victory_hle::modules::fidelityfx`):
//! the title's FidelityFX upscaler runs as console compute programs that do
//! not translate (no `DXIL` part, see `docs/updates/fsr4-black-output.md`),
//! so a title list that asks for an upscaling gets [`upscale`] instead.
//!
//! The shape follows upstream's `external` module (0b35b7f, which hands the
//! title's textures to AMD's PC library); this one records a pass of ours.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;

use crate::object::{Object, This};

/// Host states the pass leaves its textures in: colour readable by compute,
/// output unordered access.
const COLOUR_STATE: u32 = 0x40;
const OUTPUT_STATE: u32 = 0x8;
/// Descriptor pairs in the pass's heap: one pair per upscaling, reused in
/// turn (far more than the frames the host has in flight).
const PAIRS: u32 = 64;

/// The upscaling pass: bilinear filtering of the colour's render-size
/// corner onto the output's upscale-size corner, written as unordered
/// access. Spatial only: no history, no use of depth or motion.
const SOURCE: &str = r#"
#define ROOT "RootConstants(num32BitConstants=8, b0), DescriptorTable(SRV(t0)), DescriptorTable(UAV(u0)), StaticSampler(s0, filter=FILTER_MIN_MAG_MIP_LINEAR, addressU=TEXTURE_ADDRESS_CLAMP, addressV=TEXTURE_ADDRESS_CLAMP)"
cbuffer sizes : register(b0) {
    uint2 render;  // texels of the colour the title rendered
    uint2 output;  // texels of the output to fill
    float2 scale;  // render / colour texture size
    uint2 pad;
};
Texture2D<float4> colour : register(t0);
RWTexture2D<float4> result : register(u0);
SamplerState linear_clamp : register(s0);
[RootSignature(ROOT)]
[numthreads(8, 8, 1)]
void upscale(uint3 id : SV_DispatchThreadID) {
    if (any(id.xy >= output)) {
        return;
    }
    float2 at = (float2(id.xy) + 0.5) / float2(output);
    // Clamp to the rendered corner's centre texels: never past it.
    float2 half_texel = 0.5 / float2(render);
    at = clamp(at, half_texel, 1.0 - half_texel);
    result[id.xy] = colour.SampleLevel(linear_clamp, at * scale, 0);
}
"#;

struct Pass {
    root: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    views: ID3D12DescriptorHeap,
    cpu: usize,
    gpu: u64,
    step: u32,
    next: AtomicU32,
}

// SAFETY: D3D12 objects are free-threaded.
unsafe impl Send for Pass {}
// SAFETY: as above.
unsafe impl Sync for Pass {}

fn pass() -> Result<&'static Pass, String> {
    static PASS: OnceLock<Result<Pass, String>> = OnceLock::new();
    PASS.get_or_init(|| {
        let host = crate::host::host().ok_or("no host GPU")?;
        let code = crate::host::compile(
            SOURCE,
            windows::core::s!("upscale"),
            windows::core::s!("cs_5_1"),
        )
        .map_err(|e| format!("the upscaling pass did not compile ({e})"))?;
        let kind = D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV;
        // SAFETY: the root signature is inside the shader; the description
        // points at live bytes and never releases its root pointer.
        unsafe {
            let root: ID3D12RootSignature = host
                .device
                .CreateRootSignature(0, &code[..])
                .map_err(|e| e.to_string())?;
            let pipeline: ID3D12PipelineState = host
                .device
                .CreateComputePipelineState(&D3D12_COMPUTE_PIPELINE_STATE_DESC {
                    pRootSignature: std::mem::transmute_copy(&root),
                    CS: D3D12_SHADER_BYTECODE {
                        pShaderBytecode: code.as_ptr().cast(),
                        BytecodeLength: code.len(),
                    },
                    ..Default::default()
                })
                .map_err(|e| e.to_string())?;
            let views: ID3D12DescriptorHeap = host
                .device
                .CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: kind,
                    NumDescriptors: PAIRS * 2,
                    Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                    NodeMask: 0,
                })
                .map_err(|e| e.to_string())?;
            Ok(Pass {
                cpu: views.GetCPUDescriptorHandleForHeapStart().ptr,
                gpu: views.GetGPUDescriptorHandleForHeapStart().ptr,
                step: host.device.GetDescriptorHandleIncrementSize(kind),
                root,
                pipeline,
                views,
                next: AtomicU32::new(0),
            })
        }
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// A host texture's description, for the log: format, size, flags.
pub fn describe(resource: *const Object) -> Option<String> {
    let texture = crate::resource::host_resource(resource as This)?;
    // SAFETY: a live texture.
    let d = unsafe { texture.GetDesc() };
    Some(format!(
        "{}x{} f{} flags {:#x}",
        d.Width, d.Height, d.Format.0, d.Flags.0
    ))
}

/// Whether the host can sample `format` and store it as typed unordered
/// access (the pass's two views).
fn supports(device: &ID3D12Device, format: DXGI_FORMAT, store: bool) -> bool {
    let mut support = D3D12_FEATURE_DATA_FORMAT_SUPPORT {
        Format: format,
        ..Default::default()
    };
    // SAFETY: a plain query into our own structure.
    let asked = unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_FORMAT_SUPPORT,
            (&raw mut support).cast(),
            size_of_val(&support) as u32,
        )
    };
    if asked.is_err() {
        return false;
    }
    if store {
        support.Support1.0 & D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW.0 != 0
            && support.Support2.0 & D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE.0 != 0
    } else {
        support.Support1.0 & D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE.0 != 0
    }
}

/// Records into the title list `list` an upscaling of the title texture
/// `colour` (its `render` corner) into the title texture `output` (its
/// `size` corner): bilinear filtering, written as unordered access. The
/// textures are left in compute-read and unordered-access states, which
/// the list tracks, so the title's next barriers start from there. An
/// error says why nothing was recorded.
pub fn upscale(
    list: *const Object,
    colour: *const Object,
    output: *const Object,
    render: [u32; 2],
    size: [u32; 2],
) -> Result<(), String> {
    let host = crate::host::host().ok_or("no host GPU")?;
    let pass = pass()?;
    let (colour, output) = (colour as This, output as This);
    let source = crate::resource::host_resource(colour)
        .ok_or("the colour has no host texture")?
        .clone();
    let target = crate::resource::host_resource(output)
        .ok_or("the output has no host texture")?
        .clone();
    // SAFETY: live textures.
    let (s, t) = unsafe { (source.GetDesc(), target.GetDesc()) };
    if t.Flags.0 & D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS.0 == 0 {
        return Err(format!("the output (f{}) allows no unordered access", t.Format.0));
    }
    if !supports(&host.device, s.Format, false) {
        return Err(format!("the colour's format f{} cannot be sampled", s.Format.0));
    }
    if !supports(&host.device, t.Format, true) {
        return Err(format!("the output's format f{} cannot be stored", t.Format.0));
    }
    let render = [render[0].clamp(1, s.Width as u32), render[1].clamp(1, s.Height)];
    let size = [size[0].clamp(1, t.Width as u32), size[1].clamp(1, t.Height)];
    let slot = pass.next.fetch_add(1, Ordering::Relaxed) % PAIRS * 2;
    let cpu = |i: u32| D3D12_CPU_DESCRIPTOR_HANDLE {
        ptr: pass.cpu + ((slot + i) * pass.step) as usize,
    };
    let gpu = |i: u32| D3D12_GPU_DESCRIPTOR_HANDLE {
        ptr: pass.gpu + u64::from((slot + i) * pass.step),
    };
    let view = D3D12_SHADER_RESOURCE_VIEW_DESC {
        Format: s.Format,
        ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
        Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_SRV {
                MipLevels: 1,
                ..Default::default()
            },
        },
    };
    let store = D3D12_UNORDERED_ACCESS_VIEW_DESC {
        Format: t.Format,
        ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2D,
        Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_UAV::default(),
        },
    };
    let constants: [u32; 8] = [
        render[0],
        render[1],
        size[0],
        size[1],
        (render[0] as f32 / s.Width as f32).to_bits(),
        (render[1] as f32 / s.Height as f32).to_bits(),
        0,
        0,
    ];
    let mut recorded = false;
    crate::list::with_state(list as This, |hl, state| {
        crate::list::ensure_state(hl, colour, COLOUR_STATE);
        crate::list::ensure_state(hl, output, OUTPUT_STATE);
        let l = &hl.list;
        // SAFETY: an open list; slots of our heap; live textures, which the
        // list holds until it is reset.
        unsafe {
            host.device.CreateShaderResourceView(&source, Some(&view), cpu(0));
            host.device
                .CreateUnorderedAccessView(&target, None, Some(&store), cpu(1));
            l.SetDescriptorHeaps(&[Some(pass.views.clone())]);
            l.SetComputeRootSignature(&pass.root);
            l.SetPipelineState(&pass.pipeline);
            l.SetComputeRoot32BitConstants(0, 8, constants.as_ptr().cast(), 0);
            l.SetComputeRootDescriptorTable(1, gpu(0));
            l.SetComputeRootDescriptorTable(2, gpu(1));
            l.Dispatch(size[0].div_ceil(8), size[1].div_ceil(8), 1);
            l.ResourceBarrier(&[D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    UAV: std::mem::ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER {
                        pResource: std::mem::transmute_copy(&target),
                    }),
                },
            }]);
        }
        hl.keep(&source);
        hl.keep(&target);
        state.host_state_lost();
        recorded = true;
    });
    if !recorded {
        return Err("no host list".into());
    }
    crate::texture::host_written(output);
    UPSCALINGS.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

/// Upscalings recorded so far ([`upscale`]).
pub static UPSCALINGS: AtomicU32 = AtomicU32::new(0);

#[cfg(test)]
mod tests {
    use super::*;

    /// The pass compiles with the system's compiler and its root signature
    /// is the one [`upscale`] binds: constants, then two tables.
    #[test]
    fn the_upscaling_pass_compiles() {
        let code = crate::host::compile(
            SOURCE,
            windows::core::s!("upscale"),
            windows::core::s!("cs_5_1"),
        )
        .expect("compiles");
        assert!(code.starts_with(b"DXBC"));
        assert!(code.windows(4).any(|w| w == b"RTS0"), "carries its root signature");
    }
}
