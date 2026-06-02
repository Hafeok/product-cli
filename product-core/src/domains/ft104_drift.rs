//! FT-104: drift detection for default-acknowledged-cross-cutting ADRs

use crate::config::FeaturesConfig;
use crate::error::Diagnostic;
use crate::graph::KnowledgeGraph;
use crate::types::AdrScope;

/// Check for drift between [features].default-acknowledged-cross-cutting and
/// the live ADR catalog. Three forms of drift:
/// 1. Listed ADR no longer exists
/// 2. Listed ADR's scope changed away from cross-cutting
/// 3. Feature's adrs-rejected references an ADR not in default-acknowledged
pub fn check_default_ack_drift(
    graph: &KnowledgeGraph,
    features_config: &FeaturesConfig,
    warnings: &mut Vec<Diagnostic>,
) {
    let default_ack = &features_config.default_acknowledged_cross_cutting;

    // Check each ADR in the default-acknowledged list
    for adr_id in default_ack {
        match graph.adrs.get(adr_id.as_str()) {
            None => {
                // Drift form 1: ADR no longer exists
                warnings.push(
                    Diagnostic::warning(
                        "W036",
                        &format!(
                            "default-acknowledged-cross-cutting lists {} but this ADR no longer exists",
                            adr_id
                        ),
                    )
                    .with_detail(&format!(
                        "Remove {} from [features].default-acknowledged-cross-cutting in product.toml, or restore the ADR file",
                        adr_id
                    ))
                );
            }
            Some(adr) => {
                // Drift form 2: ADR scope changed away from cross-cutting
                if adr.front.scope != AdrScope::CrossCutting {
                    warnings.push(
                        Diagnostic::warning(
                            "W037",
                            &format!(
                                "default-acknowledged-cross-cutting lists {} but its scope is now {}",
                                adr_id, adr.front.scope
                            ),
                        )
                        .with_detail(&format!(
                            "Remove {} from [features].default-acknowledged-cross-cutting (it's no longer cross-cutting)",
                            adr_id
                        ))
                    );
                }
            }
        }
    }

    // Drift form 3: feature rejects an ADR not in default-acknowledged
    for feature in graph.features.values() {
        for rejection in &feature.front.adrs_rejected {
            if !default_ack.contains(&rejection.id) {
                warnings.push(
                    Diagnostic::warning(
                        "W038",
                        &format!(
                            "{} rejects {} but this ADR is not in default-acknowledged-cross-cutting",
                            feature.front.id, rejection.id
                        ),
                    )
                    .with_detail(&format!(
                        "Rejecting an ADR that is not default-acknowledged has no effect. Either add {} to [features].default-acknowledged-cross-cutting or remove it from {}'s adrs-rejected list",
                        rejection.id, feature.front.id
                    ))
                    .with_file(feature.path.clone())
                );
            }
        }
    }
}
