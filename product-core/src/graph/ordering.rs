//! Topological ordering — Kahn's algorithm for feature scheduling.

use super::model::KnowledgeGraph;
use super::types::{FeatureNextResult, PhaseGateStatus, PhaseGateTC};
use crate::error::{ProductError, Result};
use crate::types::FeatureStatus;
use std::collections::{BTreeSet, HashMap, HashSet};

impl KnowledgeGraph {
    // -----------------------------------------------------------------------
    // Topological sort (Kahn's algorithm) on depends-on DAG (ADR-012)
    // -----------------------------------------------------------------------

    /// Returns features in topological order based on depends-on edges.
    /// Returns Err if a cycle is detected.
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        let (mut in_degree, adj) = self.build_depends_on_graph();
        let mut ready = self.initial_ready_set(&in_degree);

        let mut result = Vec::new();
        while let Some((_, node)) = ready.pop_first() {
            result.push(node.clone());
            drain_neighbors(&node, &adj, &mut in_degree, &mut ready, self);
        }

        if result.len() != self.features.len() {
            let in_result: HashSet<_> = result.iter().collect();
            let cycle: Vec<String> = self
                .features
                .keys()
                .filter(|id| !in_result.contains(id))
                .cloned()
                .collect();
            return Err(ProductError::DependencyCycle { cycle });
        }

        Ok(result)
    }

    /// Build adjacency list and in-degree map for depends-on edges only
    fn build_depends_on_graph(&self) -> (HashMap<String, usize>, HashMap<String, Vec<String>>) {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for id in self.features.keys() {
            in_degree.entry(id.clone()).or_insert(0);
            adj.entry(id.clone()).or_default();
        }

        for f in self.features.values() {
            for dep in &f.front.depends_on {
                if self.features.contains_key(dep) {
                    adj.entry(dep.clone()).or_default().push(f.front.id.clone());
                    *in_degree.entry(f.front.id.clone()).or_insert(0) += 1;
                }
            }
        }

        (in_degree, adj)
    }

    /// Build the initial ready set: all nodes with in-degree 0, keyed by (phase, id)
    fn initial_ready_set(&self, in_degree: &HashMap<String, usize>) -> BTreeSet<(u32, String)> {
        let mut ready: BTreeSet<(u32, String)> = BTreeSet::new();
        for (id, &deg) in in_degree {
            if deg == 0 {
                let phase = self
                    .features
                    .get(id)
                    .map(|f| f.front.phase)
                    .unwrap_or(u32::MAX);
                ready.insert((phase, id.clone()));
            }
        }
        ready
    }

    /// Next feature to implement: first in topo order that is not complete
    /// and whose dependencies are all complete
    pub fn feature_next(&self) -> Result<Option<String>> {
        match self.feature_next_with_gate(false)? {
            FeatureNextResult::Ready(id) => Ok(Some(id)),
            FeatureNextResult::Blocked { .. } => Ok(None),
            FeatureNextResult::AllDone => Ok(None),
        }
    }

    /// Check whether a phase gate is satisfied: all exit-criteria TCs for features
    /// in the given phase must be passing.
    pub fn phase_gate_satisfied(&self, phase: u32) -> PhaseGateStatus {
        let exit_criteria: Vec<&crate::types::TestCriterion> = self.tests.values()
            .filter(|t| {
                t.front.test_type == crate::types::TestType::ExitCriteria
                    && t.front.validates.features.iter().any(|fid| {
                        self.features.get(fid).map(|f| f.front.phase == phase).unwrap_or(false)
                    })
            })
            .collect();

        if exit_criteria.is_empty() {
            return PhaseGateStatus::Open { exit_criteria: Vec::new() };
        }

        build_gate_status(&exit_criteria)
    }

    /// Next feature to implement with phase gate awareness.
    pub fn feature_next_with_gate(&self, ignore_phase_gate: bool) -> Result<FeatureNextResult> {
        let order = self.topological_sort()?;
        let gate_status = self.compute_all_gate_statuses();
        let mut first_gate_blocked: Option<(String, u32, Vec<PhaseGateTC>)> = None;

        for id in &order {
            if let Some(f) = self.features.get(id) {
                if f.front.status == FeatureStatus::Complete
                    || f.front.status == FeatureStatus::Abandoned
                {
                    continue;
                }
                if !self.deps_all_complete(f) {
                    continue;
                }
                if !ignore_phase_gate && f.front.phase > 1 {
                    if let Some(blocked) = find_blocking_gate(id, f.front.phase, &gate_status) {
                        if first_gate_blocked.is_none() {
                            first_gate_blocked = Some(blocked);
                        }
                        continue;
                    }
                }
                return Ok(FeatureNextResult::Ready(id.clone()));
            }
        }

        if let Some((candidate, blocked_phase, exit_criteria)) = first_gate_blocked {
            return Ok(FeatureNextResult::Blocked {
                candidate,
                blocked_phase,
                exit_criteria,
            });
        }

        Ok(FeatureNextResult::AllDone)
    }

    /// Check if all depends-on features are complete
    fn deps_all_complete(&self, f: &crate::types::Feature) -> bool {
        f.front.depends_on.iter().all(|dep| {
            self.features
                .get(dep)
                .map(|d| d.front.status == FeatureStatus::Complete)
                .unwrap_or(true)
        })
    }

    /// Pre-compute phase gate status for all phases in the graph
    fn compute_all_gate_statuses(&self) -> HashMap<u32, PhaseGateStatus> {
        let phases: BTreeSet<u32> = self.features.values().map(|f| f.front.phase).collect();
        phases.into_iter()
            .map(|p| (p, self.phase_gate_satisfied(p)))
            .collect()
    }
}

/// Drain neighbors of a processed node, decrementing in-degree and adding to ready set
fn drain_neighbors(
    node: &str,
    adj: &HashMap<String, Vec<String>>,
    in_degree: &mut HashMap<String, usize>,
    ready: &mut BTreeSet<(u32, String)>,
    graph: &KnowledgeGraph,
) {
    if let Some(neighbors) = adj.get(node) {
        for next in neighbors {
            if let Some(deg) = in_degree.get_mut(next) {
                *deg -= 1;
                if *deg == 0 {
                    let phase = graph
                        .features
                        .get(next)
                        .map(|f| f.front.phase)
                        .unwrap_or(u32::MAX);
                    ready.insert((phase, next.clone()));
                }
            }
        }
    }
}

/// Build PhaseGateStatus from a non-empty list of exit-criteria TCs
fn build_gate_status(exit_criteria: &[&crate::types::TestCriterion]) -> PhaseGateStatus {
    let mut failing_tcs = Vec::new();
    let mut all_tcs = Vec::new();
    for tc in exit_criteria {
        let passing = tc.front.status == crate::types::TestStatus::Passing;
        all_tcs.push(PhaseGateTC {
            id: tc.front.id.clone(),
            title: tc.front.title.clone(),
            passing,
        });
        if !passing {
            failing_tcs.push(tc.front.id.clone());
        }
    }
    if failing_tcs.is_empty() {
        PhaseGateStatus::Open { exit_criteria: all_tcs }
    } else {
        PhaseGateStatus::Locked { exit_criteria: all_tcs, failing: failing_tcs }
    }
}

/// Check if any prior phase gate blocks a feature, returning the first blocking info
fn find_blocking_gate(
    id: &str,
    phase: u32,
    gate_status: &HashMap<u32, PhaseGateStatus>,
) -> Option<(String, u32, Vec<PhaseGateTC>)> {
    for prior_phase in 1..phase {
        if let Some(PhaseGateStatus::Locked { exit_criteria, .. }) = gate_status.get(&prior_phase) {
            return Some((id.to_string(), prior_phase, exit_criteria.clone()));
        }
    }
    None
}
