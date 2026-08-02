//! `ddd validate` — schema + ontology validation with a whole-store report.

use std::path::PathBuf;

use product_core::error::{ProductError, Result};

pub fn run(root: Option<PathBuf>) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let violations = ddd_core::validate::validate_store(&store);
    let blocking = ddd_core::validate::blocking(&violations);
    let warnings = violations.len() - blocking;
    if blocking == 0 {
        for v in &violations {
            println!("warning: [{}] {}: {}", v.focus, v.path, v.message);
        }
        println!(
            "conformant — {} entr{}, {} warning(s)",
            store.entry_count(),
            if store.entry_count() == 1 { "y" } else { "ies" },
            warnings
        );
        return Ok(());
    }
    eprintln!("non-conformant — {blocking} violation(s), {warnings} warning(s):");
    for v in &violations {
        let tag = if v.severity == "warning" { " (warning)" } else { "" };
        eprintln!("  - [{}] {}: {}{}", v.focus, v.path, v.message, tag);
    }
    Err(ProductError::ConfigError(format!("{blocking} conformance violation(s)")))
}
