//! Pure text renderers for status summaries.

use super::summary::{FeatureList, FeatureRow, PhaseSummary, ProjectSummary};
use std::fmt::Write;

/// Render the full project summary to a single text string.
///
/// `show_exit_criteria` controls whether per-phase exit-criteria detail is
/// included (currently emitted when the user filters to a single phase).
pub fn render_project_summary_text(summary: &ProjectSummary, show_exit_criteria: bool) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "Project Status: {}", summary.project);
    let _ = writeln!(&mut out, "=================");
    let _ = writeln!(&mut out);

    for phase in &summary.phases {
        render_phase_into(&mut out, phase, show_exit_criteria);
    }

    out
}

/// Render a filter-result feature list to text with a header line.
pub fn render_feature_list_text(heading: &str, list: &FeatureList) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "{}", heading);
    for row in &list.items {
        let _ = writeln!(
            &mut out,
            "  {} — {} (phase {})",
            row.id, row.title, row.phase
        );
    }
    out
}

fn render_phase_into(out: &mut String, phase: &PhaseSummary, show_exit_criteria: bool) {
    let gate_label = if phase.gate.is_open {
        "[OPEN]".to_string()
    } else {
        format!(
            "[LOCKED \u{2014} exit criteria not passing: {}]",
            phase.gate.failing_exit_criteria.join(", ")
        )
    };
    let _ = writeln!(
        out,
        "Phase {} \u{2014} {} ({}/{} complete)  {}",
        phase.phase, phase.name, phase.complete, phase.total, gate_label
    );

    if show_exit_criteria && !phase.gate.exit_criteria.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "  Exit criteria:");
        for tc in &phase.gate.exit_criteria {
            let mark = if tc.passing {
                "passing  \u{2713}"
            } else {
                "failing  \u{2717}"
            };
            let _ = writeln!(out, "    {}  {}  [{}]", tc.id, tc.title, mark);
        }
        let _ = writeln!(out);
    }

    for row in &phase.features {
        render_feature_row_into(out, row);
    }
    let _ = writeln!(out);
}

fn render_feature_row_into(out: &mut String, row: &FeatureRow) {
    let marker = match row.status.as_str() {
        "complete" => "[x]",
        "in-progress" => "[~]",
        "planned" => "[ ]",
        "abandoned" => "[-]",
        _ => "[?]",
    };
    // FT-053 / ADR-045: append due-date cell when present; flag overdue with `!`.
    let due_suffix = match (&row.due_date, row.overdue) {
        (Some(d), true) => format!("  due {} \u{203C} overdue", d),
        (Some(d), false) => format!("  due {}", d),
        (None, _) => String::new(),
    };
    let _ = writeln!(
        out,
        "  {} {:<15} {} (tests: {}/{}){}",
        marker, row.id, row.title, row.tests_passing, row.tests_total, due_suffix
    );
}
