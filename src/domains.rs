//! Concern Domains and Pre-flight Analysis (ADR-025, ADR-026)
//!
//! Domain classification, cross-cutting scope, preflight checks,
//! coverage matrix, and feature acknowledgement.

use crate::error::{Diagnostic, ProductError, Result};
use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Preflight analysis
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct PreflightResult {
    pub feature_id: String,
    pub feature_domains: Vec<String>,
    pub cross_cutting_gaps: Vec<CrossCuttingGap>,
    pub domain_gaps: Vec<DomainGap>,
    pub is_clean: bool,
}

#[derive(Debug)]
pub struct CrossCuttingGap {
    pub adr_id: String,
    pub adr_title: String,
    pub adr_domains: Vec<String>,
    pub status: CoverageStatus,
}

#[derive(Debug)]
pub struct DomainGap {
    pub domain: String,
    pub adr_count: usize,
    pub status: CoverageStatus,
    pub top_adrs: Vec<(String, String)>, // (id, title)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoverageStatus {
    Linked,
    Acknowledged(String), // reason
    Gap,
}

impl std::fmt::Display for CoverageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linked => write!(f, "linked"),
            Self::Acknowledged(r) => write!(f, "acknowledged: {}", r),
            Self::Gap => write!(f, "gap"),
        }
    }
}

/// Run preflight analysis on a feature
pub fn preflight(
    graph: &KnowledgeGraph,
    feature_id: &str,
    _domain_vocab: &HashMap<String, String>,
) -> std::result::Result<PreflightResult, ProductError> {
    let feature = graph.features.get(feature_id).ok_or_else(|| {
        ProductError::NotFound(format!("feature {}", feature_id))
    })?;

    let mut cross_cutting_gaps = Vec::new();
    let mut domain_gaps = Vec::new();

    // Check all cross-cutting ADRs
    for adr in graph.adrs.values() {
        if adr.front.scope != AdrScope::CrossCutting {
            continue;
        }
        let status = if feature.front.adrs.contains(&adr.front.id) {
            CoverageStatus::Linked
        } else if let Some(reason) = find_acknowledgement(feature, &adr.front.id, &adr.front.domains) {
            CoverageStatus::Acknowledged(reason)
        } else {
            CoverageStatus::Gap
        };
        cross_cutting_gaps.push(CrossCuttingGap {
            adr_id: adr.front.id.clone(),
            adr_title: adr.front.title.clone(),
            adr_domains: adr.front.domains.clone(),
            status,
        });
    }

    // Check domain coverage for each domain the feature declares
    let centrality = graph.betweenness_centrality();
    for domain in &feature.front.domains {
        let domain_adrs: Vec<&Adr> = graph.adrs.values()
            .filter(|a| a.front.domains.contains(domain) && a.front.scope != AdrScope::CrossCutting)
            .collect();

        if domain_adrs.is_empty() {
            continue; // No ADRs for this domain — not applicable
        }

        let any_linked = domain_adrs.iter().any(|a| feature.front.adrs.contains(&a.front.id));
        let acknowledged = feature.front.domains_acknowledged.get(domain);

        let status = if any_linked {
            CoverageStatus::Linked
        } else if let Some(reason) = acknowledged {
            CoverageStatus::Acknowledged(reason.clone())
        } else {
            CoverageStatus::Gap
        };

        // Top-2 ADRs by centrality for this domain
        let mut ranked: Vec<_> = domain_adrs.iter()
            .map(|a| (a.front.id.clone(), a.front.title.clone(), centrality.get(&a.front.id).copied().unwrap_or(0.0)))
            .collect();
        ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        let top_adrs: Vec<(String, String)> = ranked.into_iter().take(2).map(|(id, title, _)| (id, title)).collect();

        domain_gaps.push(DomainGap {
            domain: domain.clone(),
            adr_count: domain_adrs.len(),
            status,
            top_adrs,
        });
    }

    let is_clean = cross_cutting_gaps.iter().all(|g| g.status != CoverageStatus::Gap)
        && domain_gaps.iter().all(|g| g.status != CoverageStatus::Gap);

    Ok(PreflightResult {
        feature_id: feature_id.to_string(),
        feature_domains: feature.front.domains.clone(),
        cross_cutting_gaps,
        domain_gaps,
        is_clean,
    })
}

fn find_acknowledgement(feature: &Feature, adr_id: &str, adr_domains: &[String]) -> Option<String> {
    // Check if any of the ADR's domains are acknowledged by the feature
    for domain in adr_domains {
        if let Some(reason) = feature.front.domains_acknowledged.get(domain) {
            if !reason.trim().is_empty() {
                return Some(reason.clone());
            }
        }
    }
    // Also check direct ADR acknowledgement (stored as adr ID key)
    feature.front.domains_acknowledged.get(adr_id)
        .filter(|r| !r.trim().is_empty())
        .cloned()
}

// ---------------------------------------------------------------------------
// Coverage matrix
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct CoverageMatrix {
    pub features: Vec<String>,
    pub domains: Vec<String>,
    pub cells: HashMap<(String, String), CoverageCell>,
}

#[derive(Debug, Clone)]
pub enum CoverageCell {
    Covered,       // ✓ linked
    Acknowledged,  // ~ acknowledged with reason
    NotApplicable, // · feature doesn't touch this domain
    Gap,           // ✗ gap
}

impl std::fmt::Display for CoverageCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Covered => write!(f, "✓"),
            Self::Acknowledged => write!(f, "~"),
            Self::NotApplicable => write!(f, "·"),
            Self::Gap => write!(f, "✗"),
        }
    }
}

pub fn build_coverage_matrix(
    graph: &KnowledgeGraph,
    domain_vocab: &HashMap<String, String>,
) -> CoverageMatrix {
    let mut features: Vec<String> = graph.features.keys().cloned().collect();
    features.sort();
    let mut domains: Vec<String> = domain_vocab.keys().cloned().collect();
    domains.sort();

    let mut cells = HashMap::new();

    for fid in &features {
        if let Some(f) = graph.features.get(fid) {
            for domain in &domains {
                let domain_adrs: Vec<&Adr> = graph.adrs.values()
                    .filter(|a| a.front.domains.contains(domain))
                    .collect();

                if domain_adrs.is_empty() || (!f.front.domains.contains(domain) && !has_cross_cutting_in_domain(graph, domain)) {
                    cells.insert((fid.clone(), domain.clone()), CoverageCell::NotApplicable);
                    continue;
                }

                let any_linked = domain_adrs.iter().any(|a| f.front.adrs.contains(&a.front.id));
                let acknowledged = f.front.domains_acknowledged.contains_key(domain);

                let cell = if any_linked {
                    CoverageCell::Covered
                } else if acknowledged {
                    CoverageCell::Acknowledged
                } else if f.front.domains.contains(domain) {
                    CoverageCell::Gap
                } else {
                    CoverageCell::NotApplicable
                };
                cells.insert((fid.clone(), domain.clone()), cell);
            }
        }
    }

    CoverageMatrix { features, domains, cells }
}

fn has_cross_cutting_in_domain(graph: &KnowledgeGraph, domain: &str) -> bool {
    graph.adrs.values().any(|a| a.front.scope == AdrScope::CrossCutting && a.front.domains.contains(&domain.to_string()))
}

pub fn render_coverage_matrix(matrix: &CoverageMatrix, graph: &KnowledgeGraph) -> String {
    render_coverage_matrix_filtered(matrix, graph, None)
}

pub fn render_coverage_matrix_filtered(
    matrix: &CoverageMatrix,
    graph: &KnowledgeGraph,
    domain_filter: Option<&str>,
) -> String {
    let mut out = String::new();

    let display_domains: Vec<&String> = if let Some(filter) = domain_filter {
        matrix.domains.iter().filter(|d| d.as_str() == filter).collect()
    } else {
        matrix.domains.iter().collect()
    };

    // Header
    out.push_str(&format!("{:<20}", ""));
    for d in &display_domains {
        let short = if d.len() > 5 { &d[..5] } else { d };
        out.push_str(&format!(" {:<5}", short));
    }
    out.push('\n');

    // Rows
    for fid in &matrix.features {
        let title = graph.features.get(fid).map(|f| f.front.title.as_str()).unwrap_or("");
        let label = format!("{} {}", fid, &title[..title.len().min(12)]);
        out.push_str(&format!("{:<20}", label));
        for d in &display_domains {
            let cell = matrix.cells.get(&(fid.clone(), (*d).clone()))
                .cloned()
                .unwrap_or(CoverageCell::NotApplicable);
            out.push_str(&format!("  {}  ", cell));
        }
        out.push('\n');
    }

    out.push_str("\nLegend:  ✓ covered   ~ acknowledged   · not applicable   ✗ gap\n");
    out
}

pub fn coverage_matrix_to_json(matrix: &CoverageMatrix) -> serde_json::Value {
    let features: Vec<serde_json::Value> = matrix.features.iter().map(|fid| {
        let domains: HashMap<String, String> = matrix.domains.iter().map(|d| {
            let cell = matrix.cells.get(&(fid.clone(), d.clone()))
                .cloned()
                .unwrap_or(CoverageCell::NotApplicable);
            let status = match cell {
                CoverageCell::Covered => "covered",
                CoverageCell::Acknowledged => "acknowledged",
                CoverageCell::NotApplicable => "not-applicable",
                CoverageCell::Gap => "gap",
            };
            (d.clone(), status.to_string())
        }).collect();
        serde_json::json!({"id": fid, "domains": domains})
    }).collect();
    serde_json::json!({"features": features, "domains": matrix.domains})
}

// ---------------------------------------------------------------------------
// Validation: E011, W010, W011
// ---------------------------------------------------------------------------

pub fn validate_domains(
    graph: &KnowledgeGraph,
    domain_vocab: &HashMap<String, String>,
    errors: &mut Vec<Diagnostic>,
    warnings: &mut Vec<Diagnostic>,
) {
    // E011: acknowledgement without reasoning
    for f in graph.features.values() {
        for (domain, reason) in &f.front.domains_acknowledged {
            if reason.trim().is_empty() {
                errors.push(
                    Diagnostic::error("E011", "acknowledgement without reasoning")
                        .with_file(f.path.clone())
                        .with_detail(&format!(
                            "{} has domains-acknowledged.{} with empty reason",
                            f.front.id, domain
                        ))
                        .with_hint("provide a reason for why this domain does not apply"),
                );
            }
        }
    }

    // E012: unknown domain
    for f in graph.features.values() {
        for domain in &f.front.domains {
            if !domain_vocab.contains_key(domain) {
                errors.push(
                    Diagnostic::error("E012", "unknown domain")
                        .with_file(f.path.clone())
                        .with_detail(&format!(
                            "{} declares domain '{}' not in product.toml [domains]",
                            f.front.id, domain
                        )),
                );
            }
        }
    }

    // W010: unacknowledged cross-cutting ADR
    let cross_cutting: Vec<&Adr> = graph.adrs.values()
        .filter(|a| a.front.scope == AdrScope::CrossCutting)
        .collect();

    for f in graph.features.values() {
        if f.front.status == FeatureStatus::Abandoned {
            continue;
        }
        for cc_adr in &cross_cutting {
            if !f.front.adrs.contains(&cc_adr.front.id) {
                let acked = find_acknowledgement(f, &cc_adr.front.id, &cc_adr.front.domains).is_some();
                if !acked {
                    warnings.push(
                        Diagnostic::warning("W010", "unacknowledged cross-cutting ADR")
                            .with_file(f.path.clone())
                            .with_detail(&format!(
                                "{} has not acknowledged {} (cross-cutting, {})",
                                f.front.id, cc_adr.front.id,
                                cc_adr.front.domains.join(", ")
                            )),
                    );
                }
            }
        }
    }

    // W011: domain gap without acknowledgement
    for f in graph.features.values() {
        if f.front.status == FeatureStatus::Abandoned {
            continue;
        }
        for domain in &f.front.domains {
            let domain_adrs: Vec<&Adr> = graph.adrs.values()
                .filter(|a| a.front.domains.contains(domain) && a.front.scope != AdrScope::CrossCutting)
                .collect();

            if domain_adrs.is_empty() {
                continue;
            }

            let any_linked = domain_adrs.iter().any(|a| f.front.adrs.contains(&a.front.id));
            let acknowledged = f.front.domains_acknowledged.contains_key(domain);

            if !any_linked && !acknowledged {
                warnings.push(
                    Diagnostic::warning("W011", "domain gap without acknowledgement")
                        .with_file(f.path.clone())
                        .with_detail(&format!(
                            "{} declares domain '{}' ({} ADRs) but none linked or acknowledged",
                            f.front.id, domain, domain_adrs.len()
                        )),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature acknowledge command
// ---------------------------------------------------------------------------

/// Add a domain acknowledgement to a feature's front-matter
pub fn acknowledge_domain(
    feature: &Feature,
    domain: &str,
    reason: &str,
) -> Result<FeatureFrontMatter> {
    if reason.trim().is_empty() {
        return Err(ProductError::ConfigError(
            "E011: acknowledgement requires a non-empty reason".to_string(),
        ));
    }
    let mut front = feature.front.clone();
    front.domains_acknowledged.insert(domain.to_string(), reason.to_string());
    Ok(front)
}

/// Add an ADR acknowledgement (stored under the ADR's domains)
pub fn acknowledge_adr(
    feature: &Feature,
    adr: &Adr,
    reason: &str,
) -> Result<FeatureFrontMatter> {
    if reason.trim().is_empty() {
        return Err(ProductError::ConfigError(
            "E011: acknowledgement requires a non-empty reason".to_string(),
        ));
    }
    let mut front = feature.front.clone();
    // Store under the ADR ID as key
    front.domains_acknowledged.insert(adr.front.id.clone(), reason.to_string());
    Ok(front)
}

// ---------------------------------------------------------------------------
// Preflight report rendering
// ---------------------------------------------------------------------------

pub fn render_preflight(result: &PreflightResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("Pre-flight analysis: {}\n", result.feature_id));
    if !result.feature_domains.is_empty() {
        out.push_str(&format!("Feature domains: {}\n", result.feature_domains.join(", ")));
    }
    out.push('\n');

    if !result.cross_cutting_gaps.is_empty() {
        out.push_str("Cross-Cutting ADRs:\n");
        for gap in &result.cross_cutting_gaps {
            let symbol = match &gap.status {
                CoverageStatus::Linked => "✓",
                CoverageStatus::Acknowledged(_) => "~",
                CoverageStatus::Gap => "✗",
            };
            out.push_str(&format!("  {}  {:<10} {:<40} [{}]\n",
                symbol, gap.adr_id, gap.adr_title,
                match &gap.status {
                    CoverageStatus::Linked => "linked".to_string(),
                    CoverageStatus::Acknowledged(r) => format!("acknowledged: {}", &r[..r.len().min(30)]),
                    CoverageStatus::Gap => "NOT COVERED".to_string(),
                }
            ));
        }
        out.push('\n');
    }

    if !result.domain_gaps.is_empty() {
        out.push_str("Domain Coverage:\n");
        for gap in &result.domain_gaps {
            let symbol = match &gap.status {
                CoverageStatus::Linked => "✓",
                CoverageStatus::Acknowledged(_) => "~",
                CoverageStatus::Gap => "✗",
            };
            let adrs_str = gap.top_adrs.iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("  {}  {:<15} {} ADR(s) — top: {}\n",
                symbol, gap.domain, gap.adr_count, adrs_str
            ));
        }
        out.push('\n');
    }

    if result.is_clean {
        out.push_str("Pre-flight: CLEAN\n");
    } else {
        let cc_gaps = result.cross_cutting_gaps.iter().filter(|g| g.status == CoverageStatus::Gap).count();
        let d_gaps = result.domain_gaps.iter().filter(|g| g.status == CoverageStatus::Gap).count();
        out.push_str(&format!("Pre-flight: {} cross-cutting gap(s), {} domain gap(s)\n", cc_gaps, d_gaps));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_feature(id: &str, adrs: Vec<&str>, domains: Vec<&str>) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.to_string(),
                title: format!("Feature {}", id),
                phase: 1,
                status: FeatureStatus::Planned,
                depends_on: vec![],
                adrs: adrs.into_iter().map(String::from).collect(),
                tests: vec![],
                domains: domains.into_iter().map(String::from).collect(),
                domains_acknowledged: HashMap::new(),
                bundle: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    fn make_adr(id: &str, scope: AdrScope, domains: Vec<&str>) -> Adr {
        Adr {
            front: AdrFrontMatter {
                id: id.to_string(),
                title: format!("ADR {}", id),
                status: AdrStatus::Accepted,
                features: vec![],
                supersedes: vec![],
                superseded_by: vec![],
                domains: domains.into_iter().map(String::from).collect(),
                scope,
                content_hash: None,
                amendments: vec![],
                source_files: vec![],
            },
            body: String::new(),
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    #[test]
    fn preflight_clean_when_all_covered() {
        let f = make_feature("FT-001", vec!["ADR-001"], vec![]);
        let a = make_adr("ADR-001", AdrScope::CrossCutting, vec!["error-handling"]);
        let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
        let vocab = HashMap::new();
        let result = preflight(&graph, "FT-001", &vocab).expect("preflight");
        assert!(result.is_clean);
    }

    #[test]
    fn preflight_detects_cross_cutting_gap() {
        let f = make_feature("FT-001", vec![], vec![]); // no ADRs linked
        let a = make_adr("ADR-001", AdrScope::CrossCutting, vec!["error-handling"]);
        let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
        let vocab = HashMap::new();
        let result = preflight(&graph, "FT-001", &vocab).expect("preflight");
        assert!(!result.is_clean);
        assert!(result.cross_cutting_gaps.iter().any(|g| g.status == CoverageStatus::Gap));
    }

    #[test]
    fn preflight_detects_domain_gap() {
        let f = make_feature("FT-001", vec![], vec!["security"]); // declares security domain
        let a = make_adr("ADR-010", AdrScope::Domain, vec!["security"]);
        let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
        let mut vocab = HashMap::new();
        vocab.insert("security".to_string(), "Security concerns".to_string());
        let result = preflight(&graph, "FT-001", &vocab).expect("preflight");
        assert!(!result.is_clean);
        assert!(result.domain_gaps.iter().any(|g| g.domain == "security" && g.status == CoverageStatus::Gap));
    }

    #[test]
    fn acknowledgement_closes_gap() {
        let mut f = make_feature("FT-001", vec![], vec!["security"]);
        f.front.domains_acknowledged.insert("security".to_string(), "no trust boundaries".to_string());
        let a = make_adr("ADR-010", AdrScope::Domain, vec!["security"]);
        let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
        let mut vocab = HashMap::new();
        vocab.insert("security".to_string(), "Security concerns".to_string());
        let result = preflight(&graph, "FT-001", &vocab).expect("preflight");
        assert!(result.is_clean);
    }

    #[test]
    fn empty_acknowledgement_rejected() {
        let f = make_feature("FT-001", vec![], vec![]);
        let result = acknowledge_domain(&f, "security", "");
        assert!(result.is_err());
    }

    #[test]
    fn coverage_matrix_builds() {
        let f = make_feature("FT-001", vec!["ADR-001"], vec!["security"]);
        let a = make_adr("ADR-001", AdrScope::Domain, vec!["security"]);
        let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
        let mut vocab = HashMap::new();
        vocab.insert("security".to_string(), "Security".to_string());
        let matrix = build_coverage_matrix(&graph, &vocab);
        assert_eq!(matrix.features.len(), 1);
        let cell = matrix.cells.get(&("FT-001".to_string(), "security".to_string()));
        assert!(matches!(cell, Some(CoverageCell::Covered)));
    }

    #[test]
    fn e011_empty_reason_detected() {
        let mut f = make_feature("FT-001", vec![], vec![]);
        f.front.domains_acknowledged.insert("security".to_string(), "".to_string());
        let graph = KnowledgeGraph::build(vec![f], vec![], vec![]);
        let vocab = HashMap::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        validate_domains(&graph, &vocab, &mut errors, &mut warnings);
        assert!(errors.iter().any(|e| e.code == "E011"));
    }

    #[test]
    fn w010_cross_cutting_unacknowledged() {
        let f = make_feature("FT-001", vec![], vec![]);
        let a = make_adr("ADR-001", AdrScope::CrossCutting, vec!["error-handling"]);
        let graph = KnowledgeGraph::build(vec![f], vec![a], vec![]);
        let vocab = HashMap::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        validate_domains(&graph, &vocab, &mut errors, &mut warnings);
        assert!(warnings.iter().any(|w| w.code == "W010"));
    }
}
