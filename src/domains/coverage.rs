//! Coverage matrix for concern domain analysis (ADR-025, ADR-026)

use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CoverageMatrix {
    pub features: Vec<String>,
    pub domains: Vec<String>,
    pub cells: HashMap<(String, String), CoverageCell>,
}

#[derive(Debug, Clone)]
pub enum CoverageCell {
    Covered,       // linked
    Acknowledged,  // ~ acknowledged with reason
    NotApplicable, // feature doesn't touch this domain
    Gap,           // gap
}

impl std::fmt::Display for CoverageCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Covered => write!(f, "\u{2713}"),
            Self::Acknowledged => write!(f, "~"),
            Self::NotApplicable => write!(f, "\u{00b7}"),
            Self::Gap => write!(f, "\u{2717}"),
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

    out.push_str("\nLegend:  \u{2713} covered   ~ acknowledged   \u{00b7} not applicable   \u{2717} gap\n");
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
