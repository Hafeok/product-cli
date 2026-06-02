//! Model response parsing for LLM-based gap analysis (ADR-019)

use super::baseline::GapBaseline;
use super::{GapFinding, GapSeverity};

/// Error type for model call failures
#[derive(Debug)]
pub struct ModelError(pub String);

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "model error: {}", self.0)
    }
}

/// Attempt model-based gap analysis. Returns Ok(findings) or Err on model failure.
/// Uses PRODUCT_GAP_INJECT_ERROR / PRODUCT_GAP_INJECT_RESPONSE env vars for testing.
pub fn try_model_analysis(adr_id: &str, baseline: &GapBaseline) -> std::result::Result<Vec<GapFinding>, ModelError> {
    // Check for injected error (testing)
    if let Ok(err_msg) = std::env::var("PRODUCT_GAP_INJECT_ERROR") {
        return Err(ModelError(err_msg));
    }

    // Check for injected response (testing)
    if let Ok(response) = std::env::var("PRODUCT_GAP_INJECT_RESPONSE") {
        return Ok(parse_model_findings(&response, adr_id, baseline));
    }

    // No real LLM call yet — return empty
    Ok(Vec::new())
}

/// Parse model response JSON into findings, discarding malformed entries.
/// Logs discarded entries to stderr.
pub fn parse_model_findings(response: &str, adr_id: &str, baseline: &GapBaseline) -> Vec<GapFinding> {
    let parsed: serde_json::Value = match serde_json::from_str(response) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("warning: model response is not valid JSON: {}", e);
            return Vec::new();
        }
    };

    let arr = match parsed.as_array() {
        Some(a) => a,
        None => {
            eprintln!("warning: model response is not a JSON array");
            return Vec::new();
        }
    };

    let mut findings = Vec::new();
    for (i, item) in arr.iter().enumerate() {
        match validate_and_parse_finding(item, adr_id, baseline) {
            Some(f) => findings.push(f),
            None => {
                eprintln!("warning: discarding malformed finding at index {}: {}", i, item);
            }
        }
    }
    findings
}

fn validate_and_parse_finding(
    value: &serde_json::Value,
    _adr_id: &str,
    baseline: &GapBaseline,
) -> Option<GapFinding> {
    let obj = value.as_object()?;

    // Required fields
    let id = obj.get("id")?.as_str()?.to_string();
    let code = obj.get("code")?.as_str()?.to_string();
    let severity_str = obj.get("severity")?.as_str()?;
    let description = obj.get("description")?.as_str()?.to_string();
    let affected_artifacts: Vec<String> = obj
        .get("affected_artifacts")?
        .as_array()?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let suggested_action = obj.get("suggested_action")?.as_str()?.to_string();

    if affected_artifacts.is_empty() {
        return None;
    }

    let severity = match severity_str {
        "high" => GapSeverity::High,
        "medium" => GapSeverity::Medium,
        "low" => GapSeverity::Low,
        _ => return None,
    };

    let suppressed = baseline.is_suppressed(&id);

    Some(GapFinding {
        id,
        code,
        severity,
        description,
        affected_artifacts,
        suggested_action,
        suppressed,
    })
}
