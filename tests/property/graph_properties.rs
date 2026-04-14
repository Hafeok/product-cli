//! TC-P005–P009: Graph algorithm property tests (ADR-018)

use proptest::prelude::*;
use product_lib::graph::KnowledgeGraph;
use product_lib::types::*;
use std::path::PathBuf;
use std::collections::HashSet;

fn make_feature(id: &str, deps: Vec<String>) -> Feature {
    Feature {
        front: FeatureFrontMatter {
            id: id.to_string(),
            title: format!("Feature {}", id),
            phase: 1,
            status: FeatureStatus::Planned,
            depends_on: deps,
            adrs: vec![],
            tests: vec![],
            domains: vec![],
            domains_acknowledged: std::collections::HashMap::new(),
            bundle: None,
        },
        body: String::new(),
        path: PathBuf::from(format!("{}.md", id)),
    }
}

/// Generate a valid DAG: only add edges from lower-index to higher-index nodes
fn arb_dag(size: usize, edge_density: f64) -> Vec<Feature> {
    let mut features = Vec::new();
    for i in 0..size {
        let id = format!("FT-{:03}", i + 1);
        let mut deps = Vec::new();
        for j in 0..i {
            // Add edge from j -> i with probability edge_density
            let hash = ((i * 31 + j * 17) % 100) as f64 / 100.0;
            if hash < edge_density {
                deps.push(format!("FT-{:03}", j + 1));
            }
        }
        features.push(make_feature(&id, deps));
    }
    features
}

/// TC-P005: Topo order respects all dependency edges
/// ∀g:DAG, (u,v)∈g.edges: pos(topo(g),u) < pos(topo(g),v)
#[test]
fn tc_p005_topo_order_respects_edges() {
    for size in [5, 10, 20, 50] {
        for density in [0.1, 0.3, 0.5] {
            let features = arb_dag(size, density);
            let graph = KnowledgeGraph::build(features.clone(), vec![], vec![]);
            let order = graph.topological_sort().expect("DAG should not have cycles");

            let pos: std::collections::HashMap<String, usize> = order
                .iter()
                .enumerate()
                .map(|(i, id)| (id.clone(), i))
                .collect();

            for f in &features {
                for dep in &f.front.depends_on {
                    let dep_pos = pos.get(dep).expect("dep in order");
                    let f_pos = pos.get(&f.front.id).expect("feature in order");
                    assert!(
                        dep_pos < f_pos,
                        "Dependency {} (pos {}) must come before {} (pos {})",
                        dep, dep_pos, f.front.id, f_pos
                    );
                }
            }
        }
    }
}

/// TC-P006: Topo sort detects all cycles
/// ∀g:CyclicGraph: topo_sort(g) = Err(E003)
#[test]
fn tc_p006_cycle_always_detected() {
    // Generate cyclic graphs of various sizes
    for size in [2, 3, 5, 10] {
        let mut features = Vec::new();
        for i in 0..size {
            let next = (i + 1) % size;
            features.push(make_feature(
                &format!("FT-{:03}", i + 1),
                vec![format!("FT-{:03}", next + 1)],
            ));
        }
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        assert!(
            graph.topological_sort().is_err(),
            "Cycle of size {} should be detected",
            size
        );
    }
}

/// TC-P007: Centrality always in range [0.0, 1.0]
/// ∀g:ConnectedGraph, n∈g.nodes: 0.0 ≤ centrality(g,n) ≤ 1.0
#[test]
fn tc_p007_centrality_in_range() {
    for size in [3, 5, 10, 20, 50] {
        let features = arb_dag(size, 0.3);
        let adrs: Vec<product_lib::types::Adr> = (0..size / 2)
            .map(|i| product_lib::types::Adr {
                front: AdrFrontMatter {
                    id: format!("ADR-{:03}", i + 1),
                    title: format!("ADR {}", i + 1),
                    status: AdrStatus::Accepted,
                    features: vec![],
                    supersedes: vec![],
                    superseded_by: vec![],
                    domains: vec![],
                    scope: product_lib::types::AdrScope::FeatureSpecific,
                },
                body: String::new(),
                path: PathBuf::from(format!("ADR-{:03}.md", i + 1)),
            })
            .collect();

        let graph = KnowledgeGraph::build(features, adrs, vec![]);
        let centrality = graph.betweenness_centrality();

        for (id, c) in &centrality {
            assert!(
                *c >= 0.0 && *c <= 1.0,
                "Centrality of {} = {} is out of range [0,1]",
                id, c
            );
        }
    }
}

/// TC-P009: BFS deduplication — node appears at most once
/// ∀g:Graph, seed:Node, d:Depth: |set(bfs)| = |bfs|
#[test]
fn tc_p009_bfs_dedup() {
    for size in [5, 10, 20] {
        let features = arb_dag(size, 0.4);
        let graph = KnowledgeGraph::build(features, vec![], vec![]);

        for depth in [1, 2, 3] {
            if let Some(seed) = graph.features.keys().next() {
                let result = graph.bfs(seed, depth);
                let unique: HashSet<&String> = result.iter().collect();
                assert_eq!(
                    result.len(),
                    unique.len(),
                    "BFS from {} depth {} has duplicates: {:?}",
                    seed, depth, result
                );
            }
        }
    }
}
