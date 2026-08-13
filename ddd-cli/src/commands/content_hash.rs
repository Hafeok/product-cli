//! `ddd content-hash` — the pin a ledger entry takes of a stored entry.
//!
//! A `.ddd` entry that is re-decided moves its content hash, so the ledger
//! version pinning it must state the new one. The M8 migration computed
//! these inline and hard-coded them; anyone re-deciding an entry afterwards
//! needs the same number and had no way to ask for it. Read-only: it
//! computes and prints, and writes nothing.

use product_core::error::{ProductError, Result};

use super::resolve_root;

/// Print the content hash of one stored entry, whatever family it is in.
pub fn run(root: Option<std::path::PathBuf>, id: &str) -> Result<()> {
    let repo_root = resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let hash = hash_of(&store, id).ok_or_else(|| {
        ProductError::NotFound(format!(
            "{id} is not a decision, claim or seam in this store"
        ))
    })?;
    println!("{hash}");
    Ok(())
}

/// The three families that carry a content hash, each under its own
/// domain-separation prefix (`ddd_core::basis_pin`).
fn hash_of(store: &ddd_core::store::DddStore, id: &str) -> Option<String> {
    if let Some(d) = store.decisions.iter().find(|d| d.id == id) {
        return Some(ddd_core::basis_pin::decision_content_hash(d));
    }
    if let Some(c) = store.claims.iter().find(|c| c.id == id) {
        return Some(ddd_core::basis_pin::claim_content_hash(c));
    }
    store
        .seams
        .iter()
        .find(|s| s.id == id)
        .map(ddd_core::basis_pin::seam_content_hash)
}
