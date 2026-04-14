//! In-memory knowledge graph with graph algorithms (ADR-003, ADR-012)
//!
//! Algorithms: topological sort (Kahn's), BFS context assembly,
//! betweenness centrality (Brandes'), reverse-graph impact analysis.

use crate::error::{CheckResult, Diagnostic, ProductError, Result};
use crate::types::*;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::path::Path;

/// Find the 1-based line number and trimmed content of the line where `needle`
/// first appears in a file. Returns `None` if the file cannot be read or needle
/// is not found.
fn find_reference_line(path: &Path, needle: &str) -> Option<(usize, String)> {
    let content = std::fs::read_to_string(path).ok()?;
    for (i, line) in content.lines().enumerate() {
        if line.contains(needle) {
            return Some((i + 1, line.trim().to_string()));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Graph model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeType {
    ImplementedBy,   // Feature -> ADR
    ValidatedBy,     // Feature -> TestCriterion
    TestedBy,        // ADR -> TestCriterion
    Supersedes,      // ADR -> ADR
    DependsOn,       // Feature -> Feature
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

#[derive(Debug)]
pub struct KnowledgeGraph {
    pub features: HashMap<String, Feature>,
    pub adrs: HashMap<String, Adr>,
    pub tests: HashMap<String, TestCriterion>,
    pub edges: Vec<Edge>,
    // Adjacency lists
    pub forward: HashMap<String, Vec<(String, EdgeType)>>,
    pub reverse: HashMap<String, Vec<(String, EdgeType)>>,
    /// Duplicate IDs detected during build (id -> list of file paths)
    pub duplicates: Vec<(String, Vec<std::path::PathBuf>)>,
    /// Parse errors collected during artifact loading (ADR-013)
    pub parse_errors: Vec<ProductError>,
}

impl KnowledgeGraph {
    /// Build graph from loaded artifacts
    pub fn build(
        features: Vec<Feature>,
        adrs: Vec<Adr>,
        tests: Vec<TestCriterion>,
    ) -> Self {
        let mut graph = Self {
            features: HashMap::new(),
            adrs: HashMap::new(),
            tests: HashMap::new(),
            edges: Vec::new(),
            forward: HashMap::new(),
            reverse: HashMap::new(),
            duplicates: Vec::new(),
            parse_errors: Vec::new(),
        };

        // Track all paths per ID to detect duplicates
        let mut id_paths: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();

        for f in features {
            id_paths.entry(f.front.id.clone()).or_default().push(f.path.clone());
            graph.features.insert(f.front.id.clone(), f);
        }
        for a in adrs {
            id_paths.entry(a.front.id.clone()).or_default().push(a.path.clone());
            graph.adrs.insert(a.front.id.clone(), a);
        }
        for t in tests {
            id_paths.entry(t.front.id.clone()).or_default().push(t.path.clone());
            graph.tests.insert(t.front.id.clone(), t);
        }

        // Record any IDs that appear in more than one file
        for (id, paths) in id_paths {
            if paths.len() > 1 {
                graph.duplicates.push((id, paths));
            }
        }
        graph.duplicates.sort_by(|a, b| a.0.cmp(&b.0));

        // Collect edges first, then add them (avoids borrow conflicts)
        let mut pending_edges: Vec<(String, String, EdgeType)> = Vec::new();

        for f in graph.features.values() {
            for adr_id in &f.front.adrs {
                pending_edges.push((f.front.id.clone(), adr_id.clone(), EdgeType::ImplementedBy));
            }
            for test_id in &f.front.tests {
                pending_edges.push((f.front.id.clone(), test_id.clone(), EdgeType::ValidatedBy));
            }
            for dep_id in &f.front.depends_on {
                pending_edges.push((f.front.id.clone(), dep_id.clone(), EdgeType::DependsOn));
            }
        }

        for a in graph.adrs.values() {
            for sup_id in &a.front.supersedes {
                pending_edges.push((a.front.id.clone(), sup_id.clone(), EdgeType::Supersedes));
            }
        }

        for t in graph.tests.values() {
            for adr_id in &t.front.validates.adrs {
                pending_edges.push((adr_id.clone(), t.front.id.clone(), EdgeType::TestedBy));
            }
        }

        for (from, to, edge_type) in pending_edges {
            graph.add_edge(&from, &to, edge_type);
        }

        graph
    }

    /// Attach parse errors collected during artifact loading.
    /// These will be included as E001 diagnostics by `check()`.
    pub fn with_parse_errors(mut self, errors: Vec<ProductError>) -> Self {
        self.parse_errors = errors;
        self
    }

    fn add_edge(&mut self, from: &str, to: &str, edge_type: EdgeType) {
        self.edges.push(Edge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type,
        });
        self.forward
            .entry(from.to_string())
            .or_default()
            .push((to.to_string(), edge_type));
        self.reverse
            .entry(to.to_string())
            .or_default()
            .push((from.to_string(), edge_type));
    }

    /// All known node IDs
    pub fn all_ids(&self) -> HashSet<String> {
        let mut ids = HashSet::new();
        ids.extend(self.features.keys().cloned());
        ids.extend(self.adrs.keys().cloned());
        ids.extend(self.tests.keys().cloned());
        ids
    }

    // -----------------------------------------------------------------------
    // Topological sort (Kahn's algorithm) on depends-on DAG (ADR-012)
    // -----------------------------------------------------------------------

    /// Returns features in topological order based on depends-on edges.
    /// Returns Err if a cycle is detected.
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        // Build adjacency for depends-on edges only
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

        // Use a BTreeSet keyed by (phase, id) so lower-phase features come first,
        // with alphabetical id as tiebreaker within the same phase.
        let mut ready: BTreeSet<(u32, String)> = BTreeSet::new();
        for (id, &deg) in &in_degree {
            if deg == 0 {
                let phase = self
                    .features
                    .get(id)
                    .map(|f| f.front.phase)
                    .unwrap_or(u32::MAX);
                ready.insert((phase, id.clone()));
            }
        }

        let mut result = Vec::new();
        while let Some((_, node)) = ready.pop_first() {
            result.push(node.clone());
            if let Some(neighbors) = adj.get(&node) {
                for next in neighbors {
                    if let Some(deg) = in_degree.get_mut(next) {
                        *deg -= 1;
                        if *deg == 0 {
                            let phase = self
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

        if result.len() != self.features.len() {
            // Cycle detected — find the cycle members
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
    /// in the given phase must be passing. If no exit-criteria TCs exist, the gate
    /// is considered satisfied (backward compat).
    pub fn phase_gate_satisfied(&self, phase: u32) -> PhaseGateStatus {
        let exit_criteria: Vec<&TestCriterion> = self.tests.values()
            .filter(|t| {
                t.front.test_type == TestType::ExitCriteria
                    && t.front.validates.features.iter().any(|fid| {
                        self.features.get(fid).map(|f| f.front.phase == phase).unwrap_or(false)
                    })
            })
            .collect();

        if exit_criteria.is_empty() {
            return PhaseGateStatus::Open { exit_criteria: Vec::new() };
        }

        let mut failing_tcs = Vec::new();
        let mut all_tcs = Vec::new();
        for tc in &exit_criteria {
            let passing = tc.front.status == TestStatus::Passing;
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

    /// Next feature to implement with phase gate awareness.
    /// Returns detailed result including gate blocking info.
    pub fn feature_next_with_gate(&self, ignore_phase_gate: bool) -> Result<FeatureNextResult> {
        let order = self.topological_sort()?;

        // Collect all phases present in the graph
        let mut phases: Vec<u32> = self.features.values()
            .map(|f| f.front.phase)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        phases.sort();

        // Pre-compute phase gate status for all phases
        let gate_status: HashMap<u32, PhaseGateStatus> = phases.iter()
            .map(|&p| (p, self.phase_gate_satisfied(p)))
            .collect();

        let mut first_gate_blocked: Option<(String, u32, Vec<PhaseGateTC>)> = None;

        for id in &order {
            if let Some(f) = self.features.get(id) {
                if f.front.status == FeatureStatus::Complete
                    || f.front.status == FeatureStatus::Abandoned
                {
                    continue;
                }
                // Check all dependencies are complete
                let deps_complete = f.front.depends_on.iter().all(|dep| {
                    self.features
                        .get(dep)
                        .map(|d| d.front.status == FeatureStatus::Complete)
                        .unwrap_or(true)
                });
                if !deps_complete {
                    continue;
                }

                // Phase gate check: if feature is in phase > 1, all prior phases must be open
                if !ignore_phase_gate && f.front.phase > 1 {
                    let mut gate_blocked = false;
                    for prior_phase in 1..f.front.phase {
                        if let Some(PhaseGateStatus::Locked { exit_criteria, .. }) = gate_status.get(&prior_phase) {
                            if first_gate_blocked.is_none() {
                                first_gate_blocked = Some((
                                    id.clone(),
                                    prior_phase,
                                    exit_criteria.clone(),
                                ));
                            }
                            gate_blocked = true;
                            break;
                        }
                    }
                    if gate_blocked {
                        continue;
                    }
                }

                return Ok(FeatureNextResult::Ready(id.clone()));
            }
        }

        // If we found a gate-blocked candidate but no ready feature, report the block
        if let Some((candidate, blocked_phase, exit_criteria)) = first_gate_blocked {
            return Ok(FeatureNextResult::Blocked {
                candidate,
                blocked_phase,
                exit_criteria,
            });
        }

        Ok(FeatureNextResult::AllDone)
    }

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

        // Build undirected adjacency for centrality computation
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
            adj.entry(edge.to.clone())
                .or_default()
                .push(edge.from.clone());
        }

        // Brandes' algorithm
        for s in &all_ids {
            let mut stack = Vec::new();
            let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
            let mut sigma: HashMap<String, f64> = HashMap::new();
            let mut dist: HashMap<String, i64> = HashMap::new();

            for v in &all_ids {
                predecessors.insert(v.clone(), Vec::new());
                sigma.insert(v.clone(), 0.0);
                dist.insert(v.clone(), -1);
            }

            sigma.insert(s.clone(), 1.0);
            dist.insert(s.clone(), 0);

            let mut queue = VecDeque::new();
            queue.push_back(s.clone());

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

            let mut delta: HashMap<String, f64> = HashMap::new();
            for v in &all_ids {
                delta.insert(v.clone(), 0.0);
            }

            while let Some(w) = stack.pop() {
                if &w == s {
                    continue;
                }
                let sigma_w = sigma.get(&w).copied().unwrap_or(1.0);
                let delta_w = delta.get(&w).copied().unwrap_or(0.0);
                if let Some(preds) = predecessors.get(&w) {
                    for v in preds {
                        let sigma_v = sigma.get(v).copied().unwrap_or(1.0);
                        let contribution = (sigma_v / sigma_w) * (1.0 + delta_w);
                        *delta.entry(v.clone()).or_insert(0.0) += contribution;
                    }
                }
                *centrality.entry(w.clone()).or_insert(0.0) += delta_w;
            }
        }

        // Normalize: divide by (n-1)(n-2) for undirected graph
        let norm = if n > 2 {
            ((n - 1) * (n - 2)) as f64
        } else {
            1.0
        };
        for val in centrality.values_mut() {
            *val /= norm;
        }

        centrality
    }

    // -----------------------------------------------------------------------
    // Impact analysis — reverse-graph BFS (ADR-012)
    // -----------------------------------------------------------------------

    /// Compute all nodes affected if `seed` changes
    pub fn impact(&self, seed: &str) -> ImpactResult {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut direct_features = Vec::new();
        let mut direct_tests = Vec::new();
        let mut transitive_features = Vec::new();
        let mut transitive_tests = Vec::new();

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
                    }
                    queue.push_back(id.clone());
                }
            }
        }

        // Transitive dependents (depth 2+)
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

        ImpactResult {
            seed: seed.to_string(),
            direct_features,
            direct_tests,
            transitive_features,
            transitive_tests,
        }
    }

    // -----------------------------------------------------------------------
    // Graph validation (ADR-009, ADR-013)
    // -----------------------------------------------------------------------

    pub fn check(&self) -> CheckResult {
        let mut result = CheckResult::new();

        // Include parse errors collected during loading (ADR-013 Tier 1)
        for pe in &self.parse_errors {
            match pe {
                ProductError::ParseError { file, line, message } => {
                    let mut diag = Diagnostic::error("E001", "malformed front-matter")
                        .with_file(file.clone())
                        .with_detail(message);
                    if let Some(l) = line {
                        diag = diag.with_line(*l);
                    }
                    result.errors.push(diag);
                }
                ProductError::InvalidId { file, id } => {
                    result.errors.push(
                        Diagnostic::error("E005", "invalid artifact ID")
                            .with_file(file.clone())
                            .with_detail(&format!("'{}' does not match PREFIX-NNN format", id)),
                    );
                }
                ProductError::MissingField { file, field } => {
                    result.errors.push(
                        Diagnostic::error("E006", "missing required field")
                            .with_file(file.clone())
                            .with_detail(&format!("required field '{}' not found", field)),
                    );
                }
                other => {
                    result.errors.push(
                        Diagnostic::error("E001", "parse error")
                            .with_detail(&format!("{}", other)),
                    );
                }
            }
        }
        let all_ids = self.all_ids();

        // E011: Duplicate IDs
        for (id, paths) in &self.duplicates {
            let path_strs: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
            result.errors.push(
                Diagnostic::error("E011", "duplicate artifact ID")
                    .with_detail(&format!("{} is declared in multiple files: {}", id, path_strs.join(", ")))
                    .with_hint("each artifact ID must be unique — rename or remove the duplicate"),
            );
        }

        // E002: Broken links (with line numbers and context per ADR-013)
        for f in self.features.values() {
            for adr_id in &f.front.adrs {
                if !all_ids.contains(adr_id) {
                    let mut diag = Diagnostic::error("E002", "broken link")
                        .with_file(f.path.clone())
                        .with_detail(&format!("{} references {} which does not exist", f.front.id, adr_id))
                        .with_hint("create the file with `product adr new` or remove the reference");
                    if let Some((line, content)) = find_reference_line(&f.path, adr_id) {
                        diag = diag.with_line(line).with_context(&content);
                    }
                    result.errors.push(diag);
                }
            }
            for test_id in &f.front.tests {
                if !all_ids.contains(test_id) {
                    let mut diag = Diagnostic::error("E002", "broken link")
                        .with_file(f.path.clone())
                        .with_detail(&format!("{} references {} which does not exist", f.front.id, test_id))
                        .with_hint("create the file with `product test new` or remove the reference");
                    if let Some((line, content)) = find_reference_line(&f.path, test_id) {
                        diag = diag.with_line(line).with_context(&content);
                    }
                    result.errors.push(diag);
                }
            }
            for dep_id in &f.front.depends_on {
                if !self.features.contains_key(dep_id) {
                    let mut diag = Diagnostic::error("E002", "broken link")
                        .with_file(f.path.clone())
                        .with_detail(&format!("{} depends-on {} which does not exist", f.front.id, dep_id))
                        .with_hint("create the feature or remove the dependency");
                    if let Some((line, content)) = find_reference_line(&f.path, dep_id) {
                        diag = diag.with_line(line).with_context(&content);
                    }
                    result.errors.push(diag);
                }
            }
        }

        for a in self.adrs.values() {
            for sup_id in &a.front.supersedes {
                if !all_ids.contains(sup_id) {
                    let mut diag = Diagnostic::error("E002", "broken link")
                        .with_file(a.path.clone())
                        .with_detail(&format!("{} supersedes {} which does not exist", a.front.id, sup_id));
                    if let Some((line, content)) = find_reference_line(&a.path, sup_id) {
                        diag = diag.with_line(line).with_context(&content);
                    }
                    result.errors.push(diag);
                }
            }
        }

        for t in self.tests.values() {
            for f_id in &t.front.validates.features {
                if !all_ids.contains(f_id) {
                    let mut diag = Diagnostic::error("E002", "broken link")
                        .with_file(t.path.clone())
                        .with_detail(&format!("{} validates feature {} which does not exist", t.front.id, f_id));
                    if let Some((line, content)) = find_reference_line(&t.path, f_id) {
                        diag = diag.with_line(line).with_context(&content);
                    }
                    result.errors.push(diag);
                }
            }
            for a_id in &t.front.validates.adrs {
                if !all_ids.contains(a_id) {
                    let mut diag = Diagnostic::error("E002", "broken link")
                        .with_file(t.path.clone())
                        .with_detail(&format!("{} validates ADR {} which does not exist", t.front.id, a_id));
                    if let Some((line, content)) = find_reference_line(&t.path, a_id) {
                        diag = diag.with_line(line).with_context(&content);
                    }
                    result.errors.push(diag);
                }
            }
        }

        // E003: Dependency cycles
        if let Err(ProductError::DependencyCycle { cycle }) = self.topological_sort() {
            result.errors.push(
                Diagnostic::error("E003", "dependency cycle in depends-on DAG")
                    .with_detail(&format!("cycle: {}", cycle.join(" -> "))),
            );
        }

        // E004: Supersession cycles
        if let Some(cycle) = self.detect_supersession_cycle() {
            result.errors.push(
                Diagnostic::error("E004", "supersession cycle in ADR supersedes chain")
                    .with_detail(&format!("cycle: {}", cycle.join(" -> "))),
            );
        }

        // W001: Orphaned artifacts
        for a in self.adrs.values() {
            let has_incoming = self.features.values().any(|f| f.front.adrs.contains(&a.front.id));
            if !has_incoming {
                result.warnings.push(
                    Diagnostic::warning("W001", "orphaned artifact")
                        .with_file(a.path.clone())
                        .with_detail(&format!("{} has no feature linking to it", a.front.id))
                        .with_hint("link it to a feature with `product feature link`"),
                );
            }
        }

        for t in self.tests.values() {
            // ADR-010: Exclude abandoned features from incoming check
            let has_incoming = self.features.values().any(|f| {
                f.front.status != FeatureStatus::Abandoned && f.front.tests.contains(&t.front.id)
            });
            if !has_incoming && t.front.validates.features.is_empty() {
                result.warnings.push(
                    Diagnostic::warning("W001", "orphaned artifact")
                        .with_file(t.path.clone())
                        .with_detail(&format!("{} has no feature linking to it", t.front.id)),
                );
            }
        }

        // W002: Features with no linked tests
        for f in self.features.values() {
            if f.front.status != FeatureStatus::Abandoned && f.front.tests.is_empty() {
                result.warnings.push(
                    Diagnostic::warning("W002", "feature has no linked test criteria")
                        .with_file(f.path.clone())
                        .with_detail(&format!("{} — {}", f.front.id, f.front.title))
                        .with_hint("add test criteria with `product test new`"),
                );
            }
        }

        // W003: Features with no exit-criteria test
        for f in self.features.values() {
            if f.front.status == FeatureStatus::Abandoned {
                continue;
            }
            let has_exit = f.front.tests.iter().any(|t_id| {
                self.tests
                    .get(t_id)
                    .map(|t| t.front.test_type == TestType::ExitCriteria)
                    .unwrap_or(false)
            });
            if !has_exit && !f.front.tests.is_empty() {
                result.warnings.push(
                    Diagnostic::warning("W003", "missing exit criteria")
                        .with_file(f.path.clone())
                        .with_detail(&format!("{} has no test of type `exit-criteria`", f.front.id))
                        .with_hint("add one with `product test new --type exit-criteria`"),
                );
            }
        }

        // W016: Feature marked complete but has unimplemented or failing TCs
        for f in self.features.values() {
            if f.front.status != FeatureStatus::Complete {
                continue;
            }
            let blocking_tcs: Vec<&str> = f
                .front
                .tests
                .iter()
                .filter_map(|t_id| {
                    self.tests.get(t_id.as_str()).and_then(|t| {
                        if t.front.status == TestStatus::Unimplemented
                            || t.front.status == TestStatus::Failing
                        {
                            Some(t.front.id.as_str())
                        } else {
                            None
                        }
                    })
                })
                .collect();
            if !blocking_tcs.is_empty() {
                let preview: Vec<&str> = blocking_tcs.iter().take(5).copied().collect();
                let suffix = if blocking_tcs.len() > 5 {
                    format!(", ... ({} total)", blocking_tcs.len())
                } else {
                    String::new()
                };
                result.warnings.push(
                    Diagnostic::warning("W016", "complete feature has unimplemented tests")
                        .with_file(f.path.clone())
                        .with_detail(&format!(
                            "{} is complete but has {} unimplemented/failing TC(s): {}{}",
                            f.front.id,
                            blocking_tcs.len(),
                            preview.join(", "),
                            suffix,
                        ))
                        .with_hint("run `product verify` to re-evaluate, or set blocking TCs to `unrunnable`"),
                );
            }
        }

        // W004: Invariant/chaos tests missing formal blocks
        for t in self.tests.values() {
            if (t.front.test_type == TestType::Invariant || t.front.test_type == TestType::Chaos)
                && t.formal_blocks.is_empty()
            {
                result.warnings.push(
                    Diagnostic::warning("W004", "missing formal specification blocks")
                        .with_file(t.path.clone())
                        .with_detail(&format!(
                            "{} is type {} but has no formal blocks",
                            t.front.id, t.front.test_type
                        )),
                );
            }
        }

        // W005: Phase label disagrees with dependency order
        for f in self.features.values() {
            for dep_id in &f.front.depends_on {
                if let Some(dep) = self.features.get(dep_id) {
                    if dep.front.phase > f.front.phase {
                        result.warnings.push(
                            Diagnostic::warning("W005", "phase label disagrees with dependency order")
                                .with_file(f.path.clone())
                                .with_detail(&format!(
                                    "{} (phase {}) depends-on {} (phase {})",
                                    f.front.id, f.front.phase, dep_id, dep.front.phase
                                )),
                        );
                    }
                }
            }
        }

        // W006: Evidence block delta below 0.7
        for t in self.tests.values() {
            for block in &t.formal_blocks {
                if let crate::formal::FormalBlock::Evidence(e) = block {
                    if e.delta < 0.7 {
                        result.warnings.push(
                            Diagnostic::warning("W006", "low-confidence specification")
                                .with_file(t.path.clone())
                                .with_detail(&format!(
                                    "{} evidence block δ={:.2} (below 0.7 threshold)",
                                    t.front.id, e.delta
                                )),
                        );
                    }
                }
            }
        }

        // Formal block diagnostics: E001 errors and W004 warnings from formal block parsing
        for t in self.tests.values() {
            let diag = crate::formal::parse_formal_blocks_with_diagnostics(&t.body);
            for err in &diag.errors {
                result.errors.push(
                    Diagnostic::error("E001", "formal block parse error")
                        .with_file(t.path.clone())
                        .with_detail(&format!("{}: {}", t.front.id, err)),
                );
            }
            for warn in &diag.warnings {
                result.warnings.push(
                    Diagnostic::warning("W004", "formal block warning")
                        .with_file(t.path.clone())
                        .with_detail(&format!("{}: {}", t.front.id, warn)),
                );
            }
        }

        // E014/W016: Content hash checks for accepted ADRs (ADR-032)
        // E015: Content hash checks for sealed TCs (ADR-032)
        let adrs_vec: Vec<&crate::types::Adr> = self.adrs.values().collect();
        let tests_vec: Vec<&crate::types::TestCriterion> = self.tests.values().collect();
        let hash_result = crate::hash::verify_all(&adrs_vec, &tests_vec);
        result.errors.extend(hash_result.errors);
        result.warnings.extend(hash_result.warnings);

        result
    }

    fn detect_supersession_cycle(&self) -> Option<Vec<String>> {
        for adr in self.adrs.values() {
            let mut visited = HashSet::new();
            let mut current = adr.front.id.clone();
            visited.insert(current.clone());
            while let Some(a) = self.adrs.get(&current) {
                if let Some(next) = a.front.supersedes.first() {
                    if visited.contains(next) {
                        return Some(visited.into_iter().collect());
                    }
                    visited.insert(next.clone());
                    current = next.clone();
                } else {
                    break;
                }
            }
        }
        None
    }

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

        let total_nodes = self.features.len() + self.adrs.len() + self.tests.len();
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

        if !self.direct_features.is_empty() || !self.direct_tests.is_empty() {
            println!("Direct dependents:");
            if !self.direct_features.is_empty() {
                let details: Vec<String> = self
                    .direct_features
                    .iter()
                    .map(|id| {
                        let status = graph
                            .features
                            .get(id)
                            .map(|f| format!("{}", f.front.status))
                            .unwrap_or_default();
                        format!("{} ({})", id, status)
                    })
                    .collect();
                println!("  Features:  {}", details.join(", "));
            }
            if !self.direct_tests.is_empty() {
                let details: Vec<String> = self
                    .direct_tests
                    .iter()
                    .map(|id| {
                        let status = graph
                            .tests
                            .get(id)
                            .map(|t| format!("{}", t.front.status))
                            .unwrap_or_default();
                        format!("{} ({})", id, status)
                    })
                    .collect();
                println!("  Tests:     {}", details.join(", "));
            }
        }

        if !self.transitive_features.is_empty() || !self.transitive_tests.is_empty() {
            println!();
            println!("Transitive dependents:");
            if !self.transitive_features.is_empty() {
                println!("  Features:  {}", self.transitive_features.join(", "));
            }
            if !self.transitive_tests.is_empty() {
                println!("  Tests:     {}", self.transitive_tests.join(", "));
            }
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::path::PathBuf;

    fn make_feature(id: &str, deps: Vec<&str>, adrs: Vec<&str>, tests: Vec<&str>, status: FeatureStatus) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.to_string(),
                title: format!("Feature {}", id),
                phase: 1,
                status,
                depends_on: deps.into_iter().map(String::from).collect(),
                adrs: adrs.into_iter().map(String::from).collect(),
                tests: tests.into_iter().map(String::from).collect(),
                domains: vec![],
                domains_acknowledged: std::collections::HashMap::new(),
                bundle: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    fn make_adr(id: &str) -> Adr {
        let title = format!("ADR {}", id);
        let body = String::new();
        let hash = crate::hash::compute_adr_hash(&title, &body);
        Adr {
            front: AdrFrontMatter {
                id: id.to_string(),
                title,
                status: AdrStatus::Accepted,
                features: vec![],
                supersedes: vec![],
                superseded_by: vec![],
                domains: vec![],
                scope: AdrScope::FeatureSpecific,
                content_hash: Some(hash),
                amendments: vec![],
                source_files: vec![],
            },
            body,
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    fn make_test(id: &str, adrs: Vec<&str>) -> TestCriterion {
        TestCriterion {
            front: TestFrontMatter {
                id: id.to_string(),
                title: format!("Test {}", id),
                test_type: TestType::Scenario,
                status: TestStatus::Unimplemented,
                validates: ValidatesBlock {
                    features: vec![],
                    adrs: adrs.into_iter().map(String::from).collect(),
                },
                phase: 1,
                content_hash: None,
                runner: None,
                runner_args: None,
                runner_timeout: None,
                requires: vec![],
                last_run: None,
                failure_message: None,
                last_run_duration: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("{}.md", id)),
            formal_blocks: vec![],
        }
    }

    #[test]
    fn topo_sort_simple() {
        let features = vec![
            make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Planned),
            make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
            make_feature("FT-003", vec!["FT-002"], vec![], vec![], FeatureStatus::Planned),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let order = graph.topological_sort().unwrap();
        assert_eq!(order, vec!["FT-001", "FT-002", "FT-003"]);
    }

    #[test]
    fn topo_sort_cycle_detected() {
        let features = vec![
            make_feature("FT-001", vec!["FT-002"], vec![], vec![], FeatureStatus::Planned),
            make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        assert!(graph.topological_sort().is_err());
    }

    #[test]
    fn feature_next_uses_topo() {
        let features = vec![
            make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Complete),
            make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::InProgress),
            make_feature("FT-003", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let next = graph.feature_next().unwrap();
        // FT-002 is in-progress with deps complete, FT-003 is planned with deps complete
        // Both are valid; topo sort order is deterministic — FT-002 comes first alphabetically
        assert_eq!(next, Some("FT-002".to_string()));
    }

    #[test]
    fn bfs_depth_1() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Planned),
        ];
        let adrs = vec![make_adr("ADR-001")];
        let tests = vec![make_test("TC-001", vec!["ADR-001"])];
        let graph = KnowledgeGraph::build(features, adrs, tests);
        let reachable = graph.bfs("FT-001", 1);
        assert!(reachable.contains(&"FT-001".to_string()));
        assert!(reachable.contains(&"ADR-001".to_string()));
        assert!(reachable.contains(&"TC-001".to_string()));
    }

    #[test]
    fn impact_analysis() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001"], vec![], FeatureStatus::InProgress),
            make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
        ];
        let adrs = vec![make_adr("ADR-001")];
        let tests = vec![make_test("TC-001", vec!["ADR-001"])];
        let graph = KnowledgeGraph::build(features, adrs, tests);
        let impact = graph.impact("ADR-001");
        assert!(impact.direct_features.contains(&"FT-001".to_string()));
    }

    #[test]
    fn graph_check_broken_link() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-999"], vec![], FeatureStatus::Planned),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let result = graph.check();
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].code == "E002");
    }

    #[test]
    fn graph_check_clean_exits_0() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Planned),
        ];
        let adrs = vec![make_adr("ADR-001")];
        let mut tc = make_test("TC-001", vec!["ADR-001"]);
        tc.front.test_type = TestType::ExitCriteria;
        tc.front.validates.features = vec!["FT-001".to_string()];
        let graph = KnowledgeGraph::build(features, adrs, vec![tc]);
        let result = graph.check();
        assert_eq!(result.exit_code(), 0, "clean graph should exit 0: errors={:?} warnings={:?}", result.errors, result.warnings);
    }

    #[test]
    fn graph_check_warning_exits_2() {
        // Feature with no tests -> W002
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001"], vec![], FeatureStatus::Planned),
        ];
        let adrs = vec![make_adr("ADR-001")];
        let graph = KnowledgeGraph::build(features, adrs, vec![]);
        let result = graph.check();
        assert!(result.errors.is_empty(), "should have no errors");
        assert!(!result.warnings.is_empty(), "should have warnings");
        assert_eq!(result.exit_code(), 2);
    }

    #[test]
    fn graph_check_e003_cycle() {
        let features = vec![
            make_feature("FT-001", vec!["FT-002"], vec![], vec![], FeatureStatus::Planned),
            make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let result = graph.check();
        assert!(result.errors.iter().any(|e| e.code == "E003"), "should detect cycle E003");
        assert_eq!(result.exit_code(), 1);
    }

    #[test]
    fn graph_check_w001_orphaned_adr() {
        let features = vec![
            make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Planned),
        ];
        let adrs = vec![make_adr("ADR-001")]; // not linked to any feature
        let graph = KnowledgeGraph::build(features, adrs, vec![]);
        let result = graph.check();
        assert!(result.warnings.iter().any(|w| w.code == "W001"), "should report orphan W001");
    }

    #[test]
    fn graph_check_w002_no_tests() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001"], vec![], FeatureStatus::Planned),
        ];
        let adrs = vec![make_adr("ADR-001")];
        let graph = KnowledgeGraph::build(features, adrs, vec![]);
        let result = graph.check();
        assert!(result.warnings.iter().any(|w| w.code == "W002"), "should report no-tests W002");
    }

    #[test]
    fn graph_check_w003_no_exit_criteria() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Planned),
        ];
        let adrs = vec![make_adr("ADR-001")];
        let tests = vec![make_test("TC-001", vec!["ADR-001"])]; // type=Scenario, not ExitCriteria
        let graph = KnowledgeGraph::build(features, adrs, tests);
        let result = graph.check();
        assert!(result.warnings.iter().any(|w| w.code == "W003"), "should report W003");
    }

    #[test]
    fn graph_check_w016_complete_with_unimplemented() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Complete),
        ];
        let adrs = vec![make_adr("ADR-001")];
        let mut tc = make_test("TC-001", vec!["ADR-001"]);
        tc.front.validates.features = vec!["FT-001".to_string()];
        // TC defaults to Unimplemented status
        let graph = KnowledgeGraph::build(features, adrs, vec![tc]);
        let result = graph.check();
        assert!(result.warnings.iter().any(|w| w.code == "W016"), "should report W016 for complete feature with unimplemented TC");
    }

    #[test]
    fn graph_check_w016_not_fired_when_passing() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001"], vec!["TC-001"], FeatureStatus::Complete),
        ];
        let adrs = vec![make_adr("ADR-001")];
        let mut tc = make_test("TC-001", vec!["ADR-001"]);
        tc.front.validates.features = vec!["FT-001".to_string()];
        tc.front.test_type = TestType::ExitCriteria;
        tc.front.status = TestStatus::Passing;
        let graph = KnowledgeGraph::build(features, adrs, vec![tc]);
        let result = graph.check();
        assert!(!result.warnings.iter().any(|w| w.code == "W016"), "should not fire W016 when all TCs passing");
    }

    #[test]
    fn graph_check_w005_phase_dep_mismatch() {
        let mut f1 = make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Planned);
        f1.front.phase = 2; // dependency in phase 2
        let mut f2 = make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned);
        f2.front.phase = 1; // feature in phase 1 depends on phase 2 feature
        let graph = KnowledgeGraph::build(vec![f1, f2], vec![], vec![]);
        let result = graph.check();
        assert!(result.warnings.iter().any(|w| w.code == "W005"), "should report W005 phase mismatch");
    }

    #[test]
    fn centrality_returns_values() {
        let features = vec![
            make_feature("FT-001", vec![], vec!["ADR-001", "ADR-002"], vec![], FeatureStatus::Planned),
            make_feature("FT-002", vec![], vec!["ADR-001"], vec![], FeatureStatus::Planned),
        ];
        let adrs = vec![make_adr("ADR-001"), make_adr("ADR-002")];
        let graph = KnowledgeGraph::build(features, adrs, vec![]);
        let centrality = graph.betweenness_centrality();
        // ADR-001 is linked to both features, should have higher centrality
        let c1 = centrality.get("ADR-001").copied().unwrap_or(0.0);
        let c2 = centrality.get("ADR-002").copied().unwrap_or(0.0);
        assert!(c1 >= 0.0 && c1 <= 1.0, "centrality should be in [0,1]");
        assert!(c2 >= 0.0 && c2 <= 1.0, "centrality should be in [0,1]");
    }

    #[test]
    fn topo_sort_parallel() {
        // FT-002 and FT-003 both depend on FT-001, no dependency between them
        let features = vec![
            make_feature("FT-001", vec![], vec![], vec![], FeatureStatus::Planned),
            make_feature("FT-002", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
            make_feature("FT-003", vec!["FT-001"], vec![], vec![], FeatureStatus::Planned),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let order = graph.topological_sort().unwrap();
        let pos1 = order.iter().position(|id| id == "FT-001").unwrap();
        let pos2 = order.iter().position(|id| id == "FT-002").unwrap();
        let pos3 = order.iter().position(|id| id == "FT-003").unwrap();
        assert!(pos1 < pos2, "FT-001 must come before FT-002");
        assert!(pos1 < pos3, "FT-001 must come before FT-003");
    }
}
