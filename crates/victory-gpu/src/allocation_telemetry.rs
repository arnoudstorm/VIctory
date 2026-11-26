//! Opt-in memory observations. Producers never wait on the writer.
use std::io::{BufWriter, Write};
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicU64, Ordering::Relaxed},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CAPACITY: usize = 1024;
const LARGE: u64 = 16 << 20;
const PERIOD: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, Debug, Default)]
struct Event {
    kind: &'static str,
    id: u64,
    bytes: u64,
    aux: u64,
    us: u64,
}

struct Ring<const N: usize> {
    events: [Event; N],
    head: usize,
    len: usize,
}

impl<const N: usize> Ring<N> {
    const fn new() -> Self {
        Self {
            events: [Event {
                kind: "",
                id: 0,
                bytes: 0,
                aux: 0,
                us: 0,
            }; N],
            head: 0,
            len: 0,
        }
    }

    fn push(&mut self, event: Event) -> bool {
        if self.len == N {
            return false;
        }
        self.events[(self.head + self.len) % N] = event;
        self.len += 1;
        true
    }

    fn pop(&mut self) -> Option<Event> {
        if self.len == 0 {
            return None;
        }
        let event = self.events[self.head];
        self.head = (self.head + 1) % N;
        self.len -= 1;
        Some(event)
    }
}

static EVENTS: Mutex<Ring<CAPACITY>> = Mutex::new(Ring::new());
static LOST: AtomicU64 = AtomicU64::new(0);
static NEXT: AtomicU64 = AtomicU64::new(1);
static PIPELINE_ACTIVE: AtomicU64 = AtomicU64::new(0);
static PIPELINE_COMPLETED: AtomicU64 = AtomicU64::new(0);
static ENABLED: OnceLock<bool> = OnceLock::new();

pub(crate) fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        let Some(path) = std::env::var_os("VICTORY_GPU_ALLOCATION_LOG") else {
            return false;
        };
        if victory_core::ensure_outside_repo(std::path::Path::new(&path)).is_err() {
            return false;
        }
        let Ok(file) = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        else {
            return false;
        };
        std::thread::Builder::new()
            .name("gpu-allocation-samples".into())
            .spawn(move || {
                if let Err(error) = sample(BufWriter::with_capacity(32 << 10, file)) {
                    tracing::warn!(%error, "GPU allocation sampler stopped");
                }
            })
            .is_ok()
    })
}

fn utc_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

pub(crate) fn note(kind: &'static str, id: u64, bytes: u64, aux: u64) {
    if !enabled() {
        return;
    }
    if let Ok(mut ring) = EVENTS.try_lock() {
        if ring.push(Event {
            kind,
            id,
            bytes,
            aux,
            us: utc_us(),
        }) {
            return;
        }
    }
    LOST.fetch_add(1, Relaxed);
}

pub(crate) struct Lifetime {
    kind: &'static str,
    id: u64,
    bytes: u64,
    began_us: u64,
}

impl Lifetime {
    pub(crate) fn used_by(&self, submission: u64) {
        if self.id != 0 {
            note("staging_copy", self.id, self.bytes, submission);
        }
    }

    pub(crate) fn new(kind: &'static str, bytes: u64, aux: u64) -> Self {
        if bytes > LARGE && enabled() {
            let id = NEXT.fetch_add(1, Relaxed);
            let began_us = utc_us();
            note(kind, id, bytes, aux);
            Self {
                kind,
                id,
                bytes,
                began_us,
            }
        } else {
            Self {
                kind,
                id: 0,
                bytes,
                began_us: 0,
            }
        }
    }
}

pub(crate) struct PipelineScope(bool);

impl PipelineScope {
    pub(crate) fn new() -> Self {
        let active = enabled();
        if active {
            PIPELINE_ACTIVE.fetch_add(1, Relaxed);
        }
        Self(active)
    }
}

impl Drop for PipelineScope {
    fn drop(&mut self) {
        if self.0 {
            PIPELINE_ACTIVE.fetch_sub(1, Relaxed);
            PIPELINE_COMPLETED.fetch_add(1, Relaxed);
        }
    }
}

impl Drop for Lifetime {
    fn drop(&mut self) {
        if self.id == 0 {
            return;
        }
        let kind = match self.kind {
            "texture_create" => "texture_release",
            "staging_create" => "staging_release",
            "card_create" => "card_release",
            _ => "shadow_release",
        };
        note(
            kind,
            self.id,
            self.bytes,
            utc_us().saturating_sub(self.began_us),
        );
    }
}

fn sample(mut out: impl Write) -> std::io::Result<()> {
    let mut batch = [Event::default(); CAPACITY];
    loop {
        let started = Instant::now();
        let mut n = 0;
        if let Ok(mut ring) = EVENTS.try_lock() {
            while n < batch.len() {
                let Some(event) = ring.pop() else { break };
                batch[n] = event;
                n += 1;
            }
        }
        for event in &batch[..n] {
            writeln!(
                out,
                "{{\"kind\":\"{}\",\"id\":{},\"bytes\":{},\"aux\":{},\"utc_us\":{}}}",
                event.kind, event.id, event.bytes, event.aux, event.us
            )?;
        }
        let (working, private) = crate::copystat::process_memory();
        let (available, commit_left) = crate::copystat::system_memory();
        let (started_submissions, retired, completed, video_memory) =
            crate::host::allocation_fence_snapshot();
        let dxgi = video_memory.map_or_else(|| "null".to_owned(), |v| format!("{v:?}"));
        let textures = crate::resource::texture_memory_totals();
        let staging = crate::texture::staged_bytes_for_sampler();
        let shadows = crate::shadow::live_copy_bytes_for_sampler();
        let card = crate::shadow::allocation_card_bytes();
        let tiles = crate::tiles::line();
        let alive = crate::host::ALIVE.each_ref().map(|v| v.load(Relaxed));
        let made = crate::host::MADE.each_ref().map(|v| v.load(Relaxed));
        let pipeline = crate::pipeline::kept();
        let pipeline_workers = [
            PIPELINE_ACTIVE.load(Relaxed),
            PIPELINE_COMPLETED.load(Relaxed),
        ];
        writeln!(
            out,
            "{{\"kind\":\"sample\",\"utc_us\":{},\"frame\":{},\"available_bytes\":{available},\"private_bytes\":{private},\"working_set_bytes\":{working},\"commit_left_bytes\":{commit_left},\"dxgi\":{dxgi},\"submissions_started\":{started_submissions},\"submissions_retired\":{retired},\"host_fence_completed\":{completed},\"texture_classes\":{textures:?},\"staging_bytes\":{staging},\"shadow_span_bytes\":{shadows},\"card_bytes\":{card:?},\"tile_mib\":[{},{},{}],\"host_alive\":{alive:?},\"host_attempts\":{made:?},\"pipeline_cache\":{pipeline:?},\"pipeline_workers\":{pipeline_workers:?},\"event_drops\":{}}}",
            utc_us(),
            crate::frame::current(),
            tiles.0,
            tiles.1,
            tiles.2,
            LOST.load(Relaxed)
        )?;
        out.flush()?;
        std::thread::sleep(PERIOD.saturating_sub(started.elapsed()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_preserves_order_under_wrap_and_rejects_overflow() {
        let mut ring = Ring::<2>::new();
        let event = |id| Event {
            id,
            ..Event::default()
        };
        assert!(ring.push(event(1)));
        assert!(ring.push(event(2)));
        assert!(!ring.push(event(3)));
        assert_eq!(ring.pop().unwrap().id, 1);
        assert!(ring.push(event(4)));
        assert_eq!(ring.pop().unwrap().id, 2);
        assert_eq!(ring.pop().unwrap().id, 4);
        assert!(ring.pop().is_none());
    }

    #[test]
    #[ignore = "requires VICTORY_GPU_ALLOCATION_LOG at a new path outside the repository"]
    fn samples_without_presents_and_persists_large_lifetimes() {
        assert!(enabled());
        let path = std::env::var_os("VICTORY_GPU_ALLOCATION_LOG").unwrap();
        let small = Lifetime::new("staging_create", LARGE, 0);
        assert_eq!(small.id, 0);
        let large = Lifetime::new("staging_create", LARGE + 1, 0);
        let id = large.id;
        assert_ne!(id, 0);
        large.used_by(42);
        drop(large);
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let lines = std::fs::read_to_string(&path).unwrap_or_default();
            let created = format!("\"kind\":\"staging_create\",\"id\":{id},");
            let used = format!("\"kind\":\"staging_copy\",\"id\":{id},");
            let released = format!("\"kind\":\"staging_release\",\"id\":{id},");
            if lines.matches("\"kind\":\"sample\"").count() >= 3 {
                assert!(lines.find(&created).unwrap() < lines.find(&used).unwrap());
                assert!(lines.find(&used).unwrap() < lines.find(&released).unwrap());
                return;
            }
            assert!(
                Instant::now() < deadline,
                "sampler did not persist three observations"
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }
}
