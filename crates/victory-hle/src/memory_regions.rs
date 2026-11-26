//! Optional region census. Neutral Windows types, never guessed owners.
use std::collections::BTreeMap;
use windows_sys::Win32::System::{Memory::*, Threading::GetCurrentProcess};

// Reserved, committed private, committed mapped, committed image, other commit.
type Counts = [u64; 5];

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"), '"' => out.push_str("\\\""),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}",c as u32)),
            c => out.push(c),
        }
    }
    out.push('"'); out
}

fn modules(file: &mut impl std::io::Write, utc: u128) -> std::io::Result<()> {
    use windows_sys::Win32::System::ProcessStatus::{K32EnumProcessModules,K32GetModuleFileNameExW,K32GetModuleInformation,MODULEINFO};
    let mut handles = vec![std::ptr::null_mut(); 2048];
    let mut needed = 0;
    // SAFETY: sized handle and information buffers, current process only.
    unsafe {
        if K32EnumProcessModules(GetCurrentProcess(),handles.as_mut_ptr(),(handles.len()*std::mem::size_of::<usize>()) as u32,&mut needed) != 0 {
            for &handle in handles.iter().take((needed as usize / std::mem::size_of::<usize>()).min(handles.len())) {
                let mut info: MODULEINFO=std::mem::zeroed();
                let mut name=[0u16;1024];
                let len=K32GetModuleFileNameExW(GetCurrentProcess(),handle,name.as_mut_ptr(),name.len() as u32) as usize;
                if K32GetModuleInformation(GetCurrentProcess(),handle,&mut info,std::mem::size_of_val(&info) as u32)!=0 {
                    writeln!(file,"{{\"utc_ms\":{utc},\"module_base\":{},\"module_bytes\":{},\"path\":{}}}",info.lpBaseOfDll as usize,info.SizeOfImage,json_string(&String::from_utf16_lossy(&name[..len.min(name.len())])))?;
                }
            }
        }
    }
    for (base,path) in crate::runtime::image_paths() {
        writeln!(file,"{{\"utc_ms\":{utc},\"title_image_base\":{base},\"path\":{}}}",json_string(&path.to_string_lossy()))?;
    }
    Ok(())
}
fn bucket(state: u32, kind: u32) -> Option<usize> {
    match state {
        MEM_RESERVE => Some(0),
        MEM_COMMIT => Some(match kind { MEM_PRIVATE => 1, MEM_MAPPED => 2, MEM_IMAGE => 3, _ => 4 }),
        _ => None,
    }
}

fn snapshot() -> (BTreeMap<usize, Counts>, usize) {
    let mut allocations = BTreeMap::<usize, Counts>::new();
    let mut address = 0usize;
    let mut regions = 0;
    loop {
        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: querying this process only, with a sized output structure.
        if unsafe { VirtualQueryEx(GetCurrentProcess(),address as *const _,&mut info,std::mem::size_of_val(&info)) } == 0 { break }
        if let Some(kind) = bucket(info.State,info.Type) {
            allocations.entry(info.AllocationBase as usize).or_default()[kind] += info.RegionSize as u64;
            regions += 1;
        }
        let next = (info.BaseAddress as usize).saturating_add(info.RegionSize);
        if next <= address { break }
        address = next;
    }
    (allocations, regions)
}

fn deltas(before: &BTreeMap<usize, Counts>, after: &BTreeMap<usize, Counts>) -> Vec<(usize, [i128; 5])> {
    let keys: std::collections::BTreeSet<_> = before.keys().chain(after.keys()).copied().collect();
    let mut changes: Vec<_> = keys.into_iter().filter_map(|base| {
        let a = before.get(&base).copied().unwrap_or_default();
        let b = after.get(&base).copied().unwrap_or_default();
        let delta = std::array::from_fn(|i| b[i] as i128 - a[i] as i128);
        (delta != [0; 5]).then_some((base, delta))
    }).collect();
    changes.sort_unstable_by_key(|(_, delta)| std::cmp::Reverse(delta.iter().map(|v|v.abs()).sum::<i128>()));
    changes
}

pub(crate) fn start() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let Some(path) = std::env::var_os("VICTORY_MEMORY_REGIONS") else { return };
        if victory_core::ensure_outside_repo(std::path::Path::new(&path)).is_err() { return }
        let Ok(file) = std::fs::File::create(path) else { return };
        let _ = std::thread::Builder::new().name("memory-regions".into()).spawn(move || {
            use std::io::Write;
            let mut file = std::io::BufWriter::new(file);
            let mut before = BTreeMap::new();
            let interval=std::env::var("VICTORY_REGION_INTERVAL_MS").ok().and_then(|s|s.parse::<u64>().ok()).unwrap_or(5000).clamp(1000,60000);
            let mut scan=0u64;
            loop {
                let began = std::time::Instant::now();
                let utc = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
                let (after, regions) = snapshot();
                if scan%6==0 && modules(&mut file,utc).is_err(){return}
                let totals: Counts = after.values().fold([0;5], |mut a,b|{for i in 0..5 {a[i]+=b[i];}a});
                let changes = deltas(&before,&after);
                // Every changed allocation, not just the largest: correlate by
                // address and UTC with events. Address reuse can be ambiguous.
                for (base, delta) in changes {
                    let current = after.get(&base).copied().unwrap_or_default();
                    if writeln!(file,"{{\"utc_ms\":{utc},\"allocation_base\":{base},\"counts\":{current:?},\"delta\":{delta:?}}}").is_err(){return}
                }
                if writeln!(file,"{{\"utc_ms\":{utc},\"scan_us\":{},\"regions\":{regions},\"totals\":{totals:?}}}",began.elapsed().as_micros()).is_err(){return}
                if file.flush().is_err(){return}
                before = after;
                scan+=1;
                std::thread::sleep(std::time::Duration::from_millis(interval));
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn region_deltas_distinguish_reserve_commit_overlap_decommit_and_release() {
        // SAFETY: own reservation; never submitted to GPU or other threads.
        unsafe {
            let base = VirtualAlloc(std::ptr::null(), 2 << 20, MEM_RESERVE, PAGE_READWRITE);
            assert!(!base.is_null());
            let key=base as usize;
            let (reserved,_) = snapshot();
            assert_eq!(reserved[&key], [2 << 20,0,0,0,0]);
            assert_eq!(VirtualAlloc(base,1 << 20,MEM_COMMIT,PAGE_READWRITE),base);
            let (committed,_) = snapshot();
            assert_eq!(committed[&key], [1 << 20,1 << 20,0,0,0]);
            assert_eq!(VirtualAlloc(base,1 << 20,MEM_COMMIT,PAGE_READWRITE),base);
            let (repeated,_) = snapshot();
            assert_eq!(repeated[&key],committed[&key],"recommit must not be counted twice");
            assert_ne!(VirtualFree(base,1 << 20,MEM_DECOMMIT),0);
            let (decommitted,_) = snapshot();
            assert_eq!(decommitted[&key],reserved[&key]);
            assert_ne!(VirtualFree(base,0,MEM_RELEASE),0);
            let (released,_) = snapshot();
            assert!(!released.contains_key(&key));
            assert!(deltas(&committed,&released).contains(&(key,[-(1 << 20),-(1 << 20),0,0,0])));
        }
    }
    #[test]
    fn region_types_are_not_interchangeable() {
        assert_eq!(bucket(MEM_COMMIT,MEM_PRIVATE),Some(1));
        assert_eq!(bucket(MEM_COMMIT,MEM_MAPPED),Some(2));
        assert_eq!(bucket(MEM_COMMIT,MEM_IMAGE),Some(3));
        assert_eq!(bucket(MEM_FREE,0),None);
    }

    #[test]
    fn mapped_section_and_loaded_image_keep_their_windows_types() {
        use windows_sys::Win32::Foundation::{CloseHandle,INVALID_HANDLE_VALUE};
        // SAFETY: own anonymous section and view; released before return.
        unsafe {
            let section=CreateFileMappingW(INVALID_HANDLE_VALUE,std::ptr::null(),PAGE_READWRITE,0,65536,std::ptr::null());
            assert!(!section.is_null());
            let view=MapViewOfFile(section,FILE_MAP_WRITE,0,0,65536);
            assert!(!view.Value.is_null());
            let mut info: MEMORY_BASIC_INFORMATION=std::mem::zeroed();
            assert_ne!(VirtualQueryEx(GetCurrentProcess(),view.Value,&mut info,std::mem::size_of_val(&info)),0);
            assert_eq!((info.State,info.Type),(MEM_COMMIT,MEM_MAPPED));
            let module=windows_sys::Win32::System::LibraryLoader::GetModuleHandleA(c"kernel32.dll".as_ptr().cast());
            assert!(!module.is_null());
            assert_ne!(VirtualQueryEx(GetCurrentProcess(),module,&mut info,std::mem::size_of_val(&info)),0);
            assert_eq!(info.Type,MEM_IMAGE);
            UnmapViewOfFile(view);CloseHandle(section);
        }
    }
}
