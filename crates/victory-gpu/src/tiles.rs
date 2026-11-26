//! Windows of title memory that hold only the pages copied into them
//! (`VICTORY_TILED_WINDOWS=1`, an experiment, off by default).
//!
//! A window is a reserved buffer of its full size. Its 64 KiB tiles are
//! mapped, when the CPU first writes one, onto tiles of pooled heaps in the
//! PC's memory (custom, write-back, pool L0); one placed buffer covers each
//! heap and is mapped, so the CPU writes a tile through it. A tile never
//! mapped reads 0 on the card (tiled resources tier 3), as the never-copied
//! bytes of a committed window (host-zeroed) did. Measured on the 780M
//! (`host::window_probe_tests`): a committed window costs its whole size at
//! once; a reserved one costs only its mapped tiles, and the card sees what
//! the CPU wrote through the placed buffer.
//!
//! Tiles can not go on heaps opened on memory of our own (the device is
//! removed, measured), so a tiled window's CPU side is one address per
//! tile, not one range: see `Shadow::read_at` and `Shadow::write_at`.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::Win32::Graphics::Direct3D12::*;

/// The size of a tile, and the console's page grid windows are cut on.
pub(crate) const TILE: usize = 64 << 10;
/// Tiles per pooled heap (4 MiB heaps).
const PER_HEAP: u32 = 64;

/// `VICTORY_TILED_WINDOWS=1`: windows outside buffers are tiled.
pub(crate) fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        let on = std::env::var_os("VICTORY_TILED_WINDOWS").is_some_and(|v| v == "1");
        if on {
            tracing::info!("windows outside buffers are tiled: only the tiles copied into hold memory (VICTORY_TILED_WINDOWS=1)");
        }
        on
    })
}

/// A heap of the pool: the heap, the placed buffer over it (kept alive
/// while mapped), its CPU address, and its free tiles.
struct Block {
    heap: ID3D12Heap,
    _cpu_buffer: ID3D12Resource,
    cpu: usize,
    free: Vec<u32>,
    #[cfg(feature = "diagnostics")]
    identity: u64,
}

// SAFETY: the COM objects are free-threaded; `cpu` is an address.
unsafe impl Send for Block {}

/// The heaps; a heap whose tiles are all free again is let go (one such
/// is kept for the next window), so the pool follows the tiles in use.
static POOL: Mutex<Vec<Option<Block>>> = Mutex::new(Vec::new());

/// Tiles mapped for declared shader writes, and by copies of title bytes,
/// since the start.
pub(crate) static BY: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];

/// Tiles mapped now, and heaps held, for the memory line.
pub(crate) static MAPPED: AtomicU64 = AtomicU64::new(0);
static HEAPS: AtomicU64 = AtomicU64::new(0);

/// A tile taken from the pool: which block, which tile of it, and its CPU
/// address.
#[derive(Clone, Copy)]
pub(crate) struct Slot {
    block: usize,
    tile: u32,
    pub(crate) cpu: usize,
    #[cfg(feature = "diagnostics")]
    identity: u64,
}

/// A tile of the pool, zeroed (a window's bytes not copied read 0); `None`
/// if no heap could be made.
fn take() -> Option<(Slot, ID3D12Heap)> {
    let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
    // The fullest heap with a free tile, so that the others empty out.
    let fullest = pool
        .iter()
        .enumerate()
        .filter_map(|(i, b)| b.as_ref().filter(|b| !b.free.is_empty()).map(|b| (b.free.len(), i)))
        .min()
        .map(|(_, i)| i);
    let block = match fullest {
        Some(i) => i,
        None => {
            let made = make_block()?;
            HEAPS.fetch_add(1, Ordering::Relaxed);
            match pool.iter().position(Option::is_none) {
                Some(i) => {
                    pool[i] = Some(made);
                    i
                }
                None => {
                    pool.push(Some(made));
                    pool.len() - 1
                }
            }
        }
    };
    let b = pool[block].as_mut()?;
    let tile = b.free.pop()?;
    let cpu = b.cpu + tile as usize * TILE;
    // SAFETY: a tile of the block's mapped placed buffer, free until now.
    unsafe { std::ptr::write_bytes(cpu as *mut u8, 0, TILE) };
    MAPPED.fetch_add(1, Ordering::Relaxed);
    Some((Slot {
        block, tile, cpu,
        #[cfg(feature = "diagnostics")]
        identity: b.identity * u64::from(PER_HEAP) + u64::from(tile),
    }, b.heap.clone()))
}

/// Gives tiles back (only once nothing runs that uses them: see
/// [`Tiles`]'s drop).
fn give(slots: Vec<Slot>, _window: u64) {
    let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
    for s in slots {
        if let Some(Some(b)) = pool.get_mut(s.block) {
            #[cfg(feature = "diagnostics")]
            if _window != 0 {
                crate::allocation_telemetry::note("window_tile_free", _window, s.identity, 0);
            }
            b.free.push(s.tile);
            MAPPED.fetch_sub(1, Ordering::Relaxed);
        }
    }
    // Empty heaps go, but one: the window that comes next takes it.
    let empty: Vec<usize> = pool
        .iter()
        .enumerate()
        .filter(|(_, b)| b.as_ref().is_some_and(|b| b.free.len() == PER_HEAP as usize))
        .map(|(i, _)| i)
        .collect();
    for &i in empty.iter().skip(1) {
        #[cfg(feature = "diagnostics")]
        if let Some(block) = &pool[i] && block.identity != 0 {
            crate::allocation_telemetry::note("window_heap_free", block.identity, 0, 0);
        }
        pool[i] = None;
        HEAPS.fetch_sub(1, Ordering::Relaxed);
    }
}

fn make_block() -> Option<Block> {
    let host = crate::host::host()?;
    let size = PER_HEAP as u64 * TILE as u64;
    // SAFETY: plain object creation and mapping.
    unsafe {
        let desc = D3D12_HEAP_DESC {
            SizeInBytes: size,
            Properties: D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_CUSTOM,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_WRITE_BACK,
                MemoryPoolPreference: D3D12_MEMORY_POOL_L0,
                ..Default::default()
            },
            Alignment: 0,
            Flags: D3D12_HEAP_FLAG_ALLOW_ONLY_BUFFERS,
        };
        let mut heap: Option<ID3D12Heap> = None;
        if let Err(e) = host.device.CreateHeap(&desc, &mut heap) {
            tracing::warn!(error = %e, "no heap for window tiles");
            return None;
        }
        let heap = heap?;
        let mut cpu_buffer: Option<ID3D12Resource> = None;
        host.device
            .CreatePlacedResource(&heap, 0, &buffer_desc(size), D3D12_RESOURCE_STATE_COMMON, None, &mut cpu_buffer)
            .ok()?;
        let cpu_buffer = cpu_buffer?;
        let mut at = std::ptr::null_mut();
        cpu_buffer.Map(0, None, Some(&mut at)).ok()?;
        #[cfg(feature = "diagnostics")]
        let identity = if crate::allocation_telemetry::window_backing_enabled() {
            use windows::core::Interface;
            static NEXT_HEAP: AtomicU64 = AtomicU64::new(1);
            let id = NEXT_HEAP.fetch_add(1, Ordering::Relaxed);
            crate::allocation_telemetry::note("window_heap_create", id, size, heap.as_raw() as u64);
            id
        } else {
            0
        };
        Some(Block {
            heap, _cpu_buffer: cpu_buffer, cpu: at as usize, free: (0..PER_HEAP).rev().collect(),
            #[cfg(feature = "diagnostics")]
            identity,
        })
    }
}

fn buffer_desc(width: u64) -> D3D12_RESOURCE_DESC {
    D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Width: width,
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        Flags: D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
        ..Default::default()
    }
}

/// A reserved buffer of `size` bytes (a multiple of [`TILE`]) with no tile
/// mapped.
pub(crate) fn reserved(size: u64) -> Option<ID3D12Resource> {
    let host = crate::host::host()?;
    let mut buffer: Option<ID3D12Resource> = None;
    // SAFETY: plain object creation.
    match unsafe { host.device.CreateReservedResource(&buffer_desc(size), D3D12_RESOURCE_STATE_COMMON, None, &mut buffer) } {
        Ok(()) => buffer,
        Err(e) => {
            tracing::warn!(error = %e, size, "no reserved buffer for a window");
            None
        }
    }
}

/// The CPU side of a tiled window: per tile, its CPU address (0: not
/// mapped).
pub(crate) struct Tiles {
    at: Box<[AtomicUsize]>,
    slots: Mutex<Vec<Slot>>,
    #[cfg(feature = "diagnostics")]
    window: u64,
}

/// 64 KiB of zeros: what a tile not mapped reads as.
static ZEROS: [u8; TILE] = [0; TILE];

impl Tiles {
    pub(crate) fn new(len: usize) -> Tiles {
        Tiles {
            at: (0..len.div_ceil(TILE)).map(|_| AtomicUsize::new(0)).collect(),
            slots: Mutex::default(),
            #[cfg(feature = "diagnostics")]
            window: 0,
        }
    }

    #[cfg(feature = "diagnostics")]
    pub(crate) fn trace_window(&mut self, window: u64) {
        if crate::allocation_telemetry::window_backing_enabled() {
            self.window = window;
        }
    }

    /// Where offset `off` of the window is, for reading: in its tile, or
    /// in [`ZEROS`] if the tile is not mapped. Valid to the tile's end.
    pub(crate) fn read_at(&self, off: usize) -> *const u8 {
        match self.at.get(off / TILE).map(|a| a.load(Ordering::Acquire)) {
            Some(cpu) if cpu != 0 => (cpu + off % TILE) as *const u8,
            _ => ZEROS[off % TILE..].as_ptr(),
        }
    }

    /// Whether a nonempty byte range already has backing for every tile.
    /// Slots stay mapped for this window's entire lifetime.
    pub(crate) fn covers(&self, from: usize, to: usize) -> bool {
        from < to
            && self.at.get(from / TILE..to.div_ceil(TILE))
                .is_some_and(|tiles| tiles.iter().all(|tile| tile.load(Ordering::Acquire) != 0))
    }

    /// Where offset `off` of the window is, for writing: its tile mapped
    /// first if it is not (on `buffer`, the window's reserved buffer, on
    /// the host's queue, so ahead of every submission sent after). Null if
    /// no tile could be had. Valid to the tile's end.
    pub(crate) fn write_at(&self, buffer: &ID3D12Resource, off: usize) -> *mut u8 {
        let i = off / TILE;
        let Some(slot) = self.at.get(i) else {
            return std::ptr::null_mut();
        };
        let cpu = slot.load(Ordering::Acquire);
        if cpu != 0 {
            return (cpu + off % TILE) as *mut u8;
        }
        let mut slots = self.slots.lock().unwrap_or_else(|e| e.into_inner());
        // Another thread may have mapped it meanwhile.
        let cpu = slot.load(Ordering::Acquire);
        if cpu != 0 {
            return (cpu + off % TILE) as *mut u8;
        }
        let Some((taken, heap)) = take() else {
            return std::ptr::null_mut();
        };
        let Some(host) = crate::host::host() else {
            give(vec![taken], 0);
            return std::ptr::null_mut();
        };
        host.map_tile(buffer, i as u32, &heap, taken.tile);
        #[cfg(feature = "diagnostics")]
        if self.window != 0 {
            crate::allocation_telemetry::note("window_tile_bind", self.window, i as u64, taken.identity);
        }
        slots.push(taken);
        slot.store(taken.cpu, Ordering::Release);
        (taken.cpu + off % TILE) as *mut u8
    }

    /// Maps every tile of offsets `[from, to)` (shaders write there).
    pub(crate) fn map_range(&self, buffer: &ID3D12Resource, from: usize, to: usize) {
        let before = self.mapped();
        let mut at = from - from % TILE;
        while at < to {
            self.write_at(buffer, at);
            at += TILE;
        }
        BY[0].fetch_add((self.mapped() - before) as u64, Ordering::Relaxed);
    }

    /// Tiles mapped in this window.
    pub(crate) fn mapped(&self) -> usize {
        self.slots.lock().unwrap_or_else(|e| e.into_inner()).len()
    }
}

/// The tiles of a window let go, given back once every submission running
/// now has finished.
struct Back(Vec<Slot>, u64);

impl Drop for Back {
    fn drop(&mut self) {
        give(std::mem::take(&mut self.0), self.1);
    }
}

impl Drop for Tiles {
    fn drop(&mut self) {
        let slots = std::mem::take(&mut *self.slots.lock().unwrap_or_else(|e| e.into_inner()));
        if !slots.is_empty() {
            #[cfg(feature = "diagnostics")]
            let window = self.window;
            #[cfg(not(feature = "diagnostics"))]
            let window = 0;
            crate::host::retire(Back(slots, window));
        }
    }
}

/// `[from, to)` of a window cut where tiles end: (offset, bytes) pieces.
pub(crate) fn chunks(from: usize, to: usize) -> impl Iterator<Item = (usize, usize)> {
    let mut at = from;
    std::iter::from_fn(move || {
        (at < to).then(|| {
            let n = (TILE - at % TILE).min(to - at);
            let piece = (at, n);
            at += n;
            piece
        })
    })
}

/// For the memory line: tiles mapped (MB) and heaps held (MB).
pub(crate) fn line() -> (u64, u64, u64) {
    let mb = |tiles: u64| tiles * TILE as u64 >> 20;
    (mb(MAPPED.load(Ordering::Relaxed)), mb(HEAPS.load(Ordering::Relaxed) * PER_HEAP as u64), mb(BY[0].load(Ordering::Relaxed)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_cut_where_tiles_end() {
        let c: Vec<_> = chunks(TILE - 10, 3 * TILE + 5).collect();
        assert_eq!(c, vec![(TILE - 10, 10), (TILE, TILE), (2 * TILE, TILE), (3 * TILE, 5)]);
        assert_eq!(chunks(5, 5).count(), 0);
        assert_eq!(chunks(0, 7).collect::<Vec<_>>(), vec![(0, 7)]);
    }

    /// A tiled window reads 0 where nothing is mapped; what the CPU writes
    /// is read back from its tile; and a copy on the card through the
    /// reserved buffer sees it, while an unmapped tile copies as 0.
    #[test]
    fn tiled_windows_read_zero_until_written_and_the_card_sees_writes() {
        let Some(host) = crate::host::host() else { return };
        let Some(buffer) = reserved(4 << 20) else { return };
        let tiles = Tiles::new(4 << 20);
        // SAFETY: read_at is valid to its tile's end.
        assert_eq!(unsafe { *tiles.read_at(3 * TILE + 7) }, 0);
        for (off, n) in chunks(TILE - 100, 2 * TILE + 100) {
            let at = tiles.write_at(&buffer, off);
            assert!(!at.is_null());
            // SAFETY: valid to the tile's end; `chunks` stays inside it.
            unsafe { std::ptr::write_bytes(at, 0x5a, n) };
        }
        assert_eq!(tiles.mapped(), 3);
        // SAFETY: as above.
        unsafe {
            assert_eq!(*tiles.read_at(TILE - 100), 0x5a);
            assert_eq!(*tiles.read_at(2 * TILE + 99), 0x5a);
            assert_eq!(*tiles.read_at(TILE - 101), 0);
        }
        let out = host.copy_out_for_test(&buffer, 0, 4 * TILE as u64).expect("read back");
        assert!(out[..TILE - 100].iter().all(|&b| b == 0), "unmapped or unwritten bytes read 0");
        assert!(out[TILE - 100..2 * TILE + 100].iter().all(|&b| b == 0x5a), "the card sees the CPU's writes");
        assert!(out[2 * TILE + 100..].iter().all(|&b| b == 0), "the rest reads 0");
        assert!(host.device_alive());
    }
}
