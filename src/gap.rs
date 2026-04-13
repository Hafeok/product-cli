//! Gap analysis — LLM-driven specification review (ADR-019)
//!
//! Structural gap checks (G003, G006, G007) run locally.
//! Semantic checks (G001, G002, G004, G005) require LLM — stubbed for now.

use crate::error::{ProductError, Result};
use crate::graph::KnowledgeGraph;
use crate::types::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

// ---------------------------------------------------------------------------
// Gap types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapSeverity {
    #[serde(rename = "high")]
    High,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
}

impl std::fmt::Display for GapSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapFinding {
    pub id: String,
    pub code: String,
    pub severity: GapSeverity,
    pub description: String,
    pub affected_artifacts: Vec<String>,
    pub suggested_action: String,
    #[serde(default)]
    pub suppressed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapReport {
    pub adr: String,
    pub run_date: String,
    pub product_version: String,
    pub findings: Vec<GapFinding>,
    pub summary: GapSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapSummary {
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub suppressed: usize,
}

// ---------------------------------------------------------------------------
// Baseline file (gaps.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GapBaseline {
    #[serde(rename = "schema-version", default = "default_schema")]
    pub schema_version: String,
    #[serde(default)]
    pub suppressions: Vec<Suppression>,
    #[serde(default)]
    pub resolved: Vec<Resolved>,
}

fn default_schema() -> String {
    "1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suppression {
    pub id: String,
    pub reason: String,
    #[serde(default)]
    pub suppressed_by: String,
    #[serde(default)]
    pub suppressed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolved {
    pub id: String,
    #[serde(default)]
    pub resolved_at: String,
    #[serde(default)]
    pub resolving_commit: String,
}

impl GapBaseline {
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            ProductError::IoError(format!("failed to serialize gaps.json: {}", e))
        })?;
        crate::fileops::write_file_atomic(path, &json)
    }

    pub fn is_suppressed(&self, gap_id: &str) -> bool {
        self.suppressions.iter().any(|s| s.id == gap_id)
    }

    pub fn suppress(&mut self, gap_id: &str, reason: &str) {
        if !self.is_suppressed(gap_id) {
            self.suppressions.push(Suppression {
                id: gap_id.to_string(),
                reason: reason.to_string(),
                suppressed_by: current_git_commit().unwrap_or_default(),
                suppressed_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    pub fn unsuppress(&mut self, gap_id: &str) {
        self.suppressions.retain(|s| s.id != gap_id);
    }

    /// Move gaps that were suppressed but are no longer detected to the resolved list
    pub fn update_resolved(&mut self, all_finding_ids: &[String]) {
        let mut newly_resolved = Vec::new();
        self.suppressions.retain(|s| {
            if all_finding_ids.contains(&s.id) {
                true // still detected, keep suppression
            } else {
                newly_resolved.push(Resolved {
                    id: s.id.clone(),
                    resolved_at: chrono::Utc::now().to_rfc3339(),
                    resolving_commit: current_git_commit().unwrap_or_default(),
                });
                false // no longer detected, remove suppression
            }
        });
        self.resolved.extend(newly_resolved);
    }
}

// ---------------------------------------------------------------------------
// Gap ID derivation
// ---------------------------------------------------------------------------

pub fn gap_id(adr_id: &str, code: &str, artifacts: &[&str], description: &str) -> String {
    let mut sorted = artifacts.to_vec();
    sorted.sort();
    let input = format!("{}{}{}{}", adr_id, code, sorted.join(","), description);
    let hash = Sha256::digest(input.as_bytes());
    let short = hex::encode(&hash[..4]);
    format!("GAP-{}-{}-{}", adr_id, code, short)
}

// ---------------------------------------------------------------------------
// Structural gap analysis (no LLM needed)
// ---------------------------------------------------------------------------

/// Run structural gap analysis on a single ADR
pub fn check_adr(graph: &KnowledgeGraph, adr_id: &str, baseline: &GapBaseline) -> Vec<GapFinding> {
    let mut findings = Vec::new();

    let adr = match graph.adrs.get(adr_id) {
        Some(a) => a,
        None => return findings,
    };

    // G003: Missing rejected alternatives section
    if !adr.body.contains("Rejected alternatives")
        && !adr.body.contains("rejected alternatives")
        && !adr.body.contains("**Rejected")
    {
        let desc = "ADR has no Rejected alternatives section".to_string();
        let id = gap_id(adr_id, "G003", &[adr_id], &desc);
        let suppressed = baseline.is_suppressed(&id);
        findings.push(GapFinding {
            id,
            code: "G003".to_string(),
            severity: GapSeverity::Medium,
            description: desc,
            affected_artifacts: vec![adr_id.to_string()],
            suggested_action: "Add a **Rejected alternatives** section documenting what was considered and why it was rejected.".to_string(),
            suppressed,
        });
    }

    // G006: Feature aspects not addressed by linked ADRs
    // Check if any feature linked to this ADR has aspects not covered
    for f in graph.features.values() {
        if f.front.adrs.contains(&adr_id.to_string()) {
            // If the feature has content but only 1 ADR linked, it might have uncovered aspects
            if f.front.adrs.len() <= 1 && f.body.len() > 200 {
                let desc = format!(
                    "Feature {} has substantial content but only 1 linked ADR — some aspects may not be addressed",
                    f.front.id
                );
                let id = gap_id(adr_id, "G006", &[adr_id, &f.front.id], &desc);
                let suppressed = baseline.is_suppressed(&id);
                findings.push(GapFinding {
                    id,
                    code: "G006".to_string(),
                    severity: GapSeverity::Medium,
                    description: desc,
                    affected_artifacts: vec![adr_id.to_string(), f.front.id.clone()],
                    suggested_action: "Review feature content and consider if additional ADRs are needed.".to_string(),
                    suppressed,
                });
            }
        }
    }

    // G007: Stale rationale — references superseded ADRs
    for other_adr in graph.adrs.values() {
        if other_adr.front.status == AdrStatus::Superseded
            && adr.body.contains(&other_adr.front.id) {
                let desc = format!(
                    "Rationale references {} which has been superseded",
                    other_adr.front.id
                );
                let id = gap_id(adr_id, "G007", &[adr_id, &other_adr.front.id], &desc);
                let suppressed = baseline.is_suppressed(&id);
                findings.push(GapFinding {
                    id,
                    code: "G007".to_string(),
                    severity: GapSeverity::Low,
                    description: desc,
                    affected_artifacts: vec![adr_id.to_string(), other_adr.front.id.clone()],
                    suggested_action: format!("Update reference to the successor ADR ({}).", other_adr.front.superseded_by.first().cloned().unwrap_or_default()),
                    suppressed,
                });
            }
    }

    // G001: ADR with testable claims but no linked TC (structural heuristic)
    let has_test_section = adr.body.contains("Test coverage") || adr.body.contains("test coverage");
    let has_linked_tests = graph.tests.values().any(|t| t.front.validates.adrs.contains(&adr_id.to_string()));
    if has_test_section && !has_linked_tests {
        let desc = "ADR has a Test coverage section but no TC files link to it".to_string();
        let id = gap_id(adr_id, "G001", &[adr_id], &desc);
        let suppressed = baseline.is_suppressed(&id);
        findings.push(GapFinding {
            id,
            code: "G001".to_string(),
            severity: GapSeverity::High,
            description: desc,
            affected_artifacts: vec![adr_id.to_string()],
            suggested_action: "Create TC files for the test scenarios described in the ADR and link them.".to_string(),
            suppressed,
        });
    }

    // G002: Formal invariant block with no scenario/chaos TC
    let adr_tests: Vec<&TestCriterion> = graph.tests.values()
        .filter(|t| t.front.validates.adrs.contains(&adr_id.to_string()))
        .collect();
    let has_formal_invariant = adr.body.contains("⟦Γ:Invariants⟧") || adr.body.contains("Invariants");
    let has_scenario_chaos = adr_tests.iter().any(|t| {
        t.front.test_type == TestType::Scenario || t.front.test_type == TestType::Chaos
    });
    if has_formal_invariant && !has_scenario_chaos && has_linked_tests {
        let desc = "ADR has formal invariant blocks but no scenario or chaos TC exercises them".to_string();
        let id = gap_id(adr_id, "G002", &[adr_id], &desc);
        let suppressed = baseline.is_suppressed(&id);
        findings.push(GapFinding {
            id,
            code: "G002".to_string(),
            severity: GapSeverity::High,
            description: desc,
            affected_artifacts: vec![adr_id.to_string()],
            suggested_action: "Add a scenario or chaos TC that exercises the declared invariants.".to_string(),
            suppressed,
        });
    }

    findings
}

/// Run gap analysis on all ADRs
pub fn check_all(graph: &KnowledgeGraph, baseline: &GapBaseline) -> Vec<GapReport> {
    let mut reports = Vec::new();
    let mut adr_ids: Vec<&String> = graph.adrs.keys().collect();
    adr_ids.sort();

    for adr_id in adr_ids {
        let findings = check_adr(graph, adr_id, baseline);
        let summary = summarize(&findings);
        reports.push(GapReport {
            adr: adr_id.clone(),
            run_date: chrono::Utc::now().to_rfc3339(),
            product_version: env!("CARGO_PKG_VERSION").to_string(),
            findings,
            summary,
        });
    }

    reports
}

/// Run gap analysis on ADRs changed in the last commit (--changed mode)
pub fn check_changed(graph: &KnowledgeGraph, baseline: &GapBaseline, repo_root: &Path) -> Vec<GapReport> {
    let changed_adrs = find_changed_adrs(repo_root, graph);
    let mut reports = Vec::new();

    for adr_id in &changed_adrs {
        let findings = check_adr(graph, adr_id, baseline);
        let summary = summarize(&findings);
        reports.push(GapReport {
            adr: adr_id.clone(),
            run_date: chrono::Utc::now().to_rfc3339(),
            product_version: env!("CARGO_PKG_VERSION").to_string(),
            findings,
            summary,
        });
    }

    reports
}

/// Compute gap statistics
pub fn gap_stats(reports: &[GapReport], baseline: &GapBaseline) -> serde_json::Value {
    let total_findings: usize = reports.iter().map(|r| r.findings.len()).sum();
    let high: usize = reports.iter().flat_map(|r| &r.findings).filter(|f| f.severity == GapSeverity::High && !f.suppressed).count();
    let medium: usize = reports.iter().flat_map(|r| &r.findings).filter(|f| f.severity == GapSeverity::Medium && !f.suppressed).count();
    let low: usize = reports.iter().flat_map(|r| &r.findings).filter(|f| f.severity == GapSeverity::Low && !f.suppressed).count();
    let suppressed = baseline.suppressions.len();
    let resolved = baseline.resolved.len();

    serde_json::json!({
        "total_findings": total_findings,
        "unsuppressed": { "high": high, "medium": medium, "low": low },
        "suppressed": suppressed,
        "resolved": resolved,
        "adrs_analysed": reports.len(),
    })
}

fn summarize(findings: &[GapFinding]) -> GapSummary {
    GapSummary {
        high: findings.iter().filter(|f| f.severity == GapSeverity::High && !f.suppressed).count(),
        medium: findings.iter().filter(|f| f.severity == GapSeverity::Medium && !f.suppressed).count(),
        low: findings.iter().filter(|f| f.severity == GapSeverity::Low && !f.suppressed).count(),
        suppressed: findings.iter().filter(|f| f.suppressed).count(),
    }
}

/// Find ADRs changed in the last commit, expanded with 1-hop neighbours
fn find_changed_adrs(repo_root: &Path, graph: &KnowledgeGraph) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1"])
        .current_dir(repo_root)
        .output();

    let changed_files = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return graph.adrs.keys().cloned().collect(), // fallback: all ADRs
    };

    let mut changed_ids = Vec::new();
    for line in changed_files.lines() {
        if line.contains("adrs/") {
            // Extract ADR ID from filename pattern
            if let Some(id) = extract_adr_id_from_path(line) {
                changed_ids.push(id);
            }
        }
    }

    // Expand with 1-hop neighbours
    let mut expanded = changed_ids.clone();
    for adr_id in &changed_ids {
        // Find features linked to this ADR
        for f in graph.features.values() {
            if f.front.adrs.contains(adr_id) {
                // Add all other ADRs linked to these features
                for other_adr in &f.front.adrs {
                    if !expanded.contains(other_adr) {
                        expanded.push(other_adr.clone());
                    }
                }
            }
        }
    }

    expanded
}

fn extract_adr_id_from_path(path: &str) -> Option<String> {
    let filename = path.rsplit('/').next()?;
    let parts: Vec<&str> = filename.splitn(3, '-').collect();
    if parts.len() >= 2 {
        Some(format!("{}-{}", parts[0], parts[1]))
    } else {
        None
    }
}

fn current_git_commit() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(format!("git:{}", String::from_utf8_lossy(&output.stdout).trim()))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Model response parsing (for LLM-based gap analysis)
// ---------------------------------------------------------------------------

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

// hex module (avoid adding a dep just for this)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_adr(id: &str, body: &str) -> Adr {
        Adr {
            front: AdrFrontMatter {
                id: id.to_string(),
                title: format!("ADR {}", id),
                status: AdrStatus::Accepted,
                features: vec![],
                supersedes: vec![],
                superseded_by: vec![],
                domains: vec![],
                scope: AdrScope::FeatureSpecific,
            },
            body: body.to_string(),
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    #[test]
    fn gap_id_deterministic() {
        let id1 = gap_id("ADR-001", "G003", &["ADR-001"], "test description");
        let id2 = gap_id("ADR-001", "G003", &["ADR-001"], "test description");
        assert_eq!(id1, id2);
    }

    #[test]
    fn gap_id_format() {
        let id = gap_id("ADR-002", "G001", &["ADR-002"], "missing test");
        assert!(id.starts_with("GAP-ADR-002-G001-"));
        assert!(id.len() > 20); // GAP-ADR-002-G001-XXXX
    }

    #[test]
    fn g003_detected_missing_rejected_alternatives() {
        let adr = make_adr("ADR-001", "**Decision:** Use Rust.\n\n**Rationale:** Fast.\n");
        let graph = KnowledgeGraph::build(vec![], vec![adr], vec![]);
        let baseline = GapBaseline::default();
        let findings = check_adr(&graph, "ADR-001", &baseline);
        assert!(findings.iter().any(|f| f.code == "G003"), "should detect G003");
    }

    #[test]
    fn g003_not_detected_when_present() {
        let adr = make_adr("ADR-001", "**Rejected alternatives:**\n- Go\n- Python\n");
        let graph = KnowledgeGraph::build(vec![], vec![adr], vec![]);
        let baseline = GapBaseline::default();
        let findings = check_adr(&graph, "ADR-001", &baseline);
        assert!(!findings.iter().any(|f| f.code == "G003"));
    }

    #[test]
    fn suppression_works() {
        let adr = make_adr("ADR-001", "Just a decision with no other section.\n");
        let graph = KnowledgeGraph::build(vec![], vec![adr], vec![]);
        let mut baseline = GapBaseline::default();

        let findings = check_adr(&graph, "ADR-001", &baseline);
        assert!(!findings.is_empty());
        let gap_id = &findings[0].id;

        baseline.suppress(gap_id, "known issue");
        let findings2 = check_adr(&graph, "ADR-001", &baseline);
        assert!(findings2.iter().all(|f| f.suppressed || f.id != *gap_id));
    }

    #[test]
    fn baseline_save_load_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("gaps.json");

        let mut baseline = GapBaseline::default();
        baseline.suppress("GAP-TEST-001", "test reason");
        baseline.save(&path).expect("save");

        let loaded = GapBaseline::load(&path);
        assert_eq!(loaded.suppressions.len(), 1);
        assert_eq!(loaded.suppressions[0].id, "GAP-TEST-001");
    }
}
