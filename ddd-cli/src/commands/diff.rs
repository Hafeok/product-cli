//! `ddd diff` — declared vs. detected governance state (CI gate).

use std::path::PathBuf;

use ddd_core::config::FindingSeverity;
use product_core::error::{ProductError, Result};

pub fn run(root: Option<PathBuf>, sarif: Vec<PathBuf>) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let detected = ddd_core::detect::detect(&repo_root, store.config.as_ref(), &sarif);
    let report = ddd_core::diff::diff(&store, &detected);
    for f in &report.findings {
        let tag = if f.severity == FindingSeverity::Warn { " (warning)" } else { "" };
        println!("{} {}/{} — {}{}", f.kind.label(), f.namespace, f.rule_id, f.detail, tag);
    }
    for n in &report.notes {
        println!("note: {n}");
    }
    let blocking = report.blocking();
    if blocking == 0 {
        println!(
            "governance diff clean — {} rule(s) detected, {} finding(s), {} note(s)",
            detected.rules.len(),
            report.findings.len(),
            report.notes.len()
        );
        return Ok(());
    }
    Err(ProductError::ConfigError(format!(
        "{blocking} governance escape(s) — every diagnostic must resolve through `ddd why` or be filed"
    )))
}
