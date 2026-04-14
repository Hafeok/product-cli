//! Graph algorithms — BFS traversal, betweenness centrality, impact analysis.

use super::model::KnowledgeGraph;
use super::types::ImpactResult;
use std::collections::{HashMap, HashSet, VecDeque};

impl KnowledgeGraph {
    // -----------------------------------------------------------------------
    // BFS context assembly (ADR-012)
    // -----------------------------------------------------------------------

    /// BFS from a seed node to depth N, returning all reachable node IDs (deduplicated)
    pub fn bfs(&self, seed: &str, depth: usize) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        visited.insert(seed.to_string());
        queue.push_back((seed.to_string(), 0));

        while let Some((node, d)) = queue.pop_front() {
            result.push(node.clone());
            if d >= depth {
                continue;
            }
            if let Some(neighbors) = self.forward.get(&node) {
                for (next, _) in neighbors {
                    if !visited.contains(next) {
                        visited.insert(next.clone());
                        queue.push_back((next.clone(), d + 1));
                    }
                }
            }
            // Also follow reverse edges for context assembly
            if let Some(neighbors) = self.reverse.get(&node) {
                for (next, _) in neighbors {
                    if !visited.contains(next) {
                        visited.insert(next.clone());
                        queue.push_back((next.clone(), d + 1));
                    }
                }
            }
        }

        result
    }

    // -----------------------------------------------------------------------
    // Betweenness centrality (Brandes' algorithm) (ADR-012)
    // -----------------------------------------------------------------------

    /// Compute betweenness centrality for all ADR nodes
    pub fn betweenness_centrality(&self) -> HashMap<String, f64> {
        let all_ids: Vec<String> = self.all_ids().into_iter().collect();
        let n = all_ids.len();
        let mut centrality: HashMap<String, f64> = HashMap::new();

        for id in &all_ids {
            centrality.insert(id.clone(), 0.0);
        }

        if n <= 2 {
            return centrality;
        }

        let adj = self.build_undirected_adjacency();

        for s in &all_ids {
            brandes_accumulate(s, &all_ids, &adj, &mut centrality);
        }

        normalize_centrality(&mut centrality, n);
        centrality
    }

    // -----------------------------------------------------------------------
    // Impact analysis — reverse-graph BFS (ADR-012)
    // -----------------------------------------------------------------------

    /// Compute all nodes affected if `seed` changes
    pub fn impact(&self, seed: &str) -> ImpactResult {
        let mut visited = HashSet::new();
        let mut direct_features = Vec::new();
        let mut direct_tests = Vec::new();
        let mut direct_adrs = Vec::new();
        let mut direct_deps = Vec::new();
        let mut queue = VecDeque::new();

        visited.insert(seed.to_string());

        // Direct dependents (depth 1 via reverse edges)
        if let Some(neighbors) = self.reverse.get(seed) {
            for (id, _) in neighbors {
                if !visited.contains(id) {
                    visited.insert(id.clone());
                    if self.features.contains_key(id) {
                        direct_features.push(id.clone());
                    } else if self.tests.contains_key(id) {
                        direct_tests.push(id.clone());
                    } else if self.adrs.contains_key(id) {
                        direct_adrs.push(id.clone());
                    } else if self.dependencies.contains_key(id) {
                        direct_deps.push(id.clone());
                    }
                    queue.push_back(id.clone());
                }
            }
        }

        // Transitive dependents (depth 2+)
        let (transitive_features, transitive_tests) =
            self.collect_transitive_dependents(&mut visited, &mut queue);

        ImpactResult {
            seed: seed.to_string(),
            direct_features,
            direct_tests,
            direct_adrs,
            direct_deps,
            transitive_features,
            transitive_tests,
        }
    }

    /// Build undirected adjacency list from graph edges
    fn build_undirected_adjacency(&self) -> HashMap<String, Vec<String>> {
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
            adj.entry(edge.to.clone())
                .or_default()
                .push(edge.from.clone());
        }
        adj
    }

    /// BFS through remaining reverse edges to collect transitive dependents
    fn collect_transitive_dependents(
        &self,
        visited: &mut HashSet<String>,
        queue: &mut VecDeque<String>,
    ) -> (Vec<String>, Vec<String>) {
        let mut transitive_features = Vec::new();
        let mut transitive_tests = Vec::new();
        while let Some(node) = queue.pop_front() {
            if let Some(neighbors) = self.reverse.get(&node) {
                for (id, _) in neighbors {
                    if !visited.contains(id) {
                        visited.insert(id.clone());
                        if self.features.contains_key(id) {
                            transitive_features.push(id.clone());
                        } else if self.tests.contains_key(id) {
                            transitive_tests.push(id.clone());
                        }
                        queue.push_back(id.clone());
                    }
                }
            }
        }
        (transitive_features, transitive_tests)
    }
}

/// Intermediate state from the BFS phase of Brandes' algorithm
struct BrandesBfsResult {
    stack: Vec<String>,
    sigma: HashMap<String, f64>,
    predecessors: HashMap<String, Vec<String>>,
}

/// Run one iteration of Brandes' algorithm from source `s`, accumulating into `centrality`
fn brandes_accumulate(
    s: &str,
    all_ids: &[String],
    adj: &HashMap<String, Vec<String>>,
    centrality: &mut HashMap<String, f64>,
) {
    let bfs = brandes_bfs(s, all_ids, adj);
    brandes_backpropagate(s, &bfs.stack, &bfs.sigma, &bfs.predecessors, centrality);
}

/// BFS phase of Brandes' algorithm: compute shortest-path counts and predecessors
fn brandes_bfs(
    s: &str,
    all_ids: &[String],
    adj: &HashMap<String, Vec<String>>,
) -> BrandesBfsResult {
    let mut stack = Vec::new();
    let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
    let mut sigma: HashMap<String, f64> = HashMap::new();
    let mut dist: HashMap<String, i64> = HashMap::new();

    for v in all_ids {
        predecessors.insert(v.clone(), Vec::new());
        sigma.insert(v.clone(), 0.0);
        dist.insert(v.clone(), -1);
    }

    sigma.insert(s.to_string(), 1.0);
    dist.insert(s.to_string(), 0);

    let mut queue = VecDeque::new();
    queue.push_back(s.to_string());

    while let Some(v) = queue.pop_front() {
        stack.push(v.clone());
        let d_v = dist[&v];
        if let Some(neighbors) = adj.get(&v) {
            for w in neighbors {
                let d_w = dist.get(w).copied().unwrap_or(-1);
                if d_w < 0 {
                    dist.insert(w.clone(), d_v + 1);
                    queue.push_back(w.clone());
                }
                if dist.get(w).copied().unwrap_or(-1) == d_v + 1 {
                    *sigma.entry(w.clone()).or_insert(0.0) += sigma[&v];
                    predecessors.entry(w.clone()).or_default().push(v.clone());
                }
            }
        }
    }

    BrandesBfsResult { stack, sigma, predecessors }
}

/// Back-propagation phase of Brandes' algorithm: accumulate dependency scores
fn brandes_backpropagate(
    s: &str,
    stack: &[String],
    sigma: &HashMap<String, f64>,
    predecessors: &HashMap<String, Vec<String>>,
    centrality: &mut HashMap<String, f64>,
) {
    let mut delta: HashMap<String, f64> = HashMap::new();

    for w in stack.iter().rev() {
        if w == s {
            continue;
        }
        let sigma_w = sigma.get(w).copied().unwrap_or(1.0);
        let delta_w = delta.get(w).copied().unwrap_or(0.0);
        if let Some(preds) = predecessors.get(w) {
            for v in preds {
                let sigma_v = sigma.get(v).copied().unwrap_or(1.0);
                let contribution = (sigma_v / sigma_w) * (1.0 + delta_w);
                *delta.entry(v.clone()).or_insert(0.0) += contribution;
            }
        }
        *centrality.entry(w.clone()).or_insert(0.0) += delta_w;
    }
}

/// Normalize centrality values for an undirected graph: divide by (n-1)(n-2)
fn normalize_centrality(centrality: &mut HashMap<String, f64>, n: usize) {
    let norm = if n > 2 {
        ((n - 1) * (n - 2)) as f64
    } else {
        1.0
    };
    for val in centrality.values_mut() {
        *val /= norm;
    }
}
