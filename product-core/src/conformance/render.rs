//! Text rendering of Two Pillars conformance reports (ADR-052).

use super::{ClauseMode, ConformanceFinding, ConformanceReport};

/// Render a conformance report as the human-readable text view.
pub fn render_report_text(report: &ConformanceReport, system_name: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Two Pillars conformance \u{2014} {} (spec {}, {})\n\n",
        system_name, report.spec, report.scope
    ));
    render_clause_table(report, &mut out);
    render_findings(&report.findings, &mut out);
    render_summary(report, &mut out);
    out
}

fn render_clause_table(report: &ConformanceReport, out: &mut String) {
    out.push_str("Clauses:\n");
    for clause in &report.clauses {
        let mark = match (clause.passed, clause.findings) {
            (true, 0) => "pass",
            (true, _) => "warn",
            (false, _) => "FAIL",
        };
        let mode = match clause.mode {
            ClauseMode::ByConstruction => " (by construction)",
            ClauseMode::Checked => "",
        };
        let count = if clause.findings > 0 {
            format!(" \u{2014} {} finding(s)", clause.findings)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "  [{}] {:<14} {}{}{}\n",
            mark, clause.clause, clause.title, mode, count
        ));
    }
}

fn render_findings(findings: &[ConformanceFinding], out: &mut String) {
    if findings.is_empty() {
        return;
    }
    out.push_str("\nFindings:\n");
    for f in findings {
        let artifact = f.artifact.as_deref().unwrap_or("project");
        out.push_str(&format!(
            "  [{}] {} {} \u{2014} {}\n",
            f.severity, f.clause, artifact, f.description
        ));
        out.push_str(&format!("    Action: {}\n", f.suggested_action));
    }
}

fn render_summary(report: &ConformanceReport, out: &mut String) {
    out.push_str(&format!(
        "\nSummary: {}/{} clauses passed; {} violation(s), {} advisory(ies)\n",
        report.summary.clauses_passed,
        report.summary.clauses_checked,
        report.summary.violations,
        report.summary.advisories,
    ));
    let verdict = if report.has_violations() {
        "does not conform to Level 3 (spec-driven) \u{2014} checkable subset"
    } else {
        "conforms to Level 3 (spec-driven) \u{2014} checkable subset"
    };
    out.push_str(&format!("Verdict: {}\n", verdict));
}
