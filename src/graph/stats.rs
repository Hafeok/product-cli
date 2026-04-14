//! Graph statistics — node counts, centrality, formal block coverage.

use super::model::KnowledgeGraph;
use super::types::GraphStats;
use crate::types::*;

impl KnowledgeGraph {
    // -----------------------------------------------------------------------
    // Graph statistics
    // -----------------------------------------------------------------------

    pub fn stats(&self) -> GraphStats {
        let centrality = self.betweenness_centrality();
        let adr_centrality: Vec<(String, f64)> = self
            .adrs
            .keys()
            .map(|id| (id.clone(), centrality.get(id).copied().unwrap_or(0.0)))
            .collect();

        let total_nodes = self.features.len() + self.adrs.len() + self.tests.len() + self.dependencies.len();
        let total_edges = self.edges.len();

        // Formal block coverage
        let invariant_chaos: Vec<&TestCriterion> = self
            .tests
            .values()
            .filter(|t| {
                t.front.test_type == TestType::Invariant || t.front.test_type == TestType::Chaos
            })
            .collect();
        let with_formal = invariant_chaos
            .iter()
            .filter(|t| !t.formal_blocks.is_empty())
            .count();
        let formal_coverage = if invariant_chaos.is_empty() {
            100
        } else {
            (with_formal * 100) / invariant_chaos.len()
        };

        GraphStats {
            features: self.features.len(),
            adrs: self.adrs.len(),
            tests: self.tests.len(),
            total_nodes,
            total_edges,
            adr_centrality,
            formal_coverage,
        }
    }
}
