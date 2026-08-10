//! Writing the materialised index under `.decisions/index/`.
//!
//! The index is a cache with a proof: `reindex` derives it from the log
//! alone, and rebuilding after deletion must reproduce the same bytes —
//! any divergence means state leaked into the cache (PRD §5). The
//! directory is gitignored at `ledger init`; nothing here is ever the
//! source of anything.

use std::path::{Path, PathBuf};

use crate::store::Store;

use super::turtle::emit;

/// The index file, relative to the store directory.
pub const INDEX_FILE: &str = "index/ledger.ttl";

/// What a reindex produced.
#[derive(Debug)]
pub struct Reindexed {
    pub path: PathBuf,
    pub bytes: usize,
    pub triples: usize,
}

/// Derive the index from the log and write it. Deleting `index/` first is
/// unnecessary: the emission depends on nothing but the log, so a rebuild
/// is a plain overwrite with provably identical content.
pub fn reindex(root: &Path) -> Result<Reindexed, String> {
    let store = crate::store::load(root);
    write_index(root, &store)
}

/// Write the index for an already-loaded store.
pub fn write_index(root: &Path, store: &Store) -> Result<Reindexed, String> {
    let ttl = emit(store);
    let path = root.join(crate::STORE_DIR).join(INDEX_FILE);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    product_core::fileops::write_file_atomic(&path, &ttl).map_err(|e| e.to_string())?;
    Ok(Reindexed {
        path,
        bytes: ttl.len(),
        triples: ttl.lines().filter(|l| l.ends_with(" .") || l.ends_with(" ;")).count(),
    })
}
