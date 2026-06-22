//! `product domain data` — data conformance with a divergence-rate trend.
//!
//! Runs the §6.3 data-conformance engine over one or all production datasets,
//! then records each run's divergence rate to a per-product history so the
//! **trend** (rising / falling / stable) is surfaced — the §3.1 spec-staleness
//! signal "made visible as it happens" (§13.3). The pure engine lives in
//! `pf::data_check`; this adapter owns the file I/O and the report.

use std::path::{Path, PathBuf};

use product_core::pf::data_check::{check_dataset, classify_trend, DataVerdict, DivergenceTrend};
use serde::{Deserialize, Serialize};

use super::domain::{load, resolve};
use super::BoxResult;

/// One recorded run in the divergence-rate history.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    dataset: String,
    at: String,
    divergence_rate: f64,
    total: usize,
    violating: usize,
}

/// Validate one or all production datasets, reporting the divergence rate and
/// its trend. Exit 1 if any record diverges.
pub(super) fn run(dataset: Option<String>, product: Option<String>, no_record: bool) -> BoxResult {
    let (_, dir) = resolve(product)?;
    let g = load(&dir)?.graph;
    let targets: Vec<String> = match dataset {
        Some(id) => vec![id],
        None => g.production_datasets.iter().map(|d| d.id.clone()).collect(),
    };
    if targets.is_empty() {
        println!("no production datasets declared — author one with `product domain new production-dataset …`");
        return Ok(());
    }
    let history = read_history(&dir);
    let mut diverged = false;
    for id in &targets {
        let ds = g.production_datasets.iter().find(|d| &d.id == id)
            .ok_or_else(|| format!("no production dataset {id:?} in the graph"))?;
        let records = read_records(&ds.source)?;
        let verdict = check_dataset(&g, id, &records)?;
        let previous = last_rate(&history, id);
        print_verdict(&verdict, classify_trend(previous, verdict.divergence_rate));
        if !no_record {
            append_history(&dir, &verdict)?;
        }
        if !verdict.conformant() {
            diverged = true;
        }
    }
    if diverged {
        return Err("data conformance: some records diverge from the declared shape \
                    — fix the data, or (if the spec is stale) fix the shape".into());
    }
    Ok(())
}

/// Read a dataset source: a JSON array of record objects.
fn read_records(source: &str) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(source)
        .map_err(|e| format!("could not read dataset source {source:?}: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("dataset source {source:?} is not valid JSON: {e}"))?;
    match value {
        serde_json::Value::Array(rows) => Ok(rows),
        _ => Err(format!("dataset source {source:?} must be a JSON array of records").into()),
    }
}

/// Print a verdict: the divergence rate + trend, then per-record findings.
fn print_verdict(v: &DataVerdict, trend: DivergenceTrend) {
    println!("Data conformance for {} (shape {}, target {}):", v.dataset, v.shape, v.target);
    println!(
        "  {} record(s): {} conforming, {} violating — divergence rate {:.1}% ({})",
        v.total, v.conforming, v.violating, v.divergence_rate * 100.0, trend.marker()
    );
    for f in &v.findings {
        println!("  ✗ record {} [{}] {}: {}", f.record, f.field, f.kind, f.detail);
    }
    if v.conformant() {
        println!("  ✓ all records conform to the shape");
    }
}

/// The history file path for a product (sibling to `session.json`).
fn history_path(dir: &Path) -> PathBuf {
    dir.join("data-history.jsonl")
}

/// Read the recorded history (best-effort — a missing or unreadable line is skipped).
fn read_history(dir: &Path) -> Vec<HistoryEntry> {
    let Ok(text) = std::fs::read_to_string(history_path(dir)) else { return Vec::new() };
    text.lines().filter_map(|l| serde_json::from_str(l).ok()).collect()
}

/// The most recent recorded divergence rate for a dataset, if any.
fn last_rate(history: &[HistoryEntry], dataset: &str) -> Option<f64> {
    history.iter().rev().find(|e| e.dataset == dataset).map(|e| e.divergence_rate)
}

/// Append this run to the history as one JSON line.
fn append_history(dir: &Path, v: &DataVerdict) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let entry = HistoryEntry {
        dataset: v.dataset.clone(),
        at: chrono::Utc::now().to_rfc3339(),
        divergence_rate: v.divergence_rate,
        total: v.total,
        violating: v.violating,
    };
    let line = format!("{}\n", serde_json::to_string(&entry)?);
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(history_path(dir))?;
    f.write_all(line.as_bytes())?;
    Ok(())
}
