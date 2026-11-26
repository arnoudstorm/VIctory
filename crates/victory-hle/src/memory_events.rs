//! Opt-in observation at title import boundaries, never a global API hook.
//! Requests are not net committed bytes: overlapping commits are legal.
#![allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_void;
use std::sync::{OnceLock, atomic::{AtomicU64, Ordering::Relaxed}, mpsc::{SyncSender, sync_channel}};
use windows_sys::Win32::Foundation::{GetLastError, SetLastError};
use windows_sys::Win32::System::{Memory as win, Threading::GetCurrentThreadId};

#[derive(Clone, Copy)]
struct Event {
    api: &'static str, address: usize, bytes: usize, flags: u32, result: usize,
    error: u32, thread: u32, utc_us: u128, stack: [usize; 16], heap: usize, offset: u64,
}
struct Recorder { sender: SyncSender<Event>, stack_min: usize }
static RECORDER: OnceLock<Option<Recorder>> = OnceLock::new();
static DROPPED: AtomicU64 = AtomicU64::new(0);

fn recorder() -> Option<&'static Recorder> {
    RECORDER.get_or_init(|| {
        let path = std::env::var_os("VICTORY_MEMORY_EVENTS")?;
        let file = std::fs::File::create(path).ok()?;
        crate::memory_regions::start();
        let stack_min = std::env::var("VICTORY_MEMORY_STACK_MIN_MB").ok()
            .and_then(|s| s.parse::<usize>().ok()).unwrap_or(16).clamp(1, 4096) * 1024 * 1024;
        let (sender, receiver) = sync_channel::<Event>(8192);
        std::thread::Builder::new().name("title-memory-events".into()).spawn(move || {
            use std::io::Write;
            let mut file = std::io::BufWriter::with_capacity(128 * 1024, file);
            let began = std::time::Instant::now();
            loop {
                match receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(e) => {
                        if writeln!(file, "{{\"api\":\"{}\",\"utc_us\":{},\"writer_elapsed_us\":{},\"address\":{},\"requested_bytes\":{},\"flags\":{},\"result\":{},\"last_error\":{},\"thread\":{},\"heap\":{},\"section_offset\":{},\"stack\":{:?},\"dropped_total\":{}}}", e.api,e.utc_us,began.elapsed().as_micros(),e.address,e.bytes,e.flags,e.result,e.error,e.thread,e.heap,e.offset,e.stack,DROPPED.load(Relaxed)).is_err() { return }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if writeln!(file,"{{\"api\":\"tracer_status\",\"dropped_total\":{}}}",DROPPED.load(Relaxed)).is_err(){return}
                    }
                }
                // Keep the most recent event durable without blocking producers.
                if file.flush().is_err() { return }
            }
        }).ok()?;
        Some(Recorder { sender, stack_min })
    }).as_ref()
}

pub(crate) fn record(api: &'static str, address: usize, bytes: usize, flags: u32, result: usize, heap: usize) {
    record_at(api,address,bytes,flags,result,heap,0);
}

pub(crate) fn record_mapping(api: &'static str, address: usize, bytes: usize, section: usize, offset: u64) {
    record_at(api,address,bytes,0,address,section,offset);
}

fn record_at(api: &'static str, address: usize, bytes: usize, flags: u32, result: usize, heap: usize, offset: u64) {
    // Preserve the native call's last-error value even when tracing allocates.
    let error = unsafe { GetLastError() };
    if let Some(recorder) = recorder() {
        let mut stack = [0usize; 16];
        if bytes >= recorder.stack_min {
            // SAFETY: fixed stack storage; raw addresses, no symbol resolution.
            unsafe { windows_sys::Win32::System::Diagnostics::Debug::RtlCaptureStackBackTrace(2, 16, stack.as_mut_ptr().cast(), std::ptr::null_mut()); }
        }
        let event = Event { api,address,bytes,flags,result,error,heap,offset,stack,
            thread: unsafe { GetCurrentThreadId() },
            utc_us: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_micros() };
        if recorder.sender.try_send(event).is_err() { DROPPED.fetch_add(1, Relaxed); }
    }
    unsafe { SetLastError(error) };
}

pub(crate) fn wrap(dll: &str, symbol: &str) -> Option<usize> {
    let dll = dll.to_ascii_lowercase();
    if dll == "ntdll.dll" {
        recorder()?;
        return match symbol {
            "RtlAllocateHeap" => Some(rtl_alloc as *const () as usize),
            "RtlReAllocateHeap" => Some(rtl_realloc as *const () as usize),
            "RtlFreeHeap" => Some(rtl_free as *const () as usize),
            _ => None,
        };
    }
    if !(dll == "kernel32.dll" || dll == "kernelbase.dll" || dll.starts_with("api-ms-win-core-memory-") || dll.starts_with("api-ms-win-core-heap-")) { return None }
    // Initialized during import resolution, before any observed native call.
    recorder()?;
    Some(match symbol {
        "VirtualAlloc" => virtual_alloc as *const () as usize,
        "VirtualFree" => virtual_free as *const () as usize,
        "VirtualAllocEx" => virtual_alloc_ex as *const () as usize,
        "VirtualAlloc2" => virtual_alloc2 as *const () as usize,
        "VirtualFreeEx" => virtual_free_ex as *const () as usize,
        "HeapAlloc" => heap_alloc as *const () as usize,
        "HeapReAlloc" => heap_realloc as *const () as usize,
        "HeapFree" => heap_free as *const () as usize,
        _ => return None,
    })
}

extern "system" fn virtual_alloc2(process: *mut c_void, base: *const c_void, bytes: usize, flags: u32, protect: u32, params: *mut win::MEM_EXTENDED_PARAMETER, count: u32) -> *mut c_void {
    let result=unsafe {win::VirtualAlloc2(process,base,bytes,flags,protect,params,count)};
    record("VirtualAlloc2",base as usize,bytes,flags,result as usize,process as usize);result
}

extern "system" fn virtual_alloc_ex(process: *mut c_void, base: *const c_void, bytes: usize, flags: u32, protect: u32) -> *mut c_void {
    let result = unsafe { win::VirtualAllocEx(process,base,bytes,flags,protect) };
    record("VirtualAllocEx",base as usize,bytes,flags,result as usize,process as usize); result
}
extern "system" fn virtual_free_ex(process: *mut c_void, base: *mut c_void, bytes: usize, flags: u32) -> i32 {
    let result = unsafe { win::VirtualFreeEx(process,base,bytes,flags) };
    record("VirtualFreeEx",base as usize,bytes,flags,result as usize,process as usize); result
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlAllocateHeap(heap: *mut c_void, flags: u32, bytes: usize) -> *mut c_void;
    fn RtlReAllocateHeap(heap: *mut c_void, flags: u32, address: *mut c_void, bytes: usize) -> *mut c_void;
    fn RtlFreeHeap(heap: *mut c_void, flags: u32, address: *mut c_void) -> u8;
}
extern "system" fn rtl_alloc(heap: *mut c_void, flags: u32, bytes: usize) -> *mut c_void {
    let result = unsafe { RtlAllocateHeap(heap,flags,bytes) };
    record("RtlAllocateHeap",0,bytes,flags,result as usize,heap as usize); result
}
extern "system" fn rtl_realloc(heap: *mut c_void, flags: u32, address: *mut c_void, bytes: usize) -> *mut c_void {
    let result = unsafe { RtlReAllocateHeap(heap,flags,address,bytes) };
    record("RtlReAllocateHeap",address as usize,bytes,flags,result as usize,heap as usize); result
}
extern "system" fn rtl_free(heap: *mut c_void, flags: u32, address: *mut c_void) -> u8 {
    let result = unsafe { RtlFreeHeap(heap,flags,address) };
    record("RtlFreeHeap",address as usize,0,flags,result as usize,heap as usize); result
}

extern "system" fn virtual_alloc(base: *const c_void, bytes: usize, flags: u32, protect: u32) -> *mut c_void {
    let result = unsafe { win::VirtualAlloc(base, bytes, flags, protect) };
    record("VirtualAlloc",base as usize,bytes,flags,result as usize,0); result
}
extern "system" fn virtual_free(base: *mut c_void, bytes: usize, flags: u32) -> i32 {
    let result = unsafe { win::VirtualFree(base,bytes,flags) };
    record("VirtualFree",base as usize,bytes,flags,result as usize,0); result
}
extern "system" fn heap_alloc(heap: *mut c_void, flags: u32, bytes: usize) -> *mut c_void {
    let result = unsafe { win::HeapAlloc(heap,flags,bytes) };
    record("HeapAlloc",0,bytes,flags,result as usize,heap as usize); result
}
extern "system" fn heap_realloc(heap: *mut c_void, flags: u32, address: *const c_void, bytes: usize) -> *mut c_void {
    let result = unsafe { win::HeapReAlloc(heap,flags,address,bytes) };
    record("HeapReAlloc",address as usize,bytes,flags,result as usize,heap as usize); result
}
extern "system" fn heap_free(heap: *mut c_void, flags: u32, address: *const c_void) -> i32 {
    let result = unsafe { win::HeapFree(heap,flags,address) };
    record("HeapFree",address as usize,0,flags,result as usize,heap as usize); result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_reserve_commit_decommit_release_preserves_native_results() {
        let base = virtual_alloc(std::ptr::null(),2 << 20,win::MEM_RESERVE,win::PAGE_READWRITE);
        assert!(!base.is_null());
        assert_eq!(virtual_alloc(base,1 << 20,win::MEM_COMMIT,win::PAGE_READWRITE),base);
        assert_eq!(virtual_alloc(base,1 << 20,win::MEM_COMMIT,win::PAGE_READWRITE),base);
        assert_ne!(virtual_free(base,1 << 20,win::MEM_DECOMMIT),0);
        assert_ne!(virtual_free(base,0,win::MEM_RELEASE),0);
    }
    #[test]
    fn observed_native_heap_realloc_keeps_bytes_and_last_error() {
        let heap = unsafe { win::HeapCreate(0,0,0) };
        assert!(!heap.is_null());
        let p = heap_alloc(heap,0,2 << 20);
        assert!(!p.is_null());
        unsafe { std::ptr::write_bytes(p,0x6d,4096); }
        let p = heap_realloc(heap,0,p,3 << 20);
        assert!(!p.is_null());
        assert!(unsafe { std::slice::from_raw_parts(p.cast::<u8>(),4096) }.iter().all(|&v|v==0x6d));
        assert_ne!(heap_free(heap,0,p),0);
        // Compare host vs observed failure including last-error behavior.
        unsafe { SetLastError(1234); }
        let expected = unsafe { win::VirtualFree(std::ptr::null_mut(),1,win::MEM_RELEASE) };
        let expected_error = unsafe { GetLastError() };
        unsafe { SetLastError(1234); }
        assert_eq!(virtual_free(std::ptr::null_mut(),1,win::MEM_RELEASE),expected);
        assert_eq!(unsafe { GetLastError() },expected_error);
        unsafe { win::HeapDestroy(heap); }
    }

    #[test]
    fn rtl_heap_observation_preserves_data() {
        let heap=unsafe {win::HeapCreate(0,0,0)};
        assert!(!heap.is_null());
        let p=rtl_alloc(heap,0,8192);assert!(!p.is_null());
        unsafe {std::ptr::write_bytes(p,0xa7,8192)};
        let p=rtl_realloc(heap,0,p,16384);assert!(!p.is_null());
        assert!(unsafe {std::slice::from_raw_parts(p.cast::<u8>(),8192)}.iter().all(|&v|v==0xa7));
        assert_ne!(rtl_free(heap,0,p),0);
        unsafe {win::HeapDestroy(heap);}
    }

    #[test]
    #[ignore = "isolated instrumentation cost and retention workload; --nocapture"]
    fn native_heap_observation_cost_and_retained_capacity() {
        use std::time::Instant;
        let _=recorder();
        let heap=unsafe {win::HeapCreate(0,8 << 20,0)};
        assert!(!heap.is_null());
        let summary=|stage: &str| {
            let mut value: win::HEAP_SUMMARY=unsafe {std::mem::zeroed()};
            value.cb=std::mem::size_of_val(&value) as u32;
            // Only this fixture owns this heap; no destruction race.
            let ok=unsafe {win::HeapSummary(heap,0,&mut value)};
            eprintln!("native heap {stage}: summary_ok={ok} allocated={} committed={} reserved={}",value.cbAllocated,value.cbCommitted,value.cbReserved);
        };
        summary("initial");
        let start=Instant::now();
        let mut blocks=Vec::new();
        for _ in 0..4096 {let p=heap_alloc(heap,0,4096);assert!(!p.is_null());blocks.push(p);}
        let alloc_us=start.elapsed().as_micros();summary("live");
        let start=Instant::now();
        for p in blocks {assert_ne!(heap_free(heap,0,p),0);}
        let free_us=start.elapsed().as_micros();summary("freed");
        eprintln!("native heap fixture: 4096 alloc_us={alloc_us} free_us={free_us} tracing={} dropped={}",recorder().is_some(),DROPPED.load(Relaxed));
        unsafe {win::HeapDestroy(heap);}
        // Give the independent writer and first region census time to flush.
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}
