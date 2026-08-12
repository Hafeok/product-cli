//! `ddd bind` — file the declarations a failing contract gate demands.
//!
//! The post-hoc CI-remedy path: for every undischarged contract-surface
//! event in a revision range, file (or amend) one seam declaration per
//! file and append signed bindings covering the range's transition —
//! before/after content hashes from git, base revision the range's base,
//! one binding per changed symbol. The parent state is committed by
//! construction. Judgment stays the author's: a fresh declaration's
//! `verdict_knowledge` is empty until written, and files with a warning.

use std::path::PathBuf;
use std::time::Duration;

use ddd_core::binding::SeamBinding;
use ddd_lsp::HostManager;
use product_core::error::{ProductError, Result};

pub fn run(root: Option<PathBuf>, range: String, wait: u64, verdict: Option<String>) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let (base, head) = super::diff_contracts::parse_range(&range)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let mut manager = HostManager::new(repo_root.clone());
    let report =
        ddd_lsp::revdiff::diff_contracts(&mut manager, base, head, Duration::from_secs(wait))
            .map_err(ProductError::ConfigError)?;
    let undischarged = ddd_core::contracts::check_discharge(&report, &store.seams);
    if undischarged.is_empty() {
        println!("nothing undischarged in {range} — no declaration needed");
        return Ok(());
    }
    let mut filed = 0usize;
    let mut warned = false;
    for file in &report.files {
        let symbols: Vec<&str> = undischarged
            .iter()
            .filter(|u| u.file == file.file)
            .map(|u| u.symbol.as_str())
            .collect();
        if symbols.is_empty() {
            continue;
        }
        let (new_bindings, empty_verdict) =
            bind_file(&repo_root, file, &symbols, &report.base, verdict.as_deref())?;
        filed += new_bindings;
        warned |= empty_verdict;
    }
    println!("{filed} binding(s) filed for {range}");
    if warned {
        println!(
            "warning: declarations with empty verdict_knowledge declare seam cost with no \
             demand absorbed — author what each boundary encodes about the verdict"
        );
    }
    Ok(())
}

/// File (or amend) one file's seam declaration: append a signed binding per
/// undischarged symbol. Returns the count of newly filed bindings plus
/// whether the declaration still carries an empty `verdict_knowledge`.
fn bind_file(
    repo_root: &std::path::Path,
    file: &ddd_core::contracts::FileContracts,
    symbols: &[&str],
    base: &str,
    verdict: Option<&str>,
) -> Result<(usize, bool)> {
    let seam_id = format!("seam/{}/{}", file.language, slug(&file.file));
    let path = ddd_core::seam::path_for(
        &repo_root.join(ddd_core::store::STORE_DIR).join("seams"),
        &seam_id,
    );
    let mut seam = ddd_core::seam::load_at(&path)
        .unwrap_or_else(|| new_seam(&seam_id, &file.file, verdict));
    let mut filed = 0usize;
    for symbol in symbols {
        let binding = SeamBinding::seal(symbol, &file.file, &file.before, &file.after, base);
        if !seam.bindings.iter().any(|b| b.hash == binding.hash) {
            seam.bindings.push(binding);
            filed += 1;
        }
    }
    seam.format = seam.format.max(2);
    let warned = seam.verdict_knowledge.trim().is_empty();
    let yaml = serde_yaml::to_string(&seam).map_err(|e| ProductError::IoError(e.to_string()))?;
    product_core::fileops::write_file_atomic(&path, &yaml)?;
    println!("bound {} event(s) on {} -> {seam_id}", symbols.len(), file.file);
    Ok((filed, warned))
}

fn new_seam(id: &str, file: &str, verdict: Option<&str>) -> ddd_core::seam::SeamDeclaration {
    ddd_core::seam::SeamDeclaration {
        format: 2,
        id: id.to_string(),
        boundary: format!("contract surface of {file}"),
        verdict_knowledge: verdict.unwrap_or_default().to_string(),
        contract_location: file.to_string(),
        obligations: Vec::new(),
        metadata: Default::default(),
        bindings: Vec::new(),
        notes: None,
    }
}

/// `ddd-core/src/lib.rs` → `ddd-core-src-lib-rs` (one token per file).
fn slug(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect()
}
