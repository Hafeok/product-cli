//! The CI gate, plus the join a restructuring work list comes from.

use std::path::Path;

use clap::Args;
use product_core::error::Result;
use serde_json::json;
use spec_core::gate;
use spec_core::map;
use spec_core::store;

use crate::exit;
use crate::render::Report;

#[derive(Args)]
pub struct CheckArgs {
    /// Verdicts only; the exit code is the output.
    #[arg(long)]
    pub ci: bool,
    /// Everything, with the candidate set and the delta beneath the verdicts.
    #[arg(long)]
    pub assessment: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct MapArgs {
    #[arg(long)]
    pub json: bool,
}

/// The CI gate.
///
/// One computed set, selected differently — never three report modes that
/// drift. `--ci` prints the verdicts, the default adds the metrics beneath
/// them, and `--assessment` adds the candidate set and the delta.
pub fn check(root: &Path, args: &CheckArgs) -> Result<Report> {
    let spec = store::load_store(root)?;
    let findings = gate::judge_store(&spec);
    let code = if findings.is_empty() { exit::CONFORMANT } else { exit::FINDINGS };

    let verdicts = findings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
    let text = if args.ci {
        verdicts
    } else {
        let mut sections = Vec::new();
        if !verdicts.is_empty() {
            sections.push(verdicts);
        }
        sections.push(metrics(&spec));
        if args.assessment {
            sections.push(assessment(&spec));
        }
        sections.join("\n\n")
    };

    let body = args.json.then(|| json!({
        "findings": findings.iter().map(|f| json!({
            "class": f.class.to_string(),
            "subject": f.record,
            "message": f.message,
        })).collect::<Vec<_>>(),
        "metrics": metric_values(&spec),
    }));
    Ok(Report::text(code, text).with_json(body))
}

/// The act ↔ entry point join.
pub fn map(root: &Path, args: &MapArgs) -> Result<Report> {
    let spec = store::load_store(root)?;
    let findings = map::join(&spec);
    let text = if findings.is_empty() {
        "no disagreements between acts and entry points".to_string()
    } else {
        findings.iter().map(map::MapFinding::line).collect::<Vec<_>>().join("\n")
    };
    let body = args.json.then(|| json!(findings));
    Ok(Report::text(exit::CONFORMANT, text).with_json(body))
}

/// The reported figures.
///
/// Reported, never gated. A codebase with unspecified regions is unspecified,
/// not non-conformant, and a flow that made the unmapped count fall quickly
/// would be doing the wrong work fast.
fn metrics(spec: &gate::SpecStore) -> String {
    let values = metric_values(spec);
    let lines = values
        .as_object()
        .map(|o| {
            o.iter()
                .map(|(k, v)| format!("  {k:<26} {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    format!("metrics (reported, not gated):\n{lines}")
}

fn metric_values(spec: &gate::SpecStore) -> serde_json::Value {
    let entry_points = spec.inventory.as_ref().map_or(0, |i| i.entry_points.len());
    let candidates = spec.inventory.as_ref().map_or(0, |i| i.candidates.len());
    let unreviewed = spec
        .inventory
        .as_ref()
        .map_or(0, |i| i.candidates.iter().filter(|c| spec.is_unreviewed(&c.id)).count());
    let mapped = spec
        .inventory
        .as_ref()
        .map_or(0, |i| i.entry_points.iter().filter(|e| !spec.acts_at(&e.id).is_empty()).count());
    json!({
        "entry_points": entry_points,
        "candidates": candidates,
        "candidates_unreviewed": unreviewed,
        "acts_ratified": spec.acts.len(),
        "candidates_refused": spec.rejections.len(),
        "entry_points_mapped": mapped,
        "mapping_coverage": coverage(mapped, entry_points),
        "records_open": spec.records.iter().filter(|r| r.is_open()).count(),
    })
}

/// Coverage as a percentage, or `null` where there is nothing to cover.
///
/// Null rather than 100%: an empty codebase is not fully specified, and a
/// figure that says otherwise is the kind of flattering default that makes a
/// metric useless.
fn coverage(mapped: usize, total: usize) -> serde_json::Value {
    if total == 0 {
        return serde_json::Value::Null;
    }
    #[allow(clippy::cast_precision_loss)]
    let ratio = (mapped as f64 / total as f64) * 100.0;
    json!(format!("{ratio:.0}%"))
}

/// The assessment surface: the candidate set and the delta, by cluster.
fn assessment(spec: &gate::SpecStore) -> String {
    let joined = map::join(spec);
    let list = if joined.is_empty() {
        "  (none)".to_string()
    } else {
        joined.iter().map(|f| format!("  {}", f.line())).collect::<Vec<_>>().join("\n")
    };
    format!("act ↔ entry point:\n{list}")
}
