//! Structural gap analysis — check_adr, check_all, check_changed (ADR-019)

use crate::graph::KnowledgeGraph;
use crate::types::*;
use sha2::{Digest, Sha256};
use std::path::Path;

use super::baseline::GapBaseline;
use super::{GapFinding, GapReport, GapSeverity, GapSummary, hex};

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

    check_g003_rejected_alternatives(adr, adr_id, baseline, &mut findings);
    check_g006_uncovered_aspects(graph, adr_id, baseline, &mut findings);
    check_g007_stale_rationale(graph, adr, adr_id, baseline, &mut findings);
    check_g001_testable_claims(graph, adr, adr_id, baseline, &mut findings);
    check_g002_formal_invariants(graph, adr, adr_id, baseline, &mut findings);

    findings
}

/// Run G008 gap analysis for a feature: check deps have governing ADRs (ADR-030)
pub fn check_feature_dep_gaps(graph: &KnowledgeGraph, feature_id: &str, baseline: &GapBaseline) -> Vec<GapFinding> {
    let mut findings = Vec::new();
    if !graph.features.contains_key(feature_id) {
        return findings;
    }
    // Find all deps linked to this feature
    for dep in graph.dependencies.values() {
        if !dep.front.features.contains(&feature_id.to_string()) {
            continue;
        }
        // Check if any ADR has a governs edge to this dep
        let has_governing_adr = dep.front.adrs.iter().any(|adr_id| graph.adrs.contains_key(adr_id));
        if !has_governing_adr {
            let desc = format!(
                "Feature {} uses dependency {} with no ADR governing its use",
                feature_id, dep.front.id
            );
            let id = gap_id(feature_id, "G008", &[feature_id, &dep.front.id], &desc);
            let suppressed = baseline.is_suppressed(&id);
            findings.push(GapFinding {
                id,
                code: "G008".to_string(),
                severity: GapSeverity::Medium,
                description: desc,
                affected_artifacts: vec![feature_id.to_string(), dep.front.id.clone()],
                suggested_action: format!("Add an ADR governing the use of {} and link it via the `adrs` field.", dep.front.id),
                suppressed,
            });
        }
    }
    findings
}

fn check_g003_rejected_alternatives(adr: &crate::types::Adr, adr_id: &str, baseline: &GapBaseline, findings: &mut Vec<GapFinding>) {
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
}

fn check_g006_uncovered_aspects(graph: &KnowledgeGraph, adr_id: &str, baseline: &GapBaseline, findings: &mut Vec<GapFinding>) {
    for f in graph.features.values() {
        if f.front.adrs.contains(&adr_id.to_string()) && f.front.adrs.len() <= 1 && f.body.len() > 200 {
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

fn check_g007_stale_rationale(graph: &KnowledgeGraph, adr: &crate::types::Adr, adr_id: &str, baseline: &GapBaseline, findings: &mut Vec<GapFinding>) {
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
}

fn check_g001_testable_claims(graph: &KnowledgeGraph, adr: &crate::types::Adr, adr_id: &str, baseline: &GapBaseline, findings: &mut Vec<GapFinding>) {
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
}

fn check_g002_formal_invariants(graph: &KnowledgeGraph, adr: &crate::types::Adr, adr_id: &str, baseline: &GapBaseline, findings: &mut Vec<GapFinding>) {
    let has_linked_tests = graph.tests.values().any(|t| t.front.validates.adrs.contains(&adr_id.to_string()));
    let adr_tests: Vec<&TestCriterion> = graph.tests.values()
        .filter(|t| t.front.validates.adrs.contains(&adr_id.to_string()))
        .collect();
    let has_formal_invariant = adr.body.contains("\u{27E6}\u{0393}:Invariants\u{27E7}") || adr.body.contains("Invariants");
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
