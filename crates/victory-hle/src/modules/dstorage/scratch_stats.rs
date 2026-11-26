//! Optional CPU scratch accounting. Layout capacity is not OS commitment.
use std::sync::{OnceLock,atomic::{AtomicU64,Ordering::Relaxed}};
#[derive(Default)]
struct Stats{requested:AtomicU64,fresh:AtomicU64,live:AtomicU64,peak:AtomicU64,allocations:AtomicU64,reuses:AtomicU64,wait_ns:AtomicU64,large:AtomicU64}
impl Stats {
    const fn new()->Self{Self{requested:AtomicU64::new(0),fresh:AtomicU64::new(0),live:AtomicU64::new(0),peak:AtomicU64::new(0),allocations:AtomicU64::new(0),reuses:AtomicU64::new(0),wait_ns:AtomicU64::new(0),large:AtomicU64::new(0)}}
    fn acquire(&self,size:u64,cap:u64,reused:bool,wait:u64){
        self.requested.fetch_add(size,Relaxed);self.wait_ns.fetch_add(wait,Relaxed);
        if reused{self.reuses.fetch_add(1,Relaxed);}else{self.allocations.fetch_add(1,Relaxed);self.fresh.fetch_add(cap,Relaxed);}
        if size>super::BufferPool::MAX_BUFFER as u64{self.large.fetch_add(1,Relaxed);}
        let live=self.live.fetch_add(cap,Relaxed)+cap;self.peak.fetch_max(live,Relaxed);
    }
    fn release(&self,cap:u64){self.live.fetch_sub(cap,Relaxed);}
}
static STATS:[Stats;4]=[const{Stats::new()};4];
pub(super) fn acquire(role:usize,size:usize,cap:usize,reused:bool,wait:u64){for i in [role,3]{STATS[i].acquire(size as u64,cap as u64,reused,wait)}}
pub(super) fn release(role:usize,cap:u64){for i in [role,3]{STATS[i].release(cap)}}
pub(super) fn enabled()->bool{
    static ON:OnceLock<bool>=OnceLock::new();
    *ON.get_or_init(||{
        let Some(path)=std::env::var_os("VICTORY_DSTORAGE_STATS")else{return false};
        if victory_core::ensure_outside_repo(std::path::Path::new(&path)).is_err(){return false}
        let Ok(file)=std::fs::File::create(path)else{return false};
        std::thread::Builder::new().name("dstorage-stats".into()).spawn(move||{
            use std::io::Write;
            let mut file=std::io::BufWriter::new(file);
            loop{
                let utc=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
                let retained=super::BUFFERS.lock().unwrap_or_else(|e|e.into_inner()).retained;
                for (role,s) in STATS.iter().enumerate(){
                    if writeln!(file,"{{\"utc_ms\":{utc},\"role\":{role},\"requested\":{},\"fresh_capacity\":{},\"active_capacity\":{},\"peak_active\":{},\"allocations\":{},\"reuses\":{},\"mutex_ns\":{},\"over_cap_requests\":{},\"pool_retained\":{retained}}}",s.requested.load(Relaxed),s.fresh.load(Relaxed),s.live.load(Relaxed),s.peak.load(Relaxed),s.allocations.load(Relaxed),s.reuses.load(Relaxed),s.wait_ns.load(Relaxed),s.large.load(Relaxed)).is_err(){return}
                }
                if file.flush().is_err(){return}
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }).is_ok()
    })
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn concurrent_capacity_accounting_balances_errors_and_reuse(){
        let stats=std::sync::Arc::new(Stats::new());
        let handles:Vec<_>=(0..4).map(|_|{let stats=stats.clone();std::thread::spawn(move||{for _ in 0..1000{stats.acquire(32,64,true,7);stats.release(64)}})}).collect();
        for h in handles{h.join().unwrap();}
        assert_eq!(stats.live.load(Relaxed),0);assert_eq!(stats.requested.load(Relaxed),128000);assert_eq!(stats.reuses.load(Relaxed),4000);assert_eq!(stats.wait_ns.load(Relaxed),28000);assert!(stats.peak.load(Relaxed)<=256);
        stats.acquire(5<<20,6<<20,false,0);stats.release(6<<20);
        assert_eq!(stats.fresh.load(Relaxed),6<<20);assert_eq!(stats.large.load(Relaxed),1);assert_eq!(stats.live.load(Relaxed),0);
    }
}
