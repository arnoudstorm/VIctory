use std::hash::Hasher;
use std::path::{Path, PathBuf};

/// Files that can change the translated DXIL or its on-disk encoding. The
/// runtime also adds host-dependent translation settings to this fingerprint.
fn inputs() -> Vec<PathBuf> {
    fn visit(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, out);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                out.push(path);
            }
        }
    }

    let mut files = vec![
        PathBuf::from("src/shader_cache.rs"),
        PathBuf::from("Cargo.toml"),
        PathBuf::from("../victory-shader/Cargo.toml"),
        PathBuf::from("../../Cargo.lock"),
    ];
    visit(Path::new("../victory-shader/src"), &mut files);
    files.sort();
    files
}

fn main() {
    // Two independent FNV-1a streams make accidental collisions vanishingly
    // unlikely without adding a build dependency just for fingerprinting.
    let mut a = Fnv::new(0xcbf2_9ce4_8422_2325);
    let mut b = Fnv::new(0x8422_2325_cbf2_9ce4);
    for path in inputs() {
        println!("cargo:rerun-if-changed={}", path.display());
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        a.write(path.to_string_lossy().as_bytes());
        a.write(&bytes);
        b.write(&bytes);
        b.write(path.to_string_lossy().as_bytes());
    }
    println!(
        "cargo:rustc-env=VICTORY_SHADER_TRANSLATOR_FINGERPRINT={:016x}{:016x}",
        a.finish(),
        b.finish()
    );
}

struct Fnv(u64);

impl Fnv {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
}

impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x100_0000_01b3);
        }
    }
}
