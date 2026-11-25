//! `amd_fidelityfx_loader_xbox.dll`: AMD FidelityFX's API (the FidelityFX
//! SDK's, MIT), served by an upscaler of ours. Opt-in: only with
//! `VICTORY_FFX_BUILTIN=1` is this module in the registry; without it the
//! title's own console DLLs run as before.
//!
//! Why: the title's console upscaler (FSR4) builds its passes from console
//! machine code in `ACCC` parts, which do not translate (no `DXIL` part,
//! 20 compute programs), so its dispatches are dropped and the output it
//! should write stays black: the title's final 1920x1080 image is built
//! from it (`docs/updates/fsr4-black-output.md`). Here the title's
//! upscaling is answered by [`victory_gpu::external::upscale`], a spatial
//! (bilinear) upscaling of the colour into the output on the host: the
//! same picture at the output size, without the temporal reconstruction.
//!
//! The API is five functions over chains of descriptions, each starting
//! with a type id and the next one's address; contexts are opaque
//! pointers. Structure layouts follow upstream's module (0b35b7f), which
//! serves the same calls with AMD's PC build instead.
#![allow(non_snake_case)]
// Title code calls these through its import table.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_void};
use std::sync::atomic::{AtomicU32, Ordering};

use victory_gpu::object::Object;

use crate::{HleModule, hle_export};

/// The switch that puts this module in the registry.
pub const SWITCH: &str = "VICTORY_FFX_BUILTIN";

// Return codes.
const OK: u32 = 0;
const UNKNOWN_TYPE: u32 = 2;
const RUNTIME_ERROR: u32 = 3;
const NO_PROVIDER: u32 = 4;
const PARAMETER: u32 = 6;

// Description types.
const CREATE_UPSCALE: u64 = 0x1_0000;
const DISPATCH_UPSCALE: u64 = 0x1_0001;
const QUERY_RATIO: u64 = 0x1_0002;
const QUERY_RENDER_SIZE: u64 = 0x1_0003;
const QUERY_JITTER_COUNT: u64 = 0x1_0004;
const QUERY_JITTER_OFFSET: u64 = 0x1_0005;
const DISPATCH_REACTIVE_MASK: u64 = 0x1_0006;
const CONFIGURE_KEY_VALUE: u64 = 0x1_0007;
const QUERY_MEMORY: u64 = 0x1_0008;
const GET_VERSIONS: u64 = 4;
const GET_PROVIDER_VERSION: u64 = 6;
const CONFIGURE_DEBUG: u64 = 1;

/// The one provider we offer.
const VERSION_ID: u64 = 0x5649_4354_0001_0000;
const VERSION_NAME: &[u8] = b"VIctory spatial upscaler 1.0\0";

/// Whether the switch is on (`1`; anything else or unset is off).
pub fn enabled() -> bool {
    std::env::var(SWITCH).is_ok_and(|v| v.trim() == "1")
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Header {
    kind: u64,
    next: *mut Header,
}

/// An upscaler context: flags, largest render and output sizes, a message
/// callback.
#[repr(C)]
#[derive(Clone, Copy)]
struct CreateUpscale {
    header: Header,
    flags: u32,
    max_render: [u32; 2],
    max_output: [u32; 2],
    message: usize,
}

/// A resource, its description (type, format, width, height, depth, mips,
/// flags, usage) and the state it is in.
#[repr(C)]
#[derive(Clone, Copy)]
struct Resource {
    resource: *mut c_void,
    description: [u32; 8],
    state: u32,
}

/// One upscaling: the command list, then colour, depth, motion vectors,
/// exposure, reactive mask, transparency mask and output; then jitter (2
/// floats), motion-vector scale (2), render size (2), upscale size (2),
/// sharpening, sharpness, frame time, pre-exposure, reset, camera near,
/// far and field of view, metres per unit, flags.
#[repr(C)]
#[derive(Clone, Copy)]
struct DispatchUpscale {
    header: Header,
    list: *mut c_void,
    resources: [Resource; 7],
    rest: [u32; 18],
}

const COLOUR: usize = 0;
const OUTPUT: usize = 6;
const NAMES: [&str; 7] = [
    "colour",
    "depth",
    "motion",
    "exposure",
    "reactive",
    "transparency",
    "output",
];

const _: () = assert!(size_of::<CreateUpscale>() == 0x30);
const _: () = assert!(size_of::<Resource>() == 0x30);
const _: () = assert!(size_of::<DispatchUpscale>() == 0x1b0);

/// The versions a backend offers: count in and out, ids, names.
#[repr(C)]
struct GetVersions {
    header: Header,
    create_kind: u64,
    device: *mut c_void,
    count: *mut u64,
    ids: *mut u64,
    names: *mut *const c_char,
}

#[repr(C)]
struct ProviderVersion {
    header: Header,
    id: u64,
    name: *const c_char,
}

#[repr(C)]
struct QueryRatio {
    header: Header,
    quality: u32,
    ratio: *mut f32,
}

#[repr(C)]
struct QueryRenderSize {
    header: Header,
    display: [u32; 2],
    quality: u32,
    width: *mut u32,
    height: *mut u32,
}

#[repr(C)]
struct QueryJitterCount {
    header: Header,
    render_width: u32,
    display_width: u32,
    count: *mut i32,
}

#[repr(C)]
struct QueryJitterOffset {
    header: Header,
    index: i32,
    count: i32,
    x: *mut f32,
    y: *mut f32,
}

#[repr(C)]
struct QueryMemory {
    header: Header,
    usage: *mut [u64; 2],
}

/// Our context: the sizes the title asked for.
struct Context {
    max_render: [u32; 2],
    max_output: [u32; 2],
}

/// The FSR quality modes' upscaling ratios: native, quality, balanced,
/// performance, ultra performance.
fn ratio(quality: u32) -> f32 {
    match quality {
        0 => 1.0,
        1 => 1.5,
        2 => 1.7,
        3 => 2.0,
        4 => 3.0,
        _ => 1.0,
    }
}

macro_rules! once {
    ($level:ident, $($arg:tt)*) => {{
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| tracing::$level!($($arg)*));
    }};
}

/// Logs a description type we do not answer, the first few times.
fn unknown(call: &str, kind: u64) {
    static SAID: AtomicU32 = AtomicU32::new(0);
    if SAID.fetch_add(1, Ordering::Relaxed) < 16 {
        tracing::warn!(call, kind = format_args!("{kind:#x}"), "FidelityFX (ours): a description type not answered");
    }
}

extern "C" fn ffxCreateContext(
    context: *mut *mut c_void,
    desc: *mut Header,
    _memory: *const c_void,
) -> u32 {
    if context.is_null() || desc.is_null() {
        return PARAMETER;
    }
    // SAFETY: the title passes a chain of descriptions.
    let kind = unsafe { (*desc).kind };
    if kind != CREATE_UPSCALE {
        tracing::warn!(
            kind = format_args!("{kind:#x}"),
            "FidelityFX (ours): only the upscaler is offered; another effect was refused"
        );
        return NO_PROVIDER;
    }
    // SAFETY: an upscaler description, by its type.
    let upscale = unsafe { desc.cast::<CreateUpscale>().read() };
    let ours = Box::new(Context {
        max_render: upscale.max_render,
        max_output: upscale.max_output,
    });
    tracing::info!(
        render = ?upscale.max_render,
        output = ?upscale.max_output,
        flags = format_args!("{:#x}", upscale.flags),
        "FidelityFX (ours, VICTORY_FFX_BUILTIN): upscaler context made; upscalings are spatial (bilinear) on the host"
    );
    // SAFETY: the title's out pointer.
    unsafe { *context = Box::into_raw(ours).cast() };
    OK
}

extern "C" fn ffxDestroyContext(context: *mut *mut c_void, _memory: *const c_void) -> u32 {
    if context.is_null() {
        return PARAMETER;
    }
    // SAFETY: the title passes the context we made, once.
    unsafe {
        let ours = (*context).cast::<Context>();
        if !ours.is_null() {
            drop(Box::from_raw(ours));
        }
        *context = std::ptr::null_mut();
    }
    OK
}

extern "C" fn ffxConfigure(_context: *mut *mut c_void, desc: *mut Header) -> u32 {
    if desc.is_null() {
        return PARAMETER;
    }
    // SAFETY: the title passes a description.
    let kind = unsafe { (*desc).kind };
    match kind {
        // Settings we have no use for.
        CONFIGURE_KEY_VALUE | CONFIGURE_DEBUG => OK,
        _ => {
            unknown("configure", kind);
            OK
        }
    }
}

extern "C" fn ffxQuery(context: *mut *mut c_void, desc: *mut Header) -> u32 {
    if desc.is_null() {
        return PARAMETER;
    }
    // SAFETY: the title passes a description of the type its header names;
    // out pointers are the title's (null ones are skipped).
    unsafe {
        match (*desc).kind {
            GET_VERSIONS => {
                let q = &mut *desc.cast::<GetVersions>();
                if let Some(count) = q.count.as_mut() {
                    if !q.ids.is_null() && *count >= 1 {
                        *q.ids = VERSION_ID;
                    }
                    if !q.names.is_null() && *count >= 1 {
                        *q.names = VERSION_NAME.as_ptr().cast();
                    }
                    *count = 1;
                }
                OK
            }
            GET_PROVIDER_VERSION => {
                let q = &mut *desc.cast::<ProviderVersion>();
                q.id = VERSION_ID;
                q.name = VERSION_NAME.as_ptr().cast();
                OK
            }
            QUERY_RATIO => {
                let q = &*desc.cast::<QueryRatio>();
                if let Some(r) = q.ratio.as_mut() {
                    *r = ratio(q.quality);
                }
                OK
            }
            QUERY_RENDER_SIZE => {
                let q = &*desc.cast::<QueryRenderSize>();
                let r = ratio(q.quality);
                if let Some(w) = q.width.as_mut() {
                    *w = (q.display[0] as f32 / r) as u32;
                }
                if let Some(h) = q.height.as_mut() {
                    *h = (q.display[1] as f32 / r) as u32;
                }
                OK
            }
            // No history: the camera need not move between frames.
            QUERY_JITTER_COUNT => {
                let q = &*desc.cast::<QueryJitterCount>();
                if let Some(c) = q.count.as_mut() {
                    *c = 1;
                }
                OK
            }
            QUERY_JITTER_OFFSET => {
                let q = &*desc.cast::<QueryJitterOffset>();
                if let Some(x) = q.x.as_mut() {
                    *x = 0.0;
                }
                if let Some(y) = q.y.as_mut() {
                    *y = 0.0;
                }
                OK
            }
            QUERY_MEMORY => {
                let q = &*desc.cast::<QueryMemory>();
                if let Some(u) = q.usage.as_mut() {
                    *u = [0, 0];
                }
                OK
            }
            kind => {
                let _ = context;
                unknown("query", kind);
                UNKNOWN_TYPE
            }
        }
    }
}

extern "C" fn ffxDispatch(context: *mut *mut c_void, desc: *mut Header) -> u32 {
    if desc.is_null() {
        return PARAMETER;
    }
    // SAFETY: the title passes a description.
    let kind = unsafe { (*desc).kind };
    match kind {
        DISPATCH_UPSCALE => {}
        // Nothing of ours reads a reactive mask.
        DISPATCH_REACTIVE_MASK => return OK,
        _ => {
            unknown("dispatch", kind);
            return UNKNOWN_TYPE;
        }
    }
    // SAFETY: an upscaling, by its type.
    let d = unsafe { desc.cast::<DispatchUpscale>().read() };
    let render = [d.rest[4], d.rest[5]];
    let size = [d.rest[6], d.rest[7]];
    static DISPATCHES: AtomicU32 = AtomicU32::new(0);
    let n = DISPATCHES.fetch_add(1, Ordering::Relaxed) + 1;
    if n <= 3 || n.is_power_of_two() {
        let resources: Vec<String> = d
            .resources
            .iter()
            .zip(NAMES)
            .map(|(r, name)| {
                let host = victory_gpu::external::describe(r.resource.cast::<Object>())
                    .unwrap_or_else(|| "no host texture".into());
                format!(
                    "{name} {:p} ({}x{} format {} state {:#x}; host {host})",
                    r.resource, r.description[2], r.description[3], r.description[1], r.state
                )
            })
            .collect();
        // SAFETY: our context, if the title passes one.
        let sizes = unsafe { context.as_ref().and_then(|c| c.cast::<Context>().as_ref()) }
            .map(|c| (c.max_render, c.max_output));
        tracing::info!(
            dispatch = n,
            ?render,
            ?size,
            context = ?sizes,
            jitter = ?[f32::from_bits(d.rest[0]), f32::from_bits(d.rest[1])],
            reset = d.rest[12],
            "FidelityFX (ours): upscaling {}",
            resources.join(", ")
        );
    }
    let (colour, output) = (d.resources[COLOUR].resource, d.resources[OUTPUT].resource);
    if colour.is_null() || output.is_null() {
        once!(warn, "FidelityFX (ours): an upscaling without colour or output; nothing done");
        return PARAMETER;
    }
    match victory_gpu::external::upscale(
        d.list.cast::<Object>().cast_const(),
        colour.cast::<Object>().cast_const(),
        output.cast::<Object>().cast_const(),
        render,
        size,
    ) {
        Ok(()) => OK,
        Err(why) => {
            static SAID: AtomicU32 = AtomicU32::new(0);
            if SAID.fetch_add(1, Ordering::Relaxed) < 8 {
                tracing::warn!("FidelityFX (ours): the upscaling was not recorded: {why}");
            }
            RUNTIME_ERROR
        }
    }
}

pub(crate) fn module() -> HleModule {
    HleModule {
        name: "fidelityfx",
        dlls: &["amd_fidelityfx_loader_xbox.dll"],
        exports: vec![
            hle_export!(ffxConfigure),
            hle_export!(ffxCreateContext),
            hle_export!(ffxDestroyContext),
            hle_export!(ffxDispatch),
            hle_export!(ffxQuery),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr::null_mut;

    fn header(kind: u64) -> Header {
        Header {
            kind,
            next: null_mut(),
        }
    }

    #[test]
    fn a_context_is_made_and_destroyed() {
        let mut create = CreateUpscale {
            header: header(CREATE_UPSCALE),
            flags: 0,
            max_render: [1280, 720],
            max_output: [1920, 1080],
            message: 0,
        };
        let mut context = null_mut();
        assert_eq!(ffxCreateContext(&mut context, &mut create.header, std::ptr::null()), OK);
        assert!(!context.is_null());
        // SAFETY: the context just made.
        let c = unsafe { &*context.cast::<Context>() };
        assert_eq!((c.max_render, c.max_output), ([1280, 720], [1920, 1080]));
        assert_eq!(ffxDestroyContext(&mut context, std::ptr::null()), OK);
        assert!(context.is_null());
    }

    #[test]
    fn other_effects_are_refused() {
        let mut frame_generation = header(0x2_0000);
        let mut context = null_mut();
        assert_eq!(
            ffxCreateContext(&mut context, &mut frame_generation, std::ptr::null()),
            NO_PROVIDER
        );
        assert!(context.is_null());
    }

    #[test]
    fn one_version_is_offered() {
        let mut count = 0u64;
        let mut q = GetVersions {
            header: header(GET_VERSIONS),
            create_kind: CREATE_UPSCALE,
            device: null_mut(),
            count: &mut count,
            ids: null_mut(),
            names: null_mut(),
        };
        assert_eq!(ffxQuery(null_mut(), &mut q.header), OK);
        assert_eq!(count, 1);
        let mut id = 0u64;
        let mut name = std::ptr::null::<c_char>();
        q.ids = &mut id;
        q.names = &mut name;
        assert_eq!(ffxQuery(null_mut(), &mut q.header), OK);
        assert_eq!(id, VERSION_ID);
        // SAFETY: our NUL-terminated name.
        let text = unsafe { std::ffi::CStr::from_ptr(name) };
        assert!(text.to_str().unwrap().contains("VIctory"));
    }

    #[test]
    fn quality_modes_give_fsr_render_sizes_and_no_jitter() {
        let (mut w, mut h) = (0u32, 0u32);
        let mut q = QueryRenderSize {
            header: header(QUERY_RENDER_SIZE),
            display: [1920, 1080],
            quality: 1,
            width: &mut w,
            height: &mut h,
        };
        assert_eq!(ffxQuery(null_mut(), &mut q.header), OK);
        assert_eq!((w, h), (1280, 720));
        let mut count = 0i32;
        let mut j = QueryJitterCount {
            header: header(QUERY_JITTER_COUNT),
            render_width: 1280,
            display_width: 1920,
            count: &mut count,
        };
        assert_eq!(ffxQuery(null_mut(), &mut j.header), OK);
        assert_eq!(count, 1);
        let (mut x, mut y) = (1.0f32, 1.0f32);
        let mut o = QueryJitterOffset {
            header: header(QUERY_JITTER_OFFSET),
            index: 3,
            count: 1,
            x: &mut x,
            y: &mut y,
        };
        assert_eq!(ffxQuery(null_mut(), &mut o.header), OK);
        assert_eq!((x, y), (0.0, 0.0));
    }

    #[test]
    fn an_upscaling_without_textures_is_refused() {
        let empty = Resource {
            resource: null_mut(),
            description: [0; 8],
            state: 0,
        };
        let mut d = DispatchUpscale {
            header: header(DISPATCH_UPSCALE),
            list: null_mut(),
            resources: [empty; 7],
            rest: [0; 18],
        };
        assert_eq!(ffxDispatch(null_mut(), &mut d.header), PARAMETER);
        let mut unknown = header(0x7_0000);
        assert_eq!(ffxDispatch(null_mut(), &mut unknown), UNKNOWN_TYPE);
    }

    #[test]
    fn the_module_serves_the_loader_with_the_five_functions() {
        let m = module();
        assert!(m.serves("AMD_FidelityFX_Loader_Xbox.dll"));
        let mut names: Vec<_> = m.exports.iter().map(|e| e.name).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            ["ffxConfigure", "ffxCreateContext", "ffxDestroyContext", "ffxDispatch", "ffxQuery"]
        );
    }
}
