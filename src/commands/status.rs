//! Status summary, impact analysis.

use product_lib::{config::ProductConfig, error::ProductError, graph, types};

use super::{load_graph, BoxResult};

pub(crate) fn handle_impact(id: &str, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    if !graph.all_ids().contains(id) {
        return Err(Box::new(ProductError::NotFound(format!("artifact {}", id))));
    }
    let impact = graph.impact(id);
    if fmt == "json" {
        let obj = serde_json::json!({
            "seed": impact.seed,
            "direct_features": impact.direct_features,
            "direct_tests": impact.direct_tests,
            "transitive_features": impact.transitive_features,
            "transitive_tests": impact.transitive_tests,
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else {
        impact.print(&graph);
    }
    Ok(())
}

pub(crate) fn handle_status(phase: Option<u32>, untested: bool, failing: bool, fmt: &str) -> BoxResult {
    let (config, _, graph) = load_graph()?;

    if untested {
        return status_untested(&graph, fmt);
    }
    if failing {
        return status_failing(&graph, fmt);
    }

    status_full(phase, &config, &graph, fmt)
}

fn status_untested(graph: &graph::KnowledgeGraph, fmt: &str) -> BoxResult {
    let items: Vec<&types::Feature> = graph
        .features
        .values()
        .filter(|f| f.front.status != types::FeatureStatus::Abandoned && f.front.tests.is_empty())
        .collect();
    if fmt == "json" {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|f| {
                serde_json::json!({
                    "id": f.front.id,
                    "title": f.front.title,
                    "phase": f.front.phase,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else {
        println!("Features with no linked test criteria:");
        for f in &items {
            println!("  {} — {} (phase {})", f.front.id, f.front.title, f.front.phase);
        }
    }
    Ok(())
}

fn status_failing(graph: &graph::KnowledgeGraph, fmt: &str) -> BoxResult {
    let items: Vec<&types::Feature> = graph
        .features
        .values()
        .filter(|f| {
            f.front.tests.iter().any(|tid| {
                graph
                    .tests
                    .get(tid.as_str())
                    .map(|t| t.front.status == types::TestStatus::Failing)
                    .unwrap_or(false)
            })
        })
        .collect();
    if fmt == "json" {
        let arr: Vec<serde_json::Value> = items
            .iter()
            .map(|f| {
                serde_json::json!({
                    "id": f.front.id,
                    "title": f.front.title,
                    "phase": f.front.phase,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else {
        println!("Features with failing tests:");
        for f in &items {
            println!("  {} — {} (phase {})", f.front.id, f.front.title, f.front.phase);
        }
    }
    Ok(())
}

fn status_full(
    phase: Option<u32>,
    config: &ProductConfig,
    graph: &graph::KnowledgeGraph,
    fmt: &str,
) -> BoxResult {
    let phases = collect_phases(graph);
    let topo_order = build_topo_order(graph);

    if fmt == "json" {
        status_full_json(phase, config, graph, &phases, &topo_order)
    } else {
        status_full_text(phase, config, graph, &phases, &topo_order)
    }
}

fn collect_phases(graph: &graph::KnowledgeGraph) -> Vec<u32> {
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

fn build_topo_order(graph: &graph::KnowledgeGraph) -> std::collections::HashMap<String, usize> {
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

fn status_full_json(
    phase: Option<u32>,
    config: &ProductConfig,
    graph: &graph::KnowledgeGraph,
    phases: &[u32],
    topo_order: &std::collections::HashMap<String, usize>,
) -> BoxResult {
    let mut phase_arr: Vec<serde_json::Value> = Vec::new();
    for p in phases {
        if let Some(filter_phase) = phase {
            if *p != filter_phase {
                continue;
            }
        }
        phase_arr.push(build_phase_json(*p, config, graph, topo_order));
    }
    let obj = serde_json::json!({
        "project": config.name,
        "phases": phase_arr,
    });
    println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    Ok(())
}

fn build_phase_json(
    p: u32,
    config: &ProductConfig,
    graph: &graph::KnowledgeGraph,
    topo_order: &std::collections::HashMap<String, usize>,
) -> serde_json::Value {
    let mut phase_features: Vec<&types::Feature> = graph
        .features
        .values()
        .filter(|f| f.front.phase == p)
        .collect();
    phase_features.sort_by_key(|f| topo_order.get(&f.front.id).copied().unwrap_or(usize::MAX));
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
    let gate_status = if gate.is_open() { "OPEN" } else { "LOCKED" };
    let features_json: Vec<serde_json::Value> = phase_features
        .iter()
        .map(|f| build_feature_json(f, graph))
        .collect();
    serde_json::json!({
        "phase": p,
        "name": name,
        "complete": complete,
        "total": total,
        "gate": gate_status,
        "features": features_json,
    })
}

fn build_feature_json(f: &types::Feature, graph: &graph::KnowledgeGraph) -> serde_json::Value {
    let test_count = f.front.tests.len();
    let passing = count_passing_tests(f, graph);
    serde_json::json!({
        "id": f.front.id,
        "title": f.front.title,
        "status": f.front.status.to_string(),
        "tests_passing": passing,
        "tests_total": test_count,
    })
}

fn count_passing_tests(f: &types::Feature, graph: &graph::KnowledgeGraph) -> usize {
    f.front
        .tests
        .iter()
        .filter(|tid| {
            graph
                .tests
                .get(tid.as_str())
                .map(|t| t.front.status == types::TestStatus::Passing)
                .unwrap_or(false)
        })
        .count()
}

fn status_full_text(
    phase: Option<u32>,
    config: &ProductConfig,
    graph: &graph::KnowledgeGraph,
    phases: &[u32],
    topo_order: &std::collections::HashMap<String, usize>,
) -> BoxResult {
    println!("Project Status: {}", config.name);
    println!("=================");
    println!();

    for p in phases {
        if let Some(filter_phase) = phase {
            if *p != filter_phase {
                continue;
            }
        }
        print_phase_text(*p, phase.is_some(), config, graph, topo_order);
    }
    Ok(())
}

fn print_phase_text(
    p: u32,
    show_exit_criteria: bool,
    config: &ProductConfig,
    graph: &graph::KnowledgeGraph,
    topo_order: &std::collections::HashMap<String, usize>,
) {
    let mut phase_features: Vec<&types::Feature> = graph
        .features
        .values()
        .filter(|f| f.front.phase == p)
        .collect();
    phase_features.sort_by_key(|f| topo_order.get(&f.front.id).copied().unwrap_or(usize::MAX));

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
    let gate_label = match &gate {
        graph::PhaseGateStatus::Open { .. } => "[OPEN]".to_string(),
        graph::PhaseGateStatus::Locked { failing, .. } => {
            format!("[LOCKED \u{2014} exit criteria not passing: {}]", failing.join(", "))
        }
    };

    println!("Phase {} \u{2014} {} ({}/{} complete)  {}", p, name, complete, total, gate_label);

    if show_exit_criteria {
        print_exit_criteria(&gate);
    }

    for f in &phase_features {
        let passing = count_passing_tests(f, graph);
        let test_count = f.front.tests.len();
        println!(
            "  {} {:<15} {} (tests: {}/{})",
            status_marker(f.front.status),
            f.front.id,
            f.front.title,
            passing,
            test_count,
        );
    }
    println!();
}

fn print_exit_criteria(gate: &graph::PhaseGateStatus) {
    let exit_criteria = match gate {
        graph::PhaseGateStatus::Open { exit_criteria } => exit_criteria,
        graph::PhaseGateStatus::Locked { exit_criteria, .. } => exit_criteria,
    };
    if !exit_criteria.is_empty() {
        println!();
        println!("  Exit criteria:");
        for tc in exit_criteria {
            let mark = if tc.passing { "passing  \u{2713}" } else { "failing  \u{2717}" };
            println!("    {}  {}  [{}]", tc.id, tc.title, mark);
        }
        println!();
    }
}

fn status_marker(status: types::FeatureStatus) -> &'static str {
    match status {
        types::FeatureStatus::Complete => "[x]",
        types::FeatureStatus::InProgress => "[~]",
        types::FeatureStatus::Planned => "[ ]",
        types::FeatureStatus::Abandoned => "[-]",
    }
}
