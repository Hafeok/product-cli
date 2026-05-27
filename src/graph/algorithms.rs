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

    /// Compute betweenness centrality over the legacy node set
    /// (FT / ADR / TC / DEP). Patterns are excluded so pre-FT-071 output
    /// remains byte-identical for `product graph central` without
    /// `--include patterns` (ADR-050 backwards-compat invariant).
    pub fn betweenness_centrality(&self) -> HashMap<String, f64> {
        self.betweenness_centrality_with(false)
    }

    /// Compute betweenness centrality. When `include_patterns` is `true`,
    /// PAT nodes and their `Requires` / `Exemplifies` / `OperationalisedBy`
    /// / `UsesPattern` edges participate in the algorithm (FT-071).
    pub fn betweenness_centrality_with(&self, include_patterns: bool) -> HashMap<String, f64> {
        let all_ids: Vec<String> = if include_patterns {
            self.all_ids().into_iter().collect()
        } else {
            self.all_ids()
                .into_iter()
                .filter(|id| !self.patterns.contains_key(id))
                .collect()
        };
        let n = all_ids.len();
        let mut centrality: HashMap<String, f64> = HashMap::new();

        for id in &all_ids {
            centrality.insert(id.clone(), 0.0);
        }

        if n <= 2 {
            return centrality;
        }

        let adj = self.build_undirected_adjacency_filtered(include_patterns);

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
        let mut buckets = ImpactBuckets::default();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(seed.to_string());
        self.impact_direct_reverse(seed, &mut visited, &mut queue, &mut buckets);
        if self.patterns.contains_key(seed) {
            self.impact_pattern_extras(seed, &mut visited, &mut buckets);
        }
        let (transitive_features, transitive_tests) =
            self.collect_transitive_dependents(&mut visited, &mut queue);
        ImpactResult {
            seed: seed.to_string(),
            direct_features: buckets.features,
            direct_tests: buckets.tests,
            direct_adrs: buckets.adrs,
            direct_deps: buckets.deps,
            direct_patterns: buckets.patterns,
            transitive_features,
            transitive_tests,
        }
    }

    fn impact_direct_reverse(
        &self,
        seed: &str,
        visited: &mut HashSet<String>,
        queue: &mut VecDeque<String>,
        b: &mut ImpactBuckets,
    ) {
        if let Some(neighbors) = self.reverse.get(seed) {
            for (id, _) in neighbors {
                if !visited.contains(id) {
                    visited.insert(id.clone());
                    classify_neighbour(self, id, b);
                    queue.push_back(id.clone());
                }
            }
        }
    }

    /// FT-071 extra hop: when the seed is a PAT, also walk to the ADRs the
    /// pattern operationalises (forward edges + the front-matter `adrs`
    /// array, so the impact tree shows every ADR cited by the pattern even
    /// when the back-edge from the ADR has not been materialised yet).
    fn impact_pattern_extras(
        &self,
        seed: &str,
        visited: &mut HashSet<String>,
        b: &mut ImpactBuckets,
    ) {
        if let Some(forward) = self.forward.get(seed) {
            for (id, _) in forward {
                if !visited.contains(id) && self.adrs.contains_key(id) {
                    visited.insert(id.clone());
                    b.adrs.push(id.clone());
                }
            }
        }
        if let Some(pat) = self.patterns.get(seed) {
            for adr_id in &pat.front.adrs {
                if visited.insert(adr_id.clone()) && self.adrs.contains_key(adr_id) {
                    b.adrs.push(adr_id.clone());
                }
            }
        }
    }

    /// Build undirected adjacency list from graph edges. Optionally
    /// drops every edge touching a PAT node so that
    /// `betweenness_centrality_with(false)` reproduces the pre-FT-071
    /// ranking byte-for-byte (ADR-050 backwards-compat invariant).
    fn build_undirected_adjacency_filtered(
        &self,
        include_patterns: bool,
    ) -> HashMap<String, Vec<String>> {
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            if !include_patterns
                && (self.patterns.contains_key(&edge.from)
                    || self.patterns.contains_key(&edge.to))
            {
                continue;
            }
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

/// Direct-impact buckets, one vector per artifact kind (FT-071).
#[derive(Debug, Default)]
struct ImpactBuckets {
    features: Vec<String>,
    tests: Vec<String>,
    adrs: Vec<String>,
    deps: Vec<String>,
    patterns: Vec<String>,
}

/// Classify a neighbour ID into one of the impact buckets (FT-071).
fn classify_neighbour(graph: &KnowledgeGraph, id: &str, b: &mut ImpactBuckets) {
    if graph.features.contains_key(id) {
        b.features.push(id.to_string());
    } else if graph.tests.contains_key(id) {
        b.tests.push(id.to_string());
    } else if graph.adrs.contains_key(id) {
        b.adrs.push(id.to_string());
    } else if graph.dependencies.contains_key(id) {
        b.deps.push(id.to_string());
    } else if graph.patterns.contains_key(id) {
        b.patterns.push(id.to_string());
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
