//! Text renderers for the builder — status output, `add` output, etc.
//!
//! These functions produce the strings the CLI prints; they never touch
//! stdout directly.

use super::status::{StatusReport, StatusRow};

pub fn render_status(r: &StatusReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Draft — type: {}  (reason: {})\n",
        r.kind,
        if r.reason.trim().is_empty() {
            "(empty — set with `product request edit` or submit --force will be blocked)"
        } else {
            r.reason.trim()
        }
    ));
    out.push('\n');
    if r.rows.is_empty() {
        out.push_str("  (no artifacts yet — `product request add feature|adr|tc|dep|doc`)\n");
    } else {
        for row in &r.rows {
            render_row(&mut out, row);
        }
    }
    out.push('\n');
    out.push_str(&format!("Errors:   {}\n", r.error_count));
    out.push_str(&format!("Warnings: {}\n", r.warning_count));
    if !r.all_codes.is_empty() {
        out.push_str(&format!("Codes:    {}\n", r.all_codes.join(", ")));
    }
    out
}

fn render_row(out: &mut String, row: &StatusRow) {
    let ref_part = row
        .ref_name
        .as_deref()
        .map(|r| format!(" ref:{r}"))
        .unwrap_or_default();
    out.push_str(&format!(
        "  {}  [{}]{}  {}\n",
        row.glyph, row.artifact_type, ref_part, row.title,
    ));
    for (k, v) in &row.key_fields {
        out.push_str(&format!("      {k}: {v}\n"));
    }
    if !row.codes.is_empty() {
        out.push_str(&format!("      findings: {}\n", row.codes.join(", ")));
    }
}

/// Render a short one-liner for `product request add ...` output.
pub fn render_added(refs: &[String], note: Option<&str>, warn_codes: &[String]) -> String {
    let mut out = String::new();
    let joined = refs.join(", ");
    out.push_str(&format!("Appended: {joined}\n"));
    if let Some(n) = note {
        out.push_str(n);
        out.push('\n');
    }
    if !warn_codes.is_empty() {
        out.push_str(&format!("Warnings: {}\n", warn_codes.join(", ")));
    }
    out
}
