//! Result types for graph operations — phase gates, impact results, statistics.

use super::model::KnowledgeGraph;
use crate::types::TestStatus;

// ---------------------------------------------------------------------------
// Phase gate types (ADR-012)
// ---------------------------------------------------------------------------

/// Status of a single exit-criteria TC for phase gate display
#[derive(Debug, Clone)]
pub struct PhaseGateTC {
    pub id: String,
    pub title: String,
    pub passing: bool,
}

/// Phase gate status
#[derive(Debug, Clone)]
pub enum PhaseGateStatus {
    /// Gate is open — all exit criteria pass (or none exist)
    Open { exit_criteria: Vec<PhaseGateTC> },
    /// Gate is locked — some exit criteria are not passing
    Locked { exit_criteria: Vec<PhaseGateTC>, failing: Vec<String> },
}

impl PhaseGateStatus {
    pub fn is_open(&self) -> bool {
        matches!(self, PhaseGateStatus::Open { .. })
    }
}

/// Result of `feature_next_with_gate`
#[derive(Debug)]
pub enum FeatureNextResult {
    /// A feature is ready to implement
    Ready(String),
    /// No ready feature found because a phase gate blocks the best candidate
    Blocked {
        candidate: String,
        blocked_phase: u32,
        exit_criteria: Vec<PhaseGateTC>,
    },
    /// All features are complete or have unsatisfied dependencies
    AllDone,
}

#[derive(Debug)]
pub struct ImpactResult {
    pub seed: String,
    pub direct_features: Vec<String>,
    pub direct_tests: Vec<String>,
    pub transitive_features: Vec<String>,
    pub transitive_tests: Vec<String>,
}

impl ImpactResult {
    pub fn print(&self, graph: &KnowledgeGraph) {
        self.print_header(graph);
        self.print_direct_dependents(graph);
        self.print_transitive_dependents();
        self.print_summary(graph);
    }

    fn print_header(&self, graph: &KnowledgeGraph) {
        let title = if let Some(f) = graph.features.get(&self.seed) {
            f.front.title.clone()
        } else if let Some(a) = graph.adrs.get(&self.seed) {
            a.front.title.clone()
        } else if let Some(t) = graph.tests.get(&self.seed) {
            t.front.title.clone()
        } else {
            String::new()
        };
        println!("Impact analysis: {} — {}", self.seed, title);
        println!();
    }

    fn print_direct_dependents(&self, graph: &KnowledgeGraph) {
        if self.direct_features.is_empty() && self.direct_tests.is_empty() {
            return;
        }
        println!("Direct dependents:");
        if !self.direct_features.is_empty() {
            let details = format_feature_ids(&self.direct_features, graph);
            println!("  Features:  {}", details.join(", "));
        }
        if !self.direct_tests.is_empty() {
            let details = format_test_ids(&self.direct_tests, graph);
            println!("  Tests:     {}", details.join(", "));
        }
    }

    fn print_transitive_dependents(&self) {
        if self.transitive_features.is_empty() && self.transitive_tests.is_empty() {
            return;
        }
        println!();
        println!("Transitive dependents:");
        if !self.transitive_features.is_empty() {
            println!("  Features:  {}", self.transitive_features.join(", "));
        }
        if !self.transitive_tests.is_empty() {
            println!("  Tests:     {}", self.transitive_tests.join(", "));
        }
    }

    fn print_summary(&self, graph: &KnowledgeGraph) {
        let total_features = self.direct_features.len() + self.transitive_features.len();
        let total_tests = self.direct_tests.len() + self.transitive_tests.len();
        let passing_tests = self
            .direct_tests
            .iter()
            .chain(self.transitive_tests.iter())
            .filter(|id| {
                graph
                    .tests
                    .get(id.as_str())
                    .map(|t| t.front.status == TestStatus::Passing)
                    .unwrap_or(false)
            })
            .count();

        println!();
        print!(
            "Summary: {} features, {} tests affected.",
            total_features, total_tests
        );
        if passing_tests > 0 {
            print!(" {} passing test(s) may be invalidated.", passing_tests);
        }
        println!();
    }
}

/// Format feature IDs with their status for display
fn format_feature_ids(ids: &[String], graph: &KnowledgeGraph) -> Vec<String> {
    ids.iter()
        .map(|id| {
            let status = graph
                .features
                .get(id)
                .map(|f| format!("{}", f.front.status))
                .unwrap_or_default();
            format!("{} ({})", id, status)
        })
        .collect()
}

/// Format test IDs with their status for display
fn format_test_ids(ids: &[String], graph: &KnowledgeGraph) -> Vec<String> {
    ids.iter()
        .map(|id| {
            let status = graph
                .tests
                .get(id)
                .map(|t| format!("{}", t.front.status))
                .unwrap_or_default();
            format!("{} ({})", id, status)
        })
        .collect()
}

#[derive(Debug)]
pub struct GraphStats {
    pub features: usize,
    pub adrs: usize,
    pub tests: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub adr_centrality: Vec<(String, f64)>,
    pub formal_coverage: usize,
}
