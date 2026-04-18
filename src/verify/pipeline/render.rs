//! Pretty + JSON rendering for pipeline results (FT-044).

use super::types::{Finding, PipelineResult, StageResult};

/// Render the result as human-readable pretty output (no ANSI colour).
pub fn render_pretty(result: &PipelineResult) -> String {
    let mut out = String::new();
    out.push_str("product verify\n\n");
    for s in &result.stages {
        render_stage(&mut out, s);
    }
    render_summary_footer(&mut out, result);
    out
}

/// Render the result as the documented `--ci` JSON schema.
pub fn render_json(result: &PipelineResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".into())
}

fn render_stage(out: &mut String, s: &StageResult) {
    let icon = s.status.icon();
    let name_padded = format!("{:<20}", s.name);
    out.push_str(&format!(
        "  [{}/6] {} {}  {}\n",
        s.stage, name_padded, icon, s.summary
    ));
    for f in &s.findings {
        render_finding(out, f);
    }
}

fn render_finding(out: &mut String, f: &Finding) {
    match f {
        Finding::Code(c) => {
            out.push_str(&format!("              {}\n", c));
        }
        Finding::Tc { tc, feature, status, reason } => {
            let feat_str = feature.as_deref().unwrap_or("-");
            let marker = tc_marker(status);
            let bracket = reason
                .as_deref()
                .map(|r| format!("  [{}]", r))
                .unwrap_or_default();
            out.push_str(&format!(
                "              {}  {}  {}{}\n",
                tc, marker, feat_str, bracket
            ));
        }
    }
}

fn tc_marker(status: &str) -> &str {
    match status {
        "failing" => "FAIL",
        "passing" => "PASS",
        "unrunnable" | "unimplemented" | "skipped" => "SKIP",
        other => other,
    }
}

fn render_summary_footer(out: &mut String, result: &PipelineResult) {
    out.push_str("\n  ");
    out.push_str(&"-".repeat(64));
    out.push('\n');
    let overall = match result.exit {
        0 => "PASS",
        1 => "FAIL",
        2 => "PASS (with warnings)",
        _ => "UNKNOWN",
    };
    out.push_str(&format!("  Result:  {}\n", overall));
    out.push_str(&format!("  Exit:    {}\n", result.exit));
}
