//! `ddd render` — write the static HTML projection of the graph.

use std::path::PathBuf;

use product_core::error::Result;

pub fn run(
    root: Option<PathBuf>,
    out: Option<PathBuf>,
    sarif: Vec<PathBuf>,
    today: Option<String>,
) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let detected = ddd_core::detect::detect(&repo_root, store.config.as_ref(), &sarif);
    let today = today.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let escapes = ddd_core::report::report_escapes(&store, &detected, &today);
    let ledger = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d")
        .ok()
        .and_then(|date| ddd_core::ledger_gate::ledger_section(&repo_root, &store, date));
    let html =
        ddd_core::render::render_html_with_ledger(&store, &escapes, ledger.as_ref(), &today);
    let out = out.unwrap_or_else(|| repo_root.join(".ddd/render.html"));
    product_core::fileops::write_file_atomic(&out, &html)?;
    println!(
        "rendered {} entr{} -> {}",
        store.entry_count(),
        if store.entry_count() == 1 { "y" } else { "ies" },
        out.display()
    );
    Ok(())
}
