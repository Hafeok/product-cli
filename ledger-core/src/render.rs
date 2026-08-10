//! Rendering a verify report for a terminal.
//!
//! Kept beside the rules rather than in the CLI so the wording of a finding
//! is part of the format, not part of one front end. A gate whose message
//! does not say what to do next is a gate people route around.

use crate::verify::Report;

/// The human-readable report. Conformant runs still print their status
/// lines: pendency and a skipped blame pass are things a reader needs to
/// see, and silence would read as "everything was checked".
pub fn render(report: &Report) -> String {
    let mut lines = Vec::new();
    if report.is_conformant() {
        lines.push(format!(
            "conformant — {} entr{}, {} decision(s)",
            report.entries,
            if report.entries == 1 { "y" } else { "ies" },
            report.decisions
        ));
    } else {
        let total = report.findings.len() + report.graph.len();
        lines.push(format!("non-conformant — {total} finding(s):"));
        for f in &report.findings {
            lines.push(format!("  - [{}] {}: {}", f.class.code(), f.subject, f.message));
        }
        if !report.graph.is_empty() {
            lines.push(format!("graph stage — {} finding(s):", report.graph.len()));
            for g in &report.graph {
                lines.push(format!("  - [{}] {}: {}", g.class.code(), g.subject, g.message));
            }
        }
    }
    lines.extend(status_lines(report));
    lines.join("\n")
}

/// What the run saw but did not fail on.
fn status_lines(report: &Report) -> Vec<String> {
    let mut out = Vec::new();
    if !report.awaiting_acceptance.is_empty() {
        out.push(format!(
            "{} allocated, awaiting acceptance:",
            report.awaiting_acceptance.len()
        ));
        for id in &report.awaiting_acceptance {
            out.push(format!("  - {id}"));
        }
    }
    if report.blame_unavailable {
        out.push(
            "blame consistency (L009) skipped: no git repository here — the check runs where history exists"
                .to_string(),
        );
    } else if report.blame_uncommitted > 0 {
        out.push(format!(
            "blame consistency (L009) skipped for {} uncommitted acceptance(s) — checked once committed",
            report.blame_uncommitted
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{Finding, VerifyClass};

    fn report() -> Report {
        Report { entries: 3, decisions: 1, ..Report::default() }
    }

    #[test]
    fn a_clean_run_says_so_with_its_counts() {
        assert_eq!(render(&report()), "conformant — 3 entries, 1 decision(s)");
    }

    #[test]
    fn one_entry_is_not_pluralised() {
        let r = Report { entries: 1, ..report() };
        assert!(render(&r).starts_with("conformant — 1 entry,"));
    }

    #[test]
    fn findings_are_listed_under_a_count() {
        let r = Report {
            findings: vec![Finding::new(VerifyClass::L001, "dec:x/Y", "no allocation")],
            ..report()
        };
        let text = render(&r);
        assert!(text.starts_with("non-conformant — 1 finding(s):"), "{text}");
        assert!(text.contains("  - [L001] dec:x/Y: no allocation"), "{text}");
    }

    #[test]
    fn pendency_is_reported_even_on_a_clean_run() {
        let r = Report { awaiting_acceptance: vec!["dec:x/Y".into()], ..report() };
        let text = render(&r);
        assert!(text.contains("conformant"), "{text}");
        assert!(text.contains("1 allocated, awaiting acceptance:"), "{text}");
        assert!(text.contains("  - dec:x/Y"), "{text}");
    }

    #[test]
    fn a_skipped_blame_pass_is_never_silent() {
        let r = Report { blame_uncommitted: 2, ..report() };
        assert!(render(&r).contains("skipped for 2 uncommitted acceptance(s)"));
        let no_repo = Report { blame_unavailable: true, blame_uncommitted: 1, ..report() };
        assert!(render(&no_repo).contains("no git repository here"));
    }
}
