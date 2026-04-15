//! Lifecycle validation checks — W017 (ADR-034: Lifecycle Gate)

use super::model::KnowledgeGraph;
use crate::error::{CheckResult, Diagnostic};
use crate::types::*;

impl KnowledgeGraph {
    /// W017: Feature in-progress or complete with a linked ADR still proposed (ADR-034)
    pub(crate) fn check_proposed_adr_lifecycle(&self, result: &mut CheckResult) {
        for f in self.features.values() {
            // W017 only fires for in-progress and complete, not planned or abandoned
            if f.front.status != FeatureStatus::InProgress
                && f.front.status != FeatureStatus::Complete
            {
                continue;
            }
            for adr_id in &f.front.adrs {
                if let Some(adr) = self.adrs.get(adr_id.as_str()) {
                    if adr.front.status == AdrStatus::Proposed {
                        result.warnings.push(
                            Diagnostic::warning(
                                "W017",
                                "feature complete but governing ADR not yet accepted",
                            )
                            .with_file(f.path.clone())
                            .with_detail(&format!(
                                "{} has status '{}' but linked {} has status 'proposed'",
                                f.front.id, f.front.status, adr_id
                            ))
                            .with_hint(&format!(
                                "accept the ADR with `product adr status {} accepted`\n           or remove the link if the ADR no longer governs this feature",
                                adr_id
                            )),
                        );
                    }
                }
            }
        }
    }
}
