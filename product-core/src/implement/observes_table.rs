//! Build the TC `observes:` surface table for the implement bundle (FT-074).
//!
//! Pure helper: given a knowledge graph and a feature id, return a vector of
//! `(tc_id, observes)` pairs for every linked TC. The implement pipeline
//! consumes this list to inject `observes:` lines adjacent to each TC body
//! in the rendered bundle (per ADR-051: tests must assert against named
//! surfaces, not on response envelopes alone).

use crate::graph::KnowledgeGraph;

/// One row of the observes table — a TC id and its declared surfaces.
#[derive(Debug, Clone)]
pub struct ObservesRow {
    pub tc_id: String,
    pub observes: Vec<String>,
}

/// Build the observes table for every TC linked to `feature_id`.
///
/// Order matches the feature's declared `tests:` list. TCs that exist on
/// disk but cannot be loaded are skipped. TCs with empty `observes:` are
/// included with an empty vector so the caller can render an explicit
/// warning per ADR-051 (belt-and-braces alongside FT-072's graph-check
/// gate).
pub fn build_observes_table(graph: &KnowledgeGraph, feature_id: &str) -> Vec<ObservesRow> {
    let feature = match graph.features.get(feature_id) {
        Some(f) => f,
        None => return Vec::new(),
    };
    let mut rows = Vec::with_capacity(feature.front.tests.len());
    for tc_id in &feature.front.tests {
        if let Some(tc) = graph.tests.get(tc_id.as_str()) {
            rows.push(ObservesRow {
                tc_id: tc.front.id.clone(),
                observes: tc.front.observes.clone(),
            });
        }
    }
    rows
}

/// Render a single `observes:` line for a TC. Returns an empty string for
/// TCs whose linked feature does not require `observes:` (matched via an
/// optional list of allowed types).
///
/// The line is rendered as ``observes: [a, b]`` so it scans the same way as
/// the source YAML key the agent will edit. The leading marker `**` makes
/// it visually distinct inside the markdown body so an agent reading the
/// bundle sees it as a callout, not body prose.
pub fn render_observes_line(observes: &[String]) -> String {
    if observes.is_empty() {
        // ADR-051 inline warning — belt-and-braces alongside FT-072's
        // graph-check gate. A required-type TC with empty observes should
        // never reach this point, but if it does, the line nudges the
        // implementing agent.
        return String::from(
            "**observes:** [] <!-- WARNING: TC missing observes per ADR-051 -->",
        );
    }
    format!("**observes:** [{}]", observes.join(", "))
}

/// Inject `observes:` lines adjacent to each TC body in a rendered bundle.
///
/// Scans the bundle text line-by-line for `### TC-XXX — ...` headings and
/// inserts the corresponding observes line immediately after the blank line
/// that follows the heading. TCs not present in the table are left
/// unchanged so this transform is safe on bundles produced for other
/// purposes.
pub fn inject_observes_inline(bundle: &str, rows: &[ObservesRow]) -> String {
    if rows.is_empty() {
        return bundle.to_string();
    }
    let lookup: std::collections::HashMap<&str, &[String]> = rows
        .iter()
        .map(|r| (r.tc_id.as_str(), r.observes.as_slice()))
        .collect();
    let lines: Vec<&str> = bundle.split('\n').collect();
    let mut out = String::with_capacity(bundle.len() + rows.len() * 64);
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        out.push_str(line);
        if i + 1 < lines.len() {
            out.push('\n');
        }
        if let Some(tc_id) = extract_tc_id_from_heading(line) {
            if let Some(observes) = lookup.get(tc_id.as_str()) {
                // Consume the blank line that follows the heading, emit the
                // observes line, then re-emit the blank line so the TC body
                // remains visually separated.
                if i + 1 < lines.len() && lines[i + 1].is_empty() {
                    out.push_str(lines[i + 1]);
                    out.push('\n');
                    out.push_str(&render_observes_line(observes));
                    out.push('\n');
                    i += 2;
                    continue;
                } else {
                    // No blank line — still emit the observes line so the
                    // assertion shape is visible.
                    out.push_str(&render_observes_line(observes));
                    if i + 1 < lines.len() {
                        out.push('\n');
                    }
                }
            }
        }
        i += 1;
    }
    out
}

/// Extract a TC id from a heading of the form ``### TC-XXX — Title (type)``.
/// Returns `None` when the line is not a recognised TC heading.
fn extract_tc_id_from_heading(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("### ") {
        return None;
    }
    let after = &trimmed[4..];
    // Token before the first space (or em-dash separator) is the id.
    let token = after.split([' ', '\t']).next().unwrap_or("");
    if token.starts_with("TC-") && token.len() > 3 {
        Some(token.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_empty_observes_emits_warning() {
        let line = render_observes_line(&[]);
        assert!(line.contains("WARNING"));
        assert!(line.contains("ADR-051"));
    }

    #[test]
    fn render_observes_line_formats_csv() {
        let line = render_observes_line(&[
            "file".to_string(),
            "graph".to_string(),
        ]);
        assert_eq!(line, "**observes:** [file, graph]");
    }

    #[test]
    fn extract_tc_id_from_recognised_heading() {
        let id = extract_tc_id_from_heading("### TC-847 — Some title (scenario)");
        assert_eq!(id, Some("TC-847".to_string()));
    }

    #[test]
    fn extract_tc_id_rejects_non_tc_headings() {
        assert_eq!(extract_tc_id_from_heading("### PAT-001 — pattern"), None);
        assert_eq!(extract_tc_id_from_heading("## Patterns"), None);
        assert_eq!(extract_tc_id_from_heading("plain text"), None);
    }

    #[test]
    fn inject_observes_inserts_after_heading_blank_line() {
        let bundle = "## Test Criteria\n\n### TC-001 — Sample (scenario)\n\nTC body.\n";
        let rows = vec![ObservesRow {
            tc_id: "TC-001".to_string(),
            observes: vec!["file".to_string()],
        }];
        let out = inject_observes_inline(bundle, &rows);
        assert!(out.contains("### TC-001"));
        assert!(out.contains("**observes:** [file]"));
        // The injection sits between the heading and the body — find the
        // heading offset and assert the observes line appears before the
        // body.
        let heading_pos = out.find("### TC-001").expect("heading");
        let observes_pos = out.find("**observes:**").expect("observes line");
        let body_pos = out.find("TC body.").expect("body");
        assert!(heading_pos < observes_pos);
        assert!(observes_pos < body_pos);
    }

    #[test]
    fn inject_observes_leaves_unknown_tcs_untouched() {
        let bundle = "### TC-999 — Unknown (scenario)\n\nBody.\n";
        let rows = vec![ObservesRow {
            tc_id: "TC-001".to_string(),
            observes: vec!["file".to_string()],
        }];
        let out = inject_observes_inline(bundle, &rows);
        // No observes line was inserted for TC-999.
        assert!(!out.contains("**observes:**"));
    }

    #[test]
    fn inject_observes_no_op_when_table_empty() {
        let bundle = "### TC-001 — Sample (scenario)\n\nBody.\n";
        let out = inject_observes_inline(bundle, &[]);
        assert_eq!(out, bundle);
    }
}
