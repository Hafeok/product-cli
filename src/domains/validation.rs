//! Domain validation diagnostics: E011, E012, W010, W011 (ADR-025, ADR-026)

use crate::error::Diagnostic;
use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::collections::HashMap;

use super::preflight::find_acknowledgement;

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
