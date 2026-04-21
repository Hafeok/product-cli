//! Status report for the current draft — per-artifact indicators, finding
//! counts, and a footer summary. Delegates all validation to `add::validate_draft`.

use super::add_helpers::validate_draft;
use super::draft::Draft;
use crate::config::ProductConfig;
use crate::request::Finding;
use serde_yaml::Value;

/// One row per artifact or change block in the draft.
pub struct StatusRow {
    pub index: usize,
    pub ref_name: Option<String>,
    pub artifact_type: String,
    pub title: String,
    pub key_fields: Vec<(String, String)>,
    pub glyph: char,
    pub codes: Vec<String>,
}

pub struct StatusReport {
    pub kind: String,
    pub reason: String,
    pub rows: Vec<StatusRow>,
    pub error_count: usize,
    pub warning_count: usize,
    pub all_codes: Vec<String>,
}

/// Build a status report for the draft.
pub fn status_report(
    draft: &Draft,
    config: &ProductConfig,
    graph: &crate::graph::KnowledgeGraph,
) -> StatusReport {
    let findings = validate_draft(draft, config, graph);
    let (errors, warnings): (Vec<_>, Vec<_>) = findings.iter().partition(|f| f.is_error());
    let error_count = errors.len();
    let warning_count = warnings.len();
    let all_codes: Vec<String> = findings.iter().map(|f| f.code.clone()).collect();

    let mut rows: Vec<StatusRow> = Vec::new();
    for (i, a) in draft.artifacts().iter().enumerate() {
        if let Value::Mapping(m) = a {
            rows.push(row_for_artifact(i, m, &findings));
        }
    }
    for (i, c) in draft.changes().iter().enumerate() {
        if let Value::Mapping(m) = c {
            rows.push(row_for_change(i, m, &findings));
        }
    }

    StatusReport {
        kind: draft
            .kind()
            .map(|k| k.as_str().to_string())
            .unwrap_or_else(|| "?".into()),
        reason: draft.reason().to_string(),
        rows,
        error_count,
        warning_count,
        all_codes,
    }
}

fn row_for_artifact(index: usize, m: &serde_yaml::Mapping, findings: &[Finding]) -> StatusRow {
    let artifact_type = str_field(m, "type").unwrap_or_else(|| "?".into());
    let ref_name = str_field(m, "ref");
    let title = str_field(m, "title").unwrap_or_else(|| "(no title)".into());
    let key_fields = key_fields_for(m, &artifact_type);
    let loc_prefix = format!("$.artifacts[{index}]");
    let (glyph, codes) = glyph_and_codes(&loc_prefix, findings);
    StatusRow { index, ref_name, artifact_type, title, key_fields, glyph, codes }
}

fn row_for_change(index: usize, m: &serde_yaml::Mapping, findings: &[Finding]) -> StatusRow {
    let target = str_field(m, "target").unwrap_or_else(|| "?".into());
    let count = m
        .get(Value::String("mutations".into()))
        .and_then(|v| v.as_sequence())
        .map(|s| s.len())
        .unwrap_or(0);
    let loc_prefix = format!("$.changes[{index}]");
    let (glyph, codes) = glyph_and_codes(&loc_prefix, findings);
    StatusRow {
        index,
        ref_name: None,
        artifact_type: "change".into(),
        title: format!("target {target}"),
        key_fields: vec![("mutations".into(), count.to_string())],
        glyph,
        codes,
    }
}

fn str_field(m: &serde_yaml::Mapping, key: &str) -> Option<String> {
    m.get(Value::String(key.into()))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn seq_field_str(m: &serde_yaml::Mapping, key: &str) -> Option<String> {
    m.get(Value::String(key.into()))
        .and_then(|v| v.as_sequence())
        .map(|s| {
            s.iter()
                .filter_map(|x| x.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
}

fn key_fields_for(m: &serde_yaml::Mapping, artifact_type: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    match artifact_type {
        "feature" => {
            if let Some(p) = m
                .get(Value::String("phase".into()))
                .and_then(|v| v.as_u64())
            {
                out.push(("phase".into(), p.to_string()));
            }
            if let Some(d) = seq_field_str(m, "domains") {
                if !d.is_empty() {
                    out.push(("domains".into(), d));
                }
            }
        }
        "adr" => {
            if let Some(s) = str_field(m, "scope") {
                out.push(("scope".into(), s));
            }
            if let Some(d) = seq_field_str(m, "domains") {
                if !d.is_empty() {
                    out.push(("domains".into(), d));
                }
            }
        }
        "tc" => {
            if let Some(t) = str_field(m, "tc-type") {
                out.push(("tc-type".into(), t));
            }
        }
        "dep" => {
            if let Some(t) = str_field(m, "dep-type") {
                out.push(("dep-type".into(), t));
            }
            if let Some(v) = str_field(m, "version") {
                out.push(("version".into(), v));
            }
        }
        _ => {}
    }
    out
}

fn glyph_and_codes(loc_prefix: &str, findings: &[Finding]) -> (char, Vec<String>) {
    let mut codes: Vec<String> = Vec::new();
    let mut has_err = false;
    let mut has_warn = false;
    for f in findings {
        if f.location.starts_with(loc_prefix) {
            codes.push(f.code.clone());
            if f.is_error() {
                has_err = true;
            } else {
                has_warn = true;
            }
        }
    }
    let glyph = if has_err {
        '✗'
    } else if has_warn {
        '⚠'
    } else {
        '✓'
    };
    (glyph, codes)
}
