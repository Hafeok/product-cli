//! §6.3 data conformance: the production-data validation engine.
//!
//! Validates a production dataset's records against its [`DataShape`], the
//! structure used as the oracle (§3.1). Pure: the records are parsed by the
//! adapter, so this stays I/O-free. The verdict carries the data-divergence
//! rate — the fraction of records that fail — which reads both ways: the data
//! may be wrong, or the spec may be stale.

use serde::Serialize;
use serde_json::Value;

use super::model::{DataShape, DomainGraph};
use crate::error::{ProductError, Result};

/// One way a single record diverged from the shape.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DataFinding {
    /// 0-based index of the record in the dataset.
    pub record: usize,
    pub field: String,
    /// `missing-required` or `not-in-reference-set`.
    pub kind: String,
    pub detail: String,
}

/// The verdict of validating a dataset against its shape (§3.1/§6.3).
#[derive(Debug, Clone, Serialize)]
pub struct DataVerdict {
    pub dataset: String,
    pub shape: String,
    pub target: String,
    pub total: usize,
    pub conforming: usize,
    pub violating: usize,
    /// Fraction of records that fail at least one constraint — the
    /// data-divergence rate (§3.1), the over-confidence signal of spec drift.
    pub divergence_rate: f64,
    pub findings: Vec<DataFinding>,
}

impl DataVerdict {
    /// True when every record satisfies the shape.
    pub fn conformant(&self) -> bool {
        self.findings.is_empty()
    }
}

/// The movement of a dataset's divergence rate versus its previous run — the
/// §3.1 spec-staleness signal, "made visible as it happens" (§13.3). A rising
/// rate is a model falling behind reality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DivergenceTrend {
    /// No prior run for this dataset.
    First,
    Rising,
    Falling,
    Stable,
}

impl DivergenceTrend {
    /// A short human marker for the report line.
    pub fn marker(self) -> &'static str {
        match self {
            Self::First => "first run",
            Self::Rising => "▲ rising",
            Self::Falling => "▼ falling",
            Self::Stable => "▬ stable",
        }
    }
}

/// Classify the current rate against the previous one. Rates within one
/// percentage point count as stable (noise, not movement).
pub fn classify_trend(previous: Option<f64>, current: f64) -> DivergenceTrend {
    match previous {
        None => DivergenceTrend::First,
        Some(prev) if (current - prev).abs() < 0.01 => DivergenceTrend::Stable,
        Some(prev) if current > prev => DivergenceTrend::Rising,
        Some(_) => DivergenceTrend::Falling,
    }
}

/// Validate `records` against the shape the `dataset` conforms to. Errors if the
/// dataset or its shape is not in the graph.
pub fn check_dataset(graph: &DomainGraph, dataset: &str, records: &[Value]) -> Result<DataVerdict> {
    let ds = graph
        .production_datasets
        .iter()
        .find(|d| d.id == dataset)
        .ok_or_else(|| ProductError::NotFound(format!("no production dataset {:?} in the graph", dataset)))?;
    let shape = graph
        .data_shapes
        .iter()
        .find(|s| s.id == ds.shape)
        .ok_or_else(|| ProductError::ConfigError(format!("dataset {:?} names unknown shape {:?}", dataset, ds.shape)))?;

    let mut findings = Vec::new();
    let mut violating = 0usize;
    for (i, rec) in records.iter().enumerate() {
        let before = findings.len();
        check_record(graph, shape, i, rec, &mut findings);
        if findings.len() > before {
            violating += 1;
        }
    }
    let total = records.len();
    let divergence_rate = if total == 0 { 0.0 } else { violating as f64 / total as f64 };
    Ok(DataVerdict {
        dataset: dataset.to_string(),
        shape: shape.id.clone(),
        target: shape.target.clone(),
        total,
        conforming: total - violating,
        violating,
        divergence_rate,
        findings,
    })
}

/// Check one record against the shape, pushing a finding per violated constraint.
fn check_record(graph: &DomainGraph, shape: &DataShape, i: usize, rec: &Value, out: &mut Vec<DataFinding>) {
    let obj = rec.as_object();
    for field in &shape.required {
        let present = obj.and_then(|m| m.get(field)).map(|v| !v.is_null()).unwrap_or(false);
        if !present {
            out.push(DataFinding {
                record: i,
                field: field.clone(),
                kind: "missing-required".to_string(),
                detail: format!("required field {:?} is absent or null", field),
            });
        }
    }
    for c in &shape.enums {
        let Some(val) = obj.and_then(|m| m.get(&c.field)) else { continue };
        if val.is_null() {
            continue;
        }
        let s = value_as_str(val);
        let allowed = graph.reference_sets.iter().find(|rs| rs.id == c.reference_set);
        let ok = allowed.map(|rs| rs.values.contains(&s)).unwrap_or(false);
        if !ok {
            out.push(DataFinding {
                record: i,
                field: c.field.clone(),
                kind: "not-in-reference-set".to_string(),
                detail: format!("value {:?} is not in reference set {:?}", s, c.reference_set),
            });
        }
    }
    for c in &shape.types {
        let Some(val) = obj.and_then(|m| m.get(&c.field)) else { continue };
        if val.is_null() {
            continue;
        }
        if !value_matches_type(val, &c.datatype) {
            out.push(DataFinding {
                record: i,
                field: c.field.clone(),
                kind: "not-of-type".to_string(),
                detail: format!("value {} is not a {}", val, c.datatype),
            });
        }
    }
}

/// True if a JSON value satisfies a declared datatype (`string` · `integer` ·
/// `number` · `boolean` · `date`). An unknown datatype never matches.
fn value_matches_type(v: &Value, datatype: &str) -> bool {
    match datatype {
        "string" => v.is_string(),
        "integer" => v.is_i64() || v.is_u64(),
        "number" => v.is_number(),
        "boolean" => v.is_boolean(),
        "date" => v.as_str().map(is_iso_date).unwrap_or(false),
        _ => false,
    }
}

/// A minimal `YYYY-MM-DD` shape check — the date datatype's machine gate.
fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })
}

/// Render a JSON scalar as the string compared against a reference set.
fn value_as_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
#[path = "data_check_tests.rs"]
mod tests;
