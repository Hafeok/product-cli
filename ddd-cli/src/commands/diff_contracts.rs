//! `ddd diff-contracts <base>..<head>` — repo-diff contract classification.

use std::path::PathBuf;
use std::time::Duration;

use ddd_lsp::HostManager;
use product_core::error::{ProductError, Result};

pub fn run(root: Option<PathBuf>, range: String, json: bool, wait: u64) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let (base, head) = parse_range(&range)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let mut manager = HostManager::new(repo_root);
    let report =
        ddd_lsp::revdiff::diff_contracts(&mut manager, base, head, Duration::from_secs(wait))
            .map_err(ProductError::ConfigError)?;
    let undischarged = ddd_core::contracts::check_discharge(&report, &store.seams);
    if json {
        let out = serde_json::json!({
            "report": report,
            "undischarged": undischarged,
        });
        let text = serde_json::to_string_pretty(&out)
            .map_err(|e| ProductError::IoError(e.to_string()))?;
        println!("{text}");
    } else {
        render(&report, &undischarged);
    }
    if !report.skipped.is_empty() {
        return Err(ProductError::ConfigError(format!(
            "{} file(s) could not be classified — a skipped file is not a clean file",
            report.skipped.len()
        )));
    }
    if !undischarged.is_empty() {
        return Err(ProductError::ConfigError(format!(
            "{} undischarged contract-surface change(s) — file the covering declarations (signed bindings) or the change stands as a governance escape",
            undischarged.len()
        )));
    }
    Ok(())
}

/// `a..b` → (a, Some(b)); `a` → (a, None) = base against the working tree.
fn parse_range(range: &str) -> Result<(&str, Option<&str>)> {
    match range.split_once("..") {
        Some((a, b)) if a.is_empty() || b.is_empty() => Err(ProductError::ConfigError(format!(
            "malformed range `{range}` — expected <base>..<head> or <base>"
        ))),
        Some((a, b)) => Ok((a, Some(b))),
        None => Ok((range, None)),
    }
}

fn render(
    report: &ddd_core::contracts::ContractDiffReport,
    undischarged: &[ddd_core::contracts::Undischarged],
) {
    println!(
        "== contract surface: {}..{} ==",
        &report.base[..12.min(report.base.len())],
        report.head_short()
    );
    let mut surface = 0usize;
    for file in &report.files {
        let events: Vec<_> = file.events.iter().filter(|e| e.event.surface).collect();
        if events.is_empty() {
            continue;
        }
        println!("{} ({})", file.file, file.language);
        for e in &events {
            let rule = e.event.rule.unwrap_or("-");
            let state = if undischarged.iter().any(|u| u.id == e.id) {
                "UNDISCHARGED"
            } else {
                "discharged"
            };
            println!("  {state}  {}  [{rule}]", e.id);
        }
        surface += events.len();
    }
    for s in &report.skipped {
        println!("skipped: {} — {}", s.file, s.reason);
    }
    for u in undischarged {
        println!("undischarged: {} — {}", u.id, u.reason);
    }
    if surface == 0 && report.skipped.is_empty() {
        println!("no contract-surface changes");
    } else {
        println!(
            "{surface} contract-surface event(s), {} undischarged",
            undischarged.len()
        );
    }
}
