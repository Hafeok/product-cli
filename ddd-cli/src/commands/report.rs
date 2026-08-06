//! `ddd report escapes` — the three-section escaped-decision report.

use std::path::PathBuf;

use ddd_core::config::FindingSeverity;
use ddd_core::report::EscapesReport;
use product_core::error::Result;

pub fn run(root: Option<PathBuf>, sarif: Vec<PathBuf>, today: Option<String>) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let detected = ddd_core::detect::detect(&repo_root, store.config.as_ref(), &sarif);
    let today = today.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let report = ddd_core::report::report_escapes(&store, &detected, &today);
    print_report(&report, &today);
    Ok(())
}

fn print_report(report: &EscapesReport, today: &str) {
    println!("== governance diff ==");
    if report.diff.findings.is_empty() {
        println!("clean");
    }
    for f in &report.diff.findings {
        let tag = if f.severity == FindingSeverity::Warn { " (warning)" } else { "" };
        println!("{} {}/{} — {}{}", f.kind.label(), f.namespace, f.rule_id, f.detail, tag);
    }
    for n in &report.diff.notes {
        println!("note: {n}");
    }
    println!("\n== revalidation cadence (today: {today}) ==");
    if report.cadence.is_empty() {
        println!("clean");
    }
    for c in &report.cadence {
        println!(
            "claim {} [{}] — revalidate_by {} has passed",
            c.claim,
            c.status.as_str(),
            c.revalidate_by
        );
    }
    println!("\n== basis loss ==");
    if report.basis_loss.is_empty() {
        println!("clean");
    }
    for b in &report.basis_loss {
        let current = match (&b.current_status, &b.current_changed) {
            (Some(s), Some(ch)) => format!("now {}@{ch}", s.as_str()),
            _ => "claim no longer exists".to_string(),
        };
        println!(
            "decision {} — basedOn {} pinned {}@{}, {}",
            b.decision,
            b.claim,
            b.pinned_status.as_str(),
            b.pinned_changed,
            current
        );
    }
    if report.is_clean() {
        println!("\nno escaped decisions — every governed diagnostic resolves");
    } else {
        let n = report.diff.findings.len() + report.cadence.len() + report.basis_loss.len();
        println!("\n{n} escape(s) — file the missing entries or revalidate the claims");
    }
}
