//! Knowledge graph data model — nodes, edges, construction from parsed artifacts.

use crate::error::ProductError;
use crate::types::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

/// Find the 1-based line number and trimmed content of the line where `needle`
/// first appears in a file. Returns `None` if the file cannot be read or needle
/// is not found.
pub(crate) fn find_reference_line(path: &Path, needle: &str) -> Option<(usize, String)> {
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
    Supersedes,      // ADR -> ADR (or DEP -> DEP)
    DependsOn,       // Feature -> Feature
    Uses,            // Feature -> Dependency (ADR-030)
    Governs,         // ADR -> Dependency (ADR-030)
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
    pub dependencies: HashMap<String, crate::types::Dependency>,
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
        Self::build_with_deps(features, adrs, tests, Vec::new())
    }

    /// Build graph from loaded artifacts including dependencies (ADR-030)
    pub fn build_with_deps(
        features: Vec<Feature>,
        adrs: Vec<Adr>,
        tests: Vec<TestCriterion>,
        deps: Vec<crate::types::Dependency>,
    ) -> Self {
        let mut graph = Self {
            features: HashMap::new(),
            adrs: HashMap::new(),
            tests: HashMap::new(),
            dependencies: HashMap::new(),
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
        for d in deps {
            id_paths.entry(d.front.id.clone()).or_default().push(d.path.clone());
            graph.dependencies.insert(d.front.id.clone(), d);
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

        // Dependency edges (ADR-030)
        // features field on DEP → Uses edge (Feature -> Dependency)
        for d in graph.dependencies.values() {
            for feat_id in &d.front.features {
                pending_edges.push((feat_id.clone(), d.front.id.clone(), EdgeType::Uses));
            }
            // adrs field on DEP → Governs edge (ADR -> Dependency)
            for adr_id in &d.front.adrs {
                pending_edges.push((adr_id.clone(), d.front.id.clone(), EdgeType::Governs));
            }
            // supersedes field on DEP → Supersedes edge (DEP -> DEP)
            for sup_id in &d.front.supersedes {
                pending_edges.push((d.front.id.clone(), sup_id.clone(), EdgeType::Supersedes));
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

    pub(crate) fn add_edge(&mut self, from: &str, to: &str, edge_type: EdgeType) {
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
        ids.extend(self.dependencies.keys().cloned());
        ids
    }
}
