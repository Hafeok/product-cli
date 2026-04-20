//! ADR conflict-check report — pure analysis over the graph.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::types::{Adr, AdrScope};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingCode {
    E004,
    /// W025: supersession asymmetry
    W025,
    /// W026: domain overlap with a cross-cutting ADR
    W026,
    /// W027: cross-cutting ADR carries explicit feature links
    W027,
}

impl FindingCode {
    pub fn is_error(&self) -> bool {
        matches!(self, FindingCode::E004)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FindingCode::E004 => "E004",
            FindingCode::W025 => "W025",
            FindingCode::W026 => "W026",
            FindingCode::W027 => "W027",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictFinding {
    pub code: FindingCode,
    pub adr: String,
    pub message: String,
}

/// Pure: run the structural conflict checks against `targets`.
/// Returns a list of findings. Caller decides on exit code / rendering.
/// Errors are returned only when a target ID is not in the graph.
pub fn check_conflicts(
    graph: &KnowledgeGraph,
    targets: &[String],
) -> Result<Vec<ConflictFinding>, ProductError> {
    let mut findings: Vec<ConflictFinding> = Vec::new();
    for adr_id in targets {
        let adr = graph
            .adrs
            .get(adr_id)
            .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;
        check_one_adr(adr_id, adr, graph, &mut findings);
    }
    Ok(findings)
}

/// Pure helper: yield every ADR id, sorted.
pub fn all_adr_ids(graph: &KnowledgeGraph) -> Vec<String> {
    let mut ids: Vec<String> = graph.adrs.keys().cloned().collect();
    ids.sort();
    ids
}

fn check_one_adr(
    adr_id: &str,
    adr: &Adr,
    graph: &KnowledgeGraph,
    findings: &mut Vec<ConflictFinding>,
) {
    if has_supersession_cycle(adr_id, graph) {
        findings.push(ConflictFinding {
            code: FindingCode::E004,
            adr: adr_id.to_string(),
            message: "supersession cycle detected".to_string(),
        });
    }
    check_supersession_symmetry(adr_id, adr, graph, findings);
    check_domain_overlap(adr_id, adr, graph, findings);
    check_scope_consistency(adr_id, adr, findings);
}

fn has_supersession_cycle(adr_id: &str, graph: &KnowledgeGraph) -> bool {
    if let Some(cycle) = graph.detect_supersession_cycle() {
        cycle.contains(&adr_id.to_string())
    } else {
        false
    }
}

fn check_supersession_symmetry(
    adr_id: &str,
    adr: &Adr,
    graph: &KnowledgeGraph,
    findings: &mut Vec<ConflictFinding>,
) {
    for by in &adr.front.superseded_by {
        if let Some(succ) = graph.adrs.get(by) {
            if !succ.front.supersedes.contains(&adr_id.to_string()) {
                findings.push(ConflictFinding {
                    code: FindingCode::W025,
                    adr: adr_id.to_string(),
                    message: format!(
                        "supersession asymmetry: {} does not list {} in supersedes",
                        by, adr_id
                    ),
                });
            }
        }
    }
    for sup in &adr.front.supersedes {
        if let Some(other) = graph.adrs.get(sup) {
            if !other.front.superseded_by.contains(&adr_id.to_string()) {
                findings.push(ConflictFinding {
                    code: FindingCode::W025,
                    adr: adr_id.to_string(),
                    message: format!(
                        "supersession asymmetry: {} does not list {} in superseded-by",
                        sup, adr_id
                    ),
                });
            }
        }
    }
}

fn check_domain_overlap(
    adr_id: &str,
    adr: &Adr,
    graph: &KnowledgeGraph,
    findings: &mut Vec<ConflictFinding>,
) {
    if adr.front.scope == AdrScope::CrossCutting {
        return;
    }
    for other in graph.adrs.values() {
        if other.front.id == adr_id {
            continue;
        }
        if other.front.scope != AdrScope::CrossCutting {
            continue;
        }
        let overlap: Vec<&String> = adr
            .front
            .domains
            .iter()
            .filter(|d| other.front.domains.contains(d))
            .collect();
        if !overlap.is_empty() {
            let overlap_str: Vec<String> = overlap.iter().map(|s| (*s).clone()).collect();
            findings.push(ConflictFinding {
                code: FindingCode::W026,
                adr: adr_id.to_string(),
                message: format!(
                    "domain overlap with cross-cutting ADR {}: {}",
                    other.front.id,
                    overlap_str.join(", ")
                ),
            });
        }
    }
}

fn check_scope_consistency(adr_id: &str, adr: &Adr, findings: &mut Vec<ConflictFinding>) {
    if adr.front.scope == AdrScope::CrossCutting && !adr.front.features.is_empty() {
        findings.push(ConflictFinding {
            code: FindingCode::W027,
            adr: adr_id.to_string(),
            message: format!(
                "cross-cutting ADR has {} feature link(s) — usually cross-cutting ADRs don't list specific features",
                adr.front.features.len()
            ),
        });
    }
}
