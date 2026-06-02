//! Checksum helpers — used to assert zero-files-changed on failed apply.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn checksum_all(dirs: &[&PathBuf]) -> BTreeMap<PathBuf, String> {
    let mut out = BTreeMap::new();
    for dir in dirs {
        let path: &Path = dir.as_path();
        if !path.exists() { continue; }
        if let Ok(entries) = std::fs::read_dir(path) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().and_then(|x| x.to_str()) == Some("md") {
                    if let Ok(b) = std::fs::read(&p) {
                        out.insert(p, sha256(&b));
                    }
                }
            }
        }
    }
    out
}

fn sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}
