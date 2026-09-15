//! The CI gate, plus the join a restructuring work list comes from.
//!
//! Two classes of verdict, reported apart so a reader can always tell which is
//! which. Structural verdicts say something is broken and are not
//! configurable. Policy verdicts are the project's own, each with a threshold,
//! a basis and a principal.

use std::path::Path;

use clap::Args;
use product_core::error::Result;
use serde_json::json;
use spec_core::{gate, map, metrics, store};

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
/// them, and `--assessment` adds the join.
pub fn check(root: &Path, args: &CheckArgs) -> Result<Report> {
    let spec = store::load_store(root)?;
    let structural = gate::judge_store(&spec);
    let policy = gate::run_policy(&spec);
    let observed = metrics::compute(&spec);

    let failed = !structural.is_empty() || !policy.is_empty();
    let code = if failed { exit::FINDINGS } else { exit::CONFORMANT };

    let text = render(args, &structural, &policy, &observed, &spec);
    let body = args.json.then(|| json!({
        "structural": structural.iter().map(|f| json!({
            "class": f.class.to_string(),
            "subject": f.record,
            "message": f.message,
        })).collect::<Vec<_>>(),
        "policy": policy,
        "metrics": observed,
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

fn render(
    args: &CheckArgs,
    structural: &[spec_core::check::Finding],
    policy: &[spec_core::policy_check::PolicyFinding],
    observed: &std::collections::BTreeMap<String, metrics::Metric>,
    spec: &gate::SpecStore,
) -> String {
    let verdicts = verdict_lines(structural, policy);
    if args.ci {
        return verdicts;
    }

    let mut sections = Vec::new();
    if !verdicts.is_empty() {
        sections.push(verdicts);
    }
    sections.push(metric_section(observed, spec));
    if args.assessment {
        sections.push(assessment(spec));
    }
    sections.join("\n\n")
}

fn verdict_lines(
    structural: &[spec_core::check::Finding],
    policy: &[spec_core::policy_check::PolicyFinding],
) -> String {
    let mut lines: Vec<String> = structural.iter().map(ToString::to_string).collect();
    lines.extend(policy.iter().map(ToString::to_string));
    lines.join("\n")
}

/// The reported figures, each marked with whether anything gates it.
///
/// A metric with no verdict attached is a metric, and saying so beside the
/// number is what keeps the two from being confused at a glance.
fn metric_section(
    observed: &std::collections::BTreeMap<String, metrics::Metric>,
    spec: &gate::SpecStore,
) -> String {
    let gated: Vec<&str> = spec
        .policy_in_force()
        .ok()
        .flatten()
        .map(|p| p.policy_verdicts.iter().map(|v| v.metric.as_str()).collect())
        .unwrap_or_default();

    let lines = observed
        .iter()
        .map(|(name, metric)| {
            let mark = if gated.contains(&name.as_str()) { "gated" } else { "" };
            format!("  {name:<24} {:<16} {mark}", metric.render())
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("metrics (reported; only those marked `gated` can fail a build):\n{lines}")
}

/// The assessment surface: the join, by cluster.
fn assessment(spec: &gate::SpecStore) -> String {
    let joined = map::join(spec);
    let list = if joined.is_empty() {
        "  (none)".to_string()
    } else {
        joined.iter().map(|f| format!("  {}", f.line())).collect::<Vec<_>>().join("\n")
    };
    format!("act ↔ entry point:\n{list}")
}
