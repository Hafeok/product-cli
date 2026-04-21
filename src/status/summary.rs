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
    /// Whether the cycle-time column should be rendered (FT-054, ADR-046 §12).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub show_cycle_time_column: bool,
    /// Recent-N median used for in-progress reference labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_median_days: Option<f64>,
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
    /// Optional commitment date, ISO 8601 (FT-053 / ADR-045). Absent when
    /// the feature has no `due-date` field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    /// True when `due-date < today AND status != complete`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub overdue: bool,
    /// Completed features: cycle time in days with one decimal (FT-054).
    /// In-progress features: elapsed-so-far. None when not applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_time_days: Option<f64>,
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
    build_project_summary_with_cycle_times(config, graph, phase_filter, None, None, 0)
}

/// Like `build_project_summary` but with cycle-time data. If
/// `show_cycle_time_column` is true the per-feature cycle_time_days is
/// populated (FT-054, ADR-046 §12).
pub fn build_project_summary_with_cycle_times(
    config: &ProductConfig,
    graph: &KnowledgeGraph,
    phase_filter: Option<u32>,
    tag_ts: Option<&crate::cycle_times::TagTimestamps>,
    recent_median: Option<f64>,
    complete_count: usize,
) -> ProjectSummary {
    let phases_all = collect_phases(graph);
    let topo = topo_order(graph);
    let show_column =
        tag_ts.is_some() && complete_count >= config.cycle_times.min_features;
    let phases = phases_all
        .into_iter()
        .filter(|p| phase_filter.is_none_or(|f| *p == f))
        .map(|p| {
            build_phase_summary(
                p,
                config,
                graph,
                &topo,
                if show_column { tag_ts } else { None },
            )
        })
        .collect();
    ProjectSummary {
        project: config.name.clone(),
        phases,
        show_cycle_time_column: show_column,
        recent_median_days: if show_column { recent_median } else { None },
    }
}

/// Pure: features with no linked test criteria (excluding abandoned).
pub fn build_untested_list(graph: &KnowledgeGraph) -> FeatureList {
    let items = graph
        .features
        .values()
        .filter(|f| f.front.status != types::FeatureStatus::Abandoned && f.front.tests.is_empty())
        .map(|f| feature_row(f, graph, None))
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
        .map(|f| feature_row(f, graph, None))
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
    tag_ts: Option<&crate::cycle_times::TagTimestamps>,
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
        .map(|f| feature_row(f, graph, tag_ts))
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

fn feature_row(
    f: &types::Feature,
    graph: &KnowledgeGraph,
    tag_ts: Option<&crate::cycle_times::TagTimestamps>,
) -> FeatureRow {
    feature_row_with_tags(f, graph, tag_ts)
}

/// Internal: compute per-feature row including optional cycle-time days.
fn feature_row_with_tags(
    f: &types::Feature,
    graph: &KnowledgeGraph,
    tag_ts: Option<&crate::cycle_times::TagTimestamps>,
) -> FeatureRow {
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
    let (due_date, overdue) = match f.front.due_date {
        Some(d) => {
            let today = chrono::Local::now().date_naive();
            let overdue =
                d < today && f.front.status != types::FeatureStatus::Complete;
            (Some(d.format("%Y-%m-%d").to_string()), overdue)
        }
        None => (None, false),
    };

    // Cycle time — computed from tag timestamps if present.
    let cycle_time_days = compute_feature_cycle_days(f, tag_ts);

    FeatureRow {
        id: f.front.id.clone(),
        title: f.front.title.clone(),
        phase: f.front.phase,
        status: f.front.status.to_string(),
        tests_passing,
        tests_total,
        due_date,
        overdue,
        cycle_time_days,
    }
}

fn compute_feature_cycle_days(
    f: &types::Feature,
    tag_ts: Option<&crate::cycle_times::TagTimestamps>,
) -> Option<f64> {
    let ts = tag_ts?.get(&f.front.id)?;
    let (started, completed) = ts;
    match f.front.status {
        types::FeatureStatus::Complete => {
            let st = crate::cycle_times::parse_instant(started.as_deref()?)?;
            let cp = crate::cycle_times::parse_instant(completed.as_deref()?)?;
            let days = crate::cycle_times::elapsed_days(&st, &cp);
            Some(crate::cycle_times::round1(days))
        }
        types::FeatureStatus::InProgress => {
            let st = crate::cycle_times::parse_instant(started.as_deref()?)?;
            let now = chrono::Local::now().fixed_offset();
            let days = crate::cycle_times::elapsed_days(&st, &now).max(0.0);
            Some(crate::cycle_times::round1(days))
        }
        _ => None,
    }
}
