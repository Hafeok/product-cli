//! Pure builders that derive structured status summaries from a KnowledgeGraph.

use crate::config::ProductConfig;
use crate::graph::{KnowledgeGraph, PhaseGateStatus};
use crate::types;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectSummary {
    pub project: String,
    pub phases: Vec<PhaseSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhaseSummary {
    pub phase: u32,
    pub name: String,
    pub complete: usize,
    pub total: usize,
    pub gate: GateSummary,
    pub features: Vec<FeatureRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateSummary {
    pub is_open: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failing_exit_criteria: Vec<String>,
    pub exit_criteria: Vec<ExitCriterionSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExitCriterionSummary {
    pub id: String,
    pub title: String,
    pub passing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureRow {
    pub id: String,
    pub title: String,
    pub phase: u32,
    pub status: String,
    pub tests_passing: usize,
    pub tests_total: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureList {
    pub items: Vec<FeatureRow>,
}

/// Pure: build the full project summary, optionally filtered to one phase.
pub fn build_project_summary(
    config: &ProductConfig,
    graph: &KnowledgeGraph,
    phase_filter: Option<u32>,
) -> ProjectSummary {
    let phases_all = collect_phases(graph);
    let topo = topo_order(graph);
    let phases = phases_all
        .into_iter()
        .filter(|p| phase_filter.is_none_or(|f| *p == f))
        .map(|p| build_phase_summary(p, config, graph, &topo))
        .collect();
    ProjectSummary {
        project: config.name.clone(),
        phases,
    }
}

/// Pure: features with no linked test criteria (excluding abandoned).
pub fn build_untested_list(graph: &KnowledgeGraph) -> FeatureList {
    let items = graph
        .features
        .values()
        .filter(|f| f.front.status != types::FeatureStatus::Abandoned && f.front.tests.is_empty())
        .map(|f| feature_row(f, graph))
        .collect();
    FeatureList { items }
}

/// Pure: features with at least one failing linked test.
pub fn build_failing_list(graph: &KnowledgeGraph) -> FeatureList {
    let items = graph
        .features
        .values()
        .filter(|f| {
            f.front.tests.iter().any(|tid| {
                graph
                    .tests
                    .get(tid.as_str())
                    .is_some_and(|t| t.front.status == types::TestStatus::Failing)
            })
        })
        .map(|f| feature_row(f, graph))
        .collect();
    FeatureList { items }
}

// ---- internal helpers (all pure) -----------------------------------------

fn collect_phases(graph: &KnowledgeGraph) -> Vec<u32> {
    let mut phases: Vec<u32> = graph
        .features
        .values()
        .map(|f| f.front.phase)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    phases.sort();
    phases
}

fn topo_order(graph: &KnowledgeGraph) -> HashMap<String, usize> {
    graph
        .topological_sort()
        .unwrap_or_else(|_| {
            let mut ids: Vec<String> = graph.features.keys().cloned().collect();
            ids.sort();
            ids
        })
        .into_iter()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect()
}

fn build_phase_summary(
    p: u32,
    config: &ProductConfig,
    graph: &KnowledgeGraph,
    topo: &HashMap<String, usize>,
) -> PhaseSummary {
    let mut phase_features: Vec<&types::Feature> = graph
        .features
        .values()
        .filter(|f| f.front.phase == p)
        .collect();
    phase_features.sort_by_key(|f| topo.get(&f.front.id).copied().unwrap_or(usize::MAX));

    let name = config
        .phases
        .get(&p.to_string())
        .cloned()
        .unwrap_or_else(|| format!("Phase {}", p));
    let complete = phase_features
        .iter()
        .filter(|f| f.front.status == types::FeatureStatus::Complete)
        .count();
    let total = phase_features.len();

    let gate = graph.phase_gate_satisfied(p);
    let gate_summary = gate_summary_from(&gate);
    let features = phase_features
        .iter()
        .map(|f| feature_row(f, graph))
        .collect();

    PhaseSummary {
        phase: p,
        name,
        complete,
        total,
        gate: gate_summary,
        features,
    }
}

fn gate_summary_from(gate: &PhaseGateStatus) -> GateSummary {
    match gate {
        PhaseGateStatus::Open { exit_criteria } => GateSummary {
            is_open: true,
            failing_exit_criteria: Vec::new(),
            exit_criteria: exit_criteria
                .iter()
                .map(|tc| ExitCriterionSummary {
                    id: tc.id.clone(),
                    title: tc.title.clone(),
                    passing: tc.passing,
                })
                .collect(),
        },
        PhaseGateStatus::Locked {
            exit_criteria,
            failing,
        } => GateSummary {
            is_open: false,
            failing_exit_criteria: failing.clone(),
            exit_criteria: exit_criteria
                .iter()
                .map(|tc| ExitCriterionSummary {
                    id: tc.id.clone(),
                    title: tc.title.clone(),
                    passing: tc.passing,
                })
                .collect(),
        },
    }
}

fn feature_row(f: &types::Feature, graph: &KnowledgeGraph) -> FeatureRow {
    let tests_total = f.front.tests.len();
    let tests_passing = f
        .front
        .tests
        .iter()
        .filter(|tid| {
            graph
                .tests
                .get(tid.as_str())
                .is_some_and(|t| t.front.status == types::TestStatus::Passing)
        })
        .count();
    FeatureRow {
        id: f.front.id.clone(),
        title: f.front.title.clone(),
        phase: f.front.phase,
        status: f.front.status.to_string(),
        tests_passing,
        tests_total,
    }
}
