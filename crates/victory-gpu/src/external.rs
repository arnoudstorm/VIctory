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

/// The upscaling pass: the colour's render-size corner onto the output's
/// upscale-size corner, written as unordered access. Spatial only: no
/// history, no use of depth or motion. Two filters (`mode`): bilinear, or
/// (`VICTORY_FFX_FILTER=bicubic`) Catmull-Rom bicubic from nine bilinear
/// taps, sharpened against the bilinear result by `sharpen`
/// (`VICTORY_FFX_SHARPEN`, 0.25) and kept inside the four nearest source
/// texels, so the HDR colour neither rings nor goes negative.
const SOURCE: &str = r#"
#define ROOT "RootConstants(num32BitConstants=8, b0), DescriptorTable(SRV(t0)), DescriptorTable(UAV(u0)), StaticSampler(s0, filter=FILTER_MIN_MAG_MIP_LINEAR, addressU=TEXTURE_ADDRESS_CLAMP, addressV=TEXTURE_ADDRESS_CLAMP)"
cbuffer sizes : register(b0) {
    uint2 render;   // texels of the colour the title rendered
    uint2 output;   // texels of the output to fill
    float2 scale;   // render / colour texture size
    uint mode;      // 0 bilinear, 1 bicubic and sharpened
    float sharpen;  // how much (bicubic only)
};
Texture2D<float4> colour : register(t0);
RWTexture2D<float4> result : register(u0);
SamplerState linear_clamp : register(s0);
float4 tap(float2 texel, float2 size) {
    return colour.SampleLevel(linear_clamp, texel / size, 0);
}
// Catmull-Rom through nine bilinear taps; `p` in texels of the colour texture.
float4 bicubic(float2 p, float2 size) {
    float2 t1 = floor(p - 0.5) + 0.5;
    float2 f = p - t1;
    float2 w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
    float2 w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
    float2 w2 = f * (0.5 + f * (2.0 - 1.5 * f));
    float2 w3 = f * f * (-0.5 + 0.5 * f);
    float2 w12 = w1 + w2;
    float2 t0 = t1 - 1.0;
    float2 t3 = t1 + 2.0;
    float2 t12 = t1 + w2 / w12;
    float4 c = 0;
    c += tap(float2(t0.x, t0.y), size) * w0.x * w0.y;
    c += tap(float2(t12.x, t0.y), size) * w12.x * w0.y;
    c += tap(float2(t3.x, t0.y), size) * w3.x * w0.y;
    c += tap(float2(t0.x, t12.y), size) * w0.x * w12.y;
    c += tap(float2(t12.x, t12.y), size) * w12.x * w12.y;
    c += tap(float2(t3.x, t12.y), size) * w3.x * w12.y;
    c += tap(float2(t0.x, t3.y), size) * w0.x * w3.y;
    c += tap(float2(t12.x, t3.y), size) * w12.x * w3.y;
    c += tap(float2(t3.x, t3.y), size) * w3.x * w3.y;
    return c;
}
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
    float4 smooth = colour.SampleLevel(linear_clamp, at * scale, 0);
    if (mode == 0) {
        result[id.xy] = smooth;
        return;
    }
    float2 size = float2(render) / scale;
    float2 p = at * float2(render);
    float4 c = bicubic(p, size);
    c += (c - smooth) * sharpen;
    // Inside the four nearest source texels.
    int2 q = int2(floor(p - 0.5));
    int2 last = int2(render) - 1;
    float4 a = colour.Load(int3(clamp(q, int2(0, 0), last), 0));
    float4 b = colour.Load(int3(clamp(q + int2(1, 0), int2(0, 0), last), 0));
    float4 d = colour.Load(int3(clamp(q + int2(0, 1), int2(0, 0), last), 0));
    float4 e = colour.Load(int3(clamp(q + int2(1, 1), int2(0, 0), last), 0));
    float4 lo = min(min(a, b), min(d, e));
    float4 hi = max(max(a, b), max(d, e));
    result[id.xy] = max(clamp(c, lo, hi), 0.0);
}
"#;

/// The temporal pass (`VICTORY_FFX_TEMPORAL`): the spatial result above is
/// blended with the previous output, reprojected by the title's motion
/// vectors (FidelityFX convention: render texels times the dispatch's
/// motion-vector scale, pointing from this frame to the previous one) and
/// clipped to the current colour's neighbourhood (mean and spread in
/// YCoCg), so dithered and stochastic effects the title leaves for its
/// temporal upscaler to resolve (hair, cloth, screen probes) settle
/// instead of staying grainy. Blending is in a tone-mapped space so bright
/// texels do not dominate, and non-finite values never enter the history.
const TEMPORAL: &str = r#"
#define ROOT "RootConstants(num32BitConstants=12, b0), DescriptorTable(SRV(t0, numDescriptors=2)), DescriptorTable(UAV(u0, numDescriptors=3)), StaticSampler(s0, filter=FILTER_MIN_MAG_MIP_LINEAR, addressU=TEXTURE_ADDRESS_CLAMP, addressV=TEXTURE_ADDRESS_CLAMP)"
cbuffer sizes : register(b0) {
    uint2 render;
    uint2 output;
    float2 scale;     // render / colour texture size
    uint mode;        // 0 bilinear, 1 bicubic and sharpened
    float sharpen;
    float2 motion_scale;
    float blend;      // weight of the current frame, 1 = no history
    uint history_ok;  // 0 on reset or a new history
};
Texture2D<float4> colour : register(t0);
Texture2D<float2> motion : register(t1);
RWTexture2D<float4> result : register(u0);
RWTexture2D<float4> history_in : register(u1);
RWTexture2D<float4> history_out : register(u2);
SamplerState linear_clamp : register(s0);
float4 finite(float4 c) {
    return (any(isnan(c)) || any(isinf(c))) ? float4(0, 0, 0, 0) : c;
}
float4 tap(float2 texel, float2 size) {
    return colour.SampleLevel(linear_clamp, texel / size, 0);
}
float4 bicubic(float2 p, float2 size) {
    float2 t1 = floor(p - 0.5) + 0.5;
    float2 f = p - t1;
    float2 w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
    float2 w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
    float2 w2 = f * (0.5 + f * (2.0 - 1.5 * f));
    float2 w3 = f * f * (-0.5 + 0.5 * f);
    float2 w12 = w1 + w2;
    float2 t0 = t1 - 1.0;
    float2 t3 = t1 + 2.0;
    float2 t12 = t1 + w2 / w12;
    float4 c = 0;
    c += tap(float2(t0.x, t0.y), size) * w0.x * w0.y;
    c += tap(float2(t12.x, t0.y), size) * w12.x * w0.y;
    c += tap(float2(t3.x, t0.y), size) * w3.x * w0.y;
    c += tap(float2(t0.x, t12.y), size) * w0.x * w12.y;
    c += tap(float2(t12.x, t12.y), size) * w12.x * w12.y;
    c += tap(float2(t3.x, t12.y), size) * w3.x * w12.y;
    c += tap(float2(t0.x, t3.y), size) * w0.x * w3.y;
    c += tap(float2(t12.x, t3.y), size) * w12.x * w3.y;
    c += tap(float2(t3.x, t3.y), size) * w3.x * w3.y;
    return c;
}
float3 to_ycocg(float3 c) {
    return float3(0.25 * c.r + 0.5 * c.g + 0.25 * c.b, 0.5 * c.r - 0.5 * c.b, -0.25 * c.r + 0.5 * c.g - 0.25 * c.b);
}
float3 from_ycocg(float3 c) {
    return float3(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}
// Tone-mapped (bounded) colour and back, for blending.
float3 squash(float3 c) { return c / (1.0 + max(c.r, max(c.g, c.b))); }
float3 unsquash(float3 c) { return c / max(1.0 - max(c.r, max(c.g, c.b)), 1e-4); }
float4 history_at(float2 p) {
    // Bilinear from history, `p` in output texels.
    float2 q = p - 0.5;
    int2 i = int2(floor(q));
    float2 f = q - float2(i);
    int2 last = int2(output) - 1;
    float4 a = finite(history_in[clamp(i, int2(0, 0), last)]);
    float4 b = finite(history_in[clamp(i + int2(1, 0), int2(0, 0), last)]);
    float4 c = finite(history_in[clamp(i + int2(0, 1), int2(0, 0), last)]);
    float4 d = finite(history_in[clamp(i + int2(1, 1), int2(0, 0), last)]);
    return lerp(lerp(a, b, f.x), lerp(c, d, f.x), f.y);
}
[RootSignature(ROOT)]
[numthreads(8, 8, 1)]
void temporal(uint3 id : SV_DispatchThreadID) {
    if (any(id.xy >= output)) {
        return;
    }
    float2 at = (float2(id.xy) + 0.5) / float2(output);
    float2 half_texel = 0.5 / float2(render);
    at = clamp(at, half_texel, 1.0 - half_texel);
    float4 smooth = finite(colour.SampleLevel(linear_clamp, at * scale, 0));
    float2 p = at * float2(render);
    float4 current = smooth;
    if (mode != 0) {
        float2 size = float2(render) / scale;
        current = finite(bicubic(p, size));
        current += (current - smooth) * sharpen;
    }
    // The neighbourhood of the nearest source texel: 3x3, in YCoCg.
    int2 centre = int2(floor(p));
    int2 last = int2(render) - 1;
    float3 m1 = 0, m2 = 0, lo = 1e30, hi = -1e30;
    [unroll] for (int y = -1; y <= 1; y++) {
        [unroll] for (int x = -1; x <= 1; x++) {
            float3 s = to_ycocg(squash(max(finite(colour.Load(int3(clamp(centre + int2(x, y), int2(0, 0), last), 0))).rgb, 0.0)));
            m1 += s;
            m2 += s * s;
            lo = min(lo, s);
            hi = max(hi, s);
        }
    }
    m1 /= 9.0;
    float3 sigma = sqrt(max(m2 / 9.0 - m1 * m1, 0.0));
    float3 box_lo = max(lo, m1 - 1.25 * sigma);
    float3 box_hi = min(hi, m1 + 1.25 * sigma);
    float3 now = to_ycocg(squash(max(current.rgb, 0.0)));
    // The spatial result kept inside the neighbourhood too (no ringing).
    now = clamp(now, lo, hi);
    float3 mixed = now;
    float2 mv = motion.Load(int3(clamp(centre, int2(0, 0), last), 0)) * motion_scale / float2(render);
    float2 previous = at + mv;
    bool inside = all(previous > 0.0) && all(previous < 1.0) && !any(isnan(previous));
    if (history_ok != 0 && inside) {
        float3 old = to_ycocg(squash(max(history_at(previous * float2(output)).rgb, 0.0)));
        old = clamp(old, box_lo, box_hi);
        mixed = lerp(old, now, blend);
    }
    float4 c = finite(float4(max(unsquash(from_ycocg(mixed)), 0.0), current.a));
    result[id.xy] = c;
    history_out[id.xy] = c;
}
"#;

struct Pass {
    root: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    views: ID3D12DescriptorHeap,
    cpu: usize,
    gpu: u64,
    step: u32,
    /// Descriptors one use of the pass takes.
    group: u32,
    next: AtomicU32,
}

// SAFETY: D3D12 objects are free-threaded.
unsafe impl Send for Pass {}
// SAFETY: as above.
unsafe impl Sync for Pass {}

fn pass() -> Result<&'static Pass, String> {
    static PASS: OnceLock<Result<Pass, String>> = OnceLock::new();
    PASS.get_or_init(|| make_pass(SOURCE, windows::core::s!("upscale"), 2))
        .as_ref()
        .map_err(Clone::clone)
}

fn temporal_pass() -> Result<&'static Pass, String> {
    static PASS: OnceLock<Result<Pass, String>> = OnceLock::new();
    PASS.get_or_init(|| make_pass(TEMPORAL, windows::core::s!("temporal"), 5))
        .as_ref()
        .map_err(Clone::clone)
}

/// A pass from `source`'s `entry`, with a heap of `PAIRS` groups of
/// `group` descriptors.
fn make_pass(source: &str, entry: windows::core::PCSTR, group: u32) -> Result<Pass, String> {
    {
        let host = crate::host::host().ok_or("no host GPU")?;
        let code = crate::host::compile(source, entry, windows::core::s!("cs_5_1"))
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
                    NumDescriptors: PAIRS * group,
                    Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                    NodeMask: 0,
                })
                .map_err(|e| e.to_string())?;
            Ok(Pass {
                cpu: views.GetCPUDescriptorHandleForHeapStart().ptr,
                gpu: views.GetGPUDescriptorHandleForHeapStart().ptr,
                step: host.device.GetDescriptorHandleIncrementSize(kind),
                group,
                root,
                pipeline,
                views,
                next: AtomicU32::new(0),
            })
        }
    }
}

/// The filter (`VICTORY_FFX_FILTER`: `bicubic`, else bilinear) and how
/// much it sharpens (`VICTORY_FFX_SHARPEN`, 0.25; bicubic only).
fn filter() -> (bool, f32) {
    static SET: OnceLock<(bool, f32)> = OnceLock::new();
    *SET.get_or_init(|| {
        let bicubic = std::env::var("VICTORY_FFX_FILTER").is_ok_and(|v| v.trim().eq_ignore_ascii_case("bicubic"));
        let sharpen = std::env::var("VICTORY_FFX_SHARPEN")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite() && (0.0..=2.0).contains(v))
            .unwrap_or(0.25);
        if bicubic {
            tracing::info!(sharpen, "upscaling (ours): bicubic, sharpened");
        }
        (bicubic, sharpen)
    })
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
    upscale_with(list, colour, output, render, size, None)
}

/// The title's motion vectors for a temporal upscaling: the texture, the
/// dispatch's motion-vector scale, and whether the title asked for its
/// history to be dropped (a camera cut).
pub struct Motion {
    pub texture: *const Object,
    pub scale: [f32; 2],
    pub reset: bool,
}

/// The temporal pass's weight of the current frame
/// (`VICTORY_FFX_TEMPORAL`: unset, `0` or `off` is off; `1` or `on` is
/// 0.1; a number between 0.02 and 1 is that weight).
pub fn temporal() -> Option<f32> {
    static SET: OnceLock<Option<f32>> = OnceLock::new();
    *SET.get_or_init(|| {
        let v = std::env::var("VICTORY_FFX_TEMPORAL").ok()?;
        let blend = match v.trim() {
            "" | "0" | "off" => return None,
            "1" | "on" => 0.1,
            v => v.parse::<f32>().ok().filter(|b| (0.02..=1.0).contains(b))?,
        };
        tracing::info!(blend, "upscaling (ours): temporal, with history");
        Some(blend)
    })
}

/// Two output-size textures the temporal pass reads and writes in turn,
/// both kept in unordered-access state.
struct History {
    size: [u32; 2],
    textures: [ID3D12Resource; 2],
    /// The one the next upscaling reads.
    current: usize,
    /// Whether `current` holds a frame.
    filled: bool,
}

// SAFETY: D3D12 objects are free-threaded.
unsafe impl Send for History {}

static HISTORY: std::sync::Mutex<Option<History>> = std::sync::Mutex::new(None);

fn history_texture(device: &ID3D12Device, size: [u32; 2]) -> Result<ID3D12Resource, String> {
    let mut texture: Option<ID3D12Resource> = None;
    // SAFETY: plain descriptions; the out pointer is ours.
    unsafe {
        device.CreateCommittedResource(
            &D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                ..Default::default()
            },
            D3D12_HEAP_FLAG_NONE,
            &D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Width: u64::from(size[0]),
                Height: size[1],
                DepthOrArraySize: 1,
                MipLevels: 1,
                Format: DXGI_FORMAT_R16G16B16A16_FLOAT,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
                Flags: D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
                Alignment: 0,
            },
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            None,
            &mut texture,
        )
    }
    .map_err(|e| format!("no history texture ({e})"))?;
    texture.ok_or_else(|| "no history texture".into())
}

/// As [`upscale`]; with `motion` and `VICTORY_FFX_TEMPORAL` set, the
/// temporal pass instead of the spatial one.
pub fn upscale_with(
    list: *const Object,
    colour: *const Object,
    output: *const Object,
    render: [u32; 2],
    size: [u32; 2],
    motion: Option<Motion>,
) -> Result<(), String> {
    if let (Some(blend), Some(m)) = (temporal(), motion.as_ref())
        && !m.texture.is_null()
    {
        match upscale_temporal(list, colour, output, render, size, m, blend) {
            Ok(()) => return Ok(()),
            Err(why) => {
                static SAID: AtomicU32 = AtomicU32::new(0);
                if SAID.fetch_add(1, Ordering::Relaxed) < 4 {
                    tracing::warn!("upscaling (ours): temporal pass not recorded, spatial instead: {why}");
                }
            }
        }
    }
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
        u32::from(filter().0),
        filter().1.to_bits(),
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

/// The temporal upscaling ([`TEMPORAL`]): as the spatial one, plus the
/// title's motion vectors and our history pair.
fn upscale_temporal(
    list: *const Object,
    colour: *const Object,
    output: *const Object,
    render: [u32; 2],
    size: [u32; 2],
    motion: &Motion,
    blend: f32,
) -> Result<(), String> {
    let host = crate::host::host().ok_or("no host GPU")?;
    let pass = temporal_pass()?;
    let (colour, output, vectors) = (colour as This, output as This, motion.texture as This);
    let source = crate::resource::host_resource(colour)
        .ok_or("the colour has no host texture")?
        .clone();
    let target = crate::resource::host_resource(output)
        .ok_or("the output has no host texture")?
        .clone();
    let moved = crate::resource::host_resource(vectors)
        .ok_or("the motion vectors have no host texture")?
        .clone();
    // SAFETY: live textures.
    let (s, t, m) = unsafe { (source.GetDesc(), target.GetDesc(), moved.GetDesc()) };
    if t.Flags.0 & D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS.0 == 0 {
        return Err(format!("the output (f{}) allows no unordered access", t.Format.0));
    }
    if !supports(&host.device, s.Format, false) || !supports(&host.device, m.Format, false) {
        return Err(format!("f{} or f{} cannot be sampled", s.Format.0, m.Format.0));
    }
    if !supports(&host.device, t.Format, true) {
        return Err(format!("the output's format f{} cannot be stored", t.Format.0));
    }
    let render = [render[0].clamp(1, s.Width as u32), render[1].clamp(1, s.Height)];
    let size = [size[0].clamp(1, t.Width as u32), size[1].clamp(1, t.Height)];
    // The pair, made (again) for this output size.
    let mut history = HISTORY.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if history.as_ref().is_none_or(|h| h.size != size) {
        *history = Some(History {
            size,
            textures: [history_texture(&host.device, size)?, history_texture(&host.device, size)?],
            current: 0,
            filled: false,
        });
    }
    let h = history.as_mut().expect("made above");
    let (read, write) = (h.textures[h.current].clone(), h.textures[1 - h.current].clone());
    let history_ok = h.filled && !motion.reset;
    let slot = pass.next.fetch_add(1, Ordering::Relaxed) % PAIRS * pass.group;
    let cpu = |i: u32| D3D12_CPU_DESCRIPTOR_HANDLE {
        ptr: pass.cpu + ((slot + i) * pass.step) as usize,
    };
    let gpu = |i: u32| D3D12_GPU_DESCRIPTOR_HANDLE {
        ptr: pass.gpu + u64::from((slot + i) * pass.step),
    };
    let srv = |format: DXGI_FORMAT| D3D12_SHADER_RESOURCE_VIEW_DESC {
        Format: format,
        ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
        Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_SRV {
                MipLevels: 1,
                ..Default::default()
            },
        },
    };
    let uav = |format: DXGI_FORMAT| D3D12_UNORDERED_ACCESS_VIEW_DESC {
        Format: format,
        ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2D,
        Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
            Texture2D: D3D12_TEX2D_UAV::default(),
        },
    };
    let constants: [u32; 12] = [
        render[0],
        render[1],
        size[0],
        size[1],
        (render[0] as f32 / s.Width as f32).to_bits(),
        (render[1] as f32 / s.Height as f32).to_bits(),
        u32::from(filter().0),
        filter().1.to_bits(),
        motion.scale[0].to_bits(),
        motion.scale[1].to_bits(),
        blend.to_bits(),
        u32::from(history_ok),
    ];
    let uav_barrier = |r: &ID3D12Resource| D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            // SAFETY: a borrowed pointer the barrier does not release.
            UAV: std::mem::ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER {
                pResource: unsafe { std::mem::transmute_copy(r) },
            }),
        },
    };
    let mut recorded = false;
    crate::list::with_state(list as This, |hl, state| {
        crate::list::ensure_state(hl, colour, COLOUR_STATE);
        crate::list::ensure_state(hl, vectors, COLOUR_STATE);
        crate::list::ensure_state(hl, output, OUTPUT_STATE);
        let l = &hl.list;
        // SAFETY: an open list; slots of our heap; live textures, which the
        // list holds until it is reset.
        unsafe {
            host.device.CreateShaderResourceView(&source, Some(&srv(s.Format)), cpu(0));
            host.device.CreateShaderResourceView(&moved, Some(&srv(m.Format)), cpu(1));
            host.device
                .CreateUnorderedAccessView(&target, None, Some(&uav(t.Format)), cpu(2));
            host.device.CreateUnorderedAccessView(
                &read,
                None,
                Some(&uav(DXGI_FORMAT_R16G16B16A16_FLOAT)),
                cpu(3),
            );
            host.device.CreateUnorderedAccessView(
                &write,
                None,
                Some(&uav(DXGI_FORMAT_R16G16B16A16_FLOAT)),
                cpu(4),
            );
            // The previous upscaling's writes to the history first.
            l.ResourceBarrier(&[uav_barrier(&read), uav_barrier(&write)]);
            l.SetDescriptorHeaps(&[Some(pass.views.clone())]);
            l.SetComputeRootSignature(&pass.root);
            l.SetPipelineState(&pass.pipeline);
            l.SetComputeRoot32BitConstants(0, 12, constants.as_ptr().cast(), 0);
            l.SetComputeRootDescriptorTable(1, gpu(0));
            l.SetComputeRootDescriptorTable(2, gpu(2));
            l.Dispatch(size[0].div_ceil(8), size[1].div_ceil(8), 1);
            l.ResourceBarrier(&[uav_barrier(&target), uav_barrier(&write)]);
        }
        hl.keep(&source);
        hl.keep(&moved);
        hl.keep(&target);
        hl.keep(&read);
        hl.keep(&write);
        state.host_state_lost();
        recorded = true;
    });
    if !recorded {
        return Err("no host list".into());
    }
    h.current = 1 - h.current;
    h.filled = true;
    drop(history);
    crate::texture::host_written(output);
    UPSCALINGS.fetch_add(1, Ordering::Relaxed);
    TEMPORAL_UPSCALINGS.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

/// Temporal upscalings recorded so far.
pub static TEMPORAL_UPSCALINGS: AtomicU32 = AtomicU32::new(0);

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

    /// The temporal pass compiles too, with its own root signature.
    #[test]
    fn the_temporal_pass_compiles() {
        let code = crate::host::compile(
            TEMPORAL,
            windows::core::s!("temporal"),
            windows::core::s!("cs_5_1"),
        )
        .expect("compiles");
        assert!(code.starts_with(b"DXBC"));
        assert!(code.windows(4).any(|w| w == b"RTS0"), "carries its root signature");
    }
}
