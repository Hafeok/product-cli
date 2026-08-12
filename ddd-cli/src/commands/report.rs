//! `ddd report escapes` — the three-section escaped-decision report.

use std::path::PathBuf;

use ddd_core::config::FindingSeverity;
use ddd_core::report::EscapesReport;
use product_core::error::{ProductError, Result};

pub fn run(
    root: Option<PathBuf>,
    sarif: Vec<PathBuf>,
    today: Option<String>,
    json: bool,
) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let detected = ddd_core::detect::detect(&repo_root, store.config.as_ref(), &sarif);
    let today = today.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let report = ddd_core::report::report_escapes(&store, &detected, &today);
    let config = store.config.clone().unwrap_or_default();
    let pair = ddd_lsp::adapter::htmlcss_pair::check_pairs(&repo_root, &config);
    if json {
        let out = serde_json::json!({
            "today": today,
            "clean": report.is_clean() && pair.is_clean(),
            "escapes": report,
            "pair": pair,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| ProductError::IoError(e.to_string()))?
        );
        return Ok(());
    }
    print_report(&report, &pair, &today);
    Ok(())
}

fn print_report(
    report: &EscapesReport,
    pair: &ddd_lsp::adapter::htmlcss_pair::PairReport,
    today: &str,
) {
    print_diff(report);
    print_cadence(report, today);
    print_basis(report);
    print_pair(pair);
    if report.is_clean() && pair.is_clean() {
        println!("\nno escaped decisions — every governed diagnostic resolves");
    } else {
        let n = report.diff.findings.len()
            + report.cadence.len()
            + report.basis_loss.len()
            + pair.findings.len();
        println!("\n{n} escape(s) — file the missing entries or revalidate the claims");
    }
}

/// The M7 pair contract: orphan classes plus dead selectors, with the
/// check's own coverage stated (dec/ddd/report-coverage-explicit).
fn print_pair(pair: &ddd_lsp::adapter::htmlcss_pair::PairReport) {
    if pair.coverage.units == 0 {
        return;
    }
    println!("\n== html+css pair contract ==");
    if pair.is_clean() {
        println!(
            "clean — {} unit(s), {} selector(s) checked",
            pair.coverage.units, pair.coverage.selectors_checked
        );
    }
    for f in &pair.findings {
        println!("{} `{}` — {}", f.kind, f.name, f.detail);
    }
    if !pair.coverage.selectors_skipped.is_empty() {
        println!(
            "not checkable: {} selector(s) outside the modelled subset — {}",
            pair.coverage.selectors_skipped.len(),
            join_capped(&pair.coverage.selectors_skipped)
        );
    }
}

fn print_diff(report: &EscapesReport) {
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
}

fn print_cadence(report: &EscapesReport, today: &str) {
    let cov = &report.cadence_coverage;
    println!("\n== revalidation cadence (today: {today}) ==");
    if report.cadence.is_empty() {
        println!("clean — {} live claim(s) carry a revalidate_by", cov.dated);
    }
    for c in &report.cadence {
        println!(
            "claim {} [{}] — revalidate_by {} has passed",
            c.claim,
            c.status.as_str(),
            c.revalidate_by
        );
    }
    if !cov.undated.is_empty() {
        println!(
            "not checkable: {} live claim(s) carry no revalidate_by — {}",
            cov.undated.len(),
            join_capped(&cov.undated)
        );
    }
}

fn print_basis(report: &EscapesReport) {
    let cov = &report.basis_coverage;
    println!("\n== basis loss ==");
    if report.basis_loss.is_empty() {
        println!("clean — {} pinned basedOn edge(s) checked", cov.pinned);
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
    if !cov.unpinned.is_empty() {
        let ids: Vec<String> = cov.unpinned.iter().map(|u| u.decision.clone()).collect();
        println!(
            "not checkable: {} unpinned edge(s) — {} (pin them with format-2 based_on edges)",
            cov.unpinned.len(),
            join_capped(&ids)
        );
    }
}

/// Name the uncheckable set, but keep one long list from swamping the
/// report: past six, name six and count the rest.
fn join_capped(ids: &[String]) -> String {
    const CAP: usize = 6;
    if ids.len() <= CAP {
        return ids.join(", ");
    }
    format!("{}, and {} more", ids[..CAP].join(", "), ids.len() - CAP)
}
