//! Pattern requires-DAG topological sort (FT-071, ADR-050).
//!
//! Pure module: given a knowledge graph and a list of seed PAT ids, returns
//! every transitively-required pattern in topological order so that each
//! prerequisite precedes its dependants in the rendered bundle. Cycles are
//! reported as `RequiresCycle` for the graph-check E-tier diagnostic.

use crate::graph::KnowledgeGraph;
use std::collections::{HashMap, HashSet};

/// Outcome of a topo sort over the `requires:` DAG.
#[derive(Debug, Clone)]
pub enum TopoResult {
    /// Patterns in topo order — prerequisites first.
    Ordered(Vec<String>),
    /// A cycle was detected along the path described by `cycle`.
    Cycle(Vec<String>),
}

/// Collect every pattern id transitively reachable from `seeds` along the
/// `requires:` edges. Returns the deduplicated set in arbitrary order; the
/// topo sort runs separately.
pub fn collect_transitive(graph: &KnowledgeGraph, seeds: &[String]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = Vec::new();
    for id in seeds {
        if graph.patterns.contains_key(id) && seen.insert(id.clone()) {
            stack.push(id.clone());
        }
    }
    let mut result: Vec<String> = Vec::new();
    while let Some(id) = stack.pop() {
        result.push(id.clone());
        if let Some(pat) = graph.patterns.get(&id) {
            for req in &pat.front.requires {
                if graph.patterns.contains_key(req) && seen.insert(req.clone()) {
                    stack.push(req.clone());
                }
            }
        }
    }
    result
}

/// Topologically sort `ids` over `requires:` edges. The result lists every
/// prerequisite before its dependant — i.e. if PAT-B requires PAT-A, then
/// PAT-A precedes PAT-B in the output. Returns `TopoResult::Cycle` if any
/// cycle exists involving the supplied ids.
pub fn topo_sort_patterns(graph: &KnowledgeGraph, ids: &[String]) -> TopoResult {
    let id_set: HashSet<String> = ids.iter().cloned().collect();
    let (out_edges, mut in_degree) = build_requires_adjacency(graph, &id_set);
    let ordered = kahn_drain(&out_edges, &mut in_degree);
    if ordered.len() == id_set.len() {
        return TopoResult::Ordered(ordered);
    }
    let unsorted: Vec<String> = in_degree
        .iter()
        .filter(|(_, d)| **d > 0)
        .map(|(k, _)| k.clone())
        .collect();
    let cycle = extract_cycle(graph, &unsorted, &id_set).unwrap_or(unsorted);
    TopoResult::Cycle(cycle)
}

/// Build the prerequisite → dependant adjacency map and the in-degree map
/// for the working set of pattern ids.
fn build_requires_adjacency(
    graph: &KnowledgeGraph,
    id_set: &HashSet<String>,
) -> (HashMap<String, Vec<String>>, HashMap<String, usize>) {
    let mut out_edges: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    for id in id_set {
        out_edges.entry(id.clone()).or_default();
        in_degree.entry(id.clone()).or_insert(0);
    }
    for id in id_set {
        if let Some(pat) = graph.patterns.get(id) {
            for req in &pat.front.requires {
                if id_set.contains(req) {
                    out_edges.entry(req.clone()).or_default().push(id.clone());
                    *in_degree.entry(id.clone()).or_insert(0) += 1;
                }
            }
        }
    }
    (out_edges, in_degree)
}

/// Kahn's algorithm — drain zero-in-degree nodes deterministically by
/// always taking the smallest-id-first so the bundle output is stable.
fn kahn_drain(
    out_edges: &HashMap<String, Vec<String>>,
    in_degree: &mut HashMap<String, usize>,
) -> Vec<String> {
    let mut available: Vec<String> = in_degree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(k, _)| k.clone())
        .collect();
    available.sort();
    let mut ordered: Vec<String> = Vec::new();
    while let Some(node) = take_smallest(&mut available) {
        ordered.push(node.clone());
        if let Some(dependants) = out_edges.get(&node) {
            for d in dependants {
                if let Some(deg) = in_degree.get_mut(d) {
                    if *deg > 0 {
                        *deg -= 1;
                        if *deg == 0 {
                            available.push(d.clone());
                            available.sort();
                        }
                    }
                }
            }
        }
    }
    ordered
}

/// Detect any cycle in the entire patterns subgraph. Returns the cycle as a
/// node path (`A -> B -> A`).
pub fn detect_any_cycle(graph: &KnowledgeGraph) -> Option<Vec<String>> {
    let all_ids: Vec<String> = graph.patterns.keys().cloned().collect();
    let id_set: HashSet<String> = all_ids.iter().cloned().collect();
    if let TopoResult::Cycle(c) = topo_sort_patterns(graph, &all_ids) {
        return Some(c);
    }
    // Above is the canonical path; the fallback below catches degenerate
    // cases by per-node DFS.
    for start in &all_ids {
        let mut path: Vec<String> = Vec::new();
        let mut visiting: HashSet<String> = HashSet::new();
        if let Some(cycle) = dfs_find_cycle(graph, start, &id_set, &mut visiting, &mut path) {
            return Some(cycle);
        }
    }
    None
}

fn take_smallest(v: &mut Vec<String>) -> Option<String> {
    if v.is_empty() {
        return None;
    }
    // Find smallest by string order.
    let mut idx = 0;
    for i in 1..v.len() {
        if v[i] < v[idx] {
            idx = i;
        }
    }
    Some(v.remove(idx))
}

fn extract_cycle(
    graph: &KnowledgeGraph,
    candidates: &[String],
    id_set: &HashSet<String>,
) -> Option<Vec<String>> {
    for start in candidates {
        let mut path: Vec<String> = Vec::new();
        let mut visiting: HashSet<String> = HashSet::new();
        if let Some(c) = dfs_find_cycle(graph, start, id_set, &mut visiting, &mut path) {
            return Some(c);
        }
    }
    None
}

fn dfs_find_cycle(
    graph: &KnowledgeGraph,
    node: &str,
    id_set: &HashSet<String>,
    visiting: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if visiting.contains(node) {
        // Trim the path back to where the cycle started.
        if let Some(pos) = path.iter().position(|p| p == node) {
            let mut cycle: Vec<String> = path[pos..].to_vec();
            cycle.push(node.to_string());
            return Some(cycle);
        }
        return Some(vec![node.to_string(), node.to_string()]);
    }
    visiting.insert(node.to_string());
    path.push(node.to_string());
    if let Some(pat) = graph.patterns.get(node) {
        for req in &pat.front.requires {
            if id_set.contains(req) {
                if let Some(c) = dfs_find_cycle(graph, req, id_set, visiting, path) {
                    return Some(c);
                }
            }
        }
    }
    path.pop();
    visiting.remove(node);
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Pattern, PatternFrontMatter, PatternStatus};
    use std::path::PathBuf;

    fn mk_pat(id: &str, requires: Vec<&str>) -> Pattern {
        Pattern {
            front: PatternFrontMatter {
                id: id.into(),
                title: id.into(),
                status: PatternStatus::Live,
                domains: vec![],
                adrs: vec![],
                requires: requires.into_iter().map(String::from).collect(),
                examples: vec![],
                deprecated_by: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("docs/patterns/{}.md", id)),
        }
    }

    fn build_graph(pats: Vec<Pattern>) -> KnowledgeGraph {
        KnowledgeGraph::build_full(vec![], vec![], vec![], vec![], pats)
    }

    #[test]
    fn topo_orders_prerequisites_first() {
        let g = build_graph(vec![
            mk_pat("PAT-001", vec![]),
            mk_pat("PAT-002", vec!["PAT-001"]),
            mk_pat("PAT-003", vec!["PAT-001", "PAT-002"]),
        ]);
        let ids = vec!["PAT-001".into(), "PAT-002".into(), "PAT-003".into()];
        match topo_sort_patterns(&g, &ids) {
            TopoResult::Ordered(o) => {
                assert_eq!(o, vec!["PAT-001", "PAT-002", "PAT-003"]);
            }
            TopoResult::Cycle(c) => panic!("unexpected cycle: {:?}", c),
        }
    }

    #[test]
    fn topo_detects_cycle() {
        let g = build_graph(vec![
            mk_pat("PAT-001", vec!["PAT-002"]),
            mk_pat("PAT-002", vec!["PAT-001"]),
        ]);
        let ids = vec!["PAT-001".into(), "PAT-002".into()];
        match topo_sort_patterns(&g, &ids) {
            TopoResult::Cycle(c) => {
                assert!(c.contains(&"PAT-001".to_string()));
                assert!(c.contains(&"PAT-002".to_string()));
            }
            TopoResult::Ordered(o) => panic!("expected cycle, got {:?}", o),
        }
    }

    #[test]
    fn collect_transitive_walks_requires() {
        let g = build_graph(vec![
            mk_pat("PAT-001", vec![]),
            mk_pat("PAT-002", vec!["PAT-001"]),
            mk_pat("PAT-003", vec!["PAT-002"]),
        ]);
        let seeds = vec!["PAT-003".into()];
        let found = collect_transitive(&g, &seeds);
        assert!(found.contains(&"PAT-001".to_string()));
        assert!(found.contains(&"PAT-002".to_string()));
        assert!(found.contains(&"PAT-003".to_string()));
    }

    #[test]
    fn detect_any_cycle_finds_loop() {
        let g = build_graph(vec![
            mk_pat("PAT-001", vec!["PAT-002"]),
            mk_pat("PAT-002", vec!["PAT-001"]),
        ]);
        let c = detect_any_cycle(&g).expect("expected cycle");
        assert!(c.contains(&"PAT-001".to_string()));
    }
}
