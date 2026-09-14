//! Reader recall, walk recall, walk precision — measured against a hand-enumerated ground truth (CG-R-71).
//!
//! A ground-truth file lists the composition edges of one project, read from
//! the source by a person: `(from type, target type)` at the reader's
//! granularity (targets are original definitions). Two figures follow:
//! **reader recall** — the fraction present as `parameter`, `resolve`,
//! `signature` or (for collection injection) `generic-argument` facts from a
//! member of the type; **walk recall** and **walk
//! precision** — the fraction the walk produced as composition edges, and the
//! fraction of the walk's edges from that project the enumeration contains.
//! Until reader recall is known, coverage is uninterpretable (CG-R-71), so
//! every report prints these beside it when a file is given.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::csharp_inventory::{Index, Inventory};
use super::csharp_walk::Closure;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct GtEdge {
    pub from: String,
    pub target: String,
}

/// The file: which project, and the edges a person found in its source.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroundTruth {
    pub project: String,
    #[serde(default)]
    pub source: String,
    pub edges: Vec<GtEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroundTruthReport {
    pub project: String,
    pub source: String,
    pub edges: usize,
    pub reader_present: usize,
    pub reader_recall_percent: f64,
    pub reader_missing: Vec<GtEdge>,
    pub walk_hits: usize,
    pub walk_recall_percent: f64,
    pub walk_missing: Vec<GtEdge>,
    /// Walk edges from the project's types the enumeration does not contain.
    pub walk_extra: Vec<GtEdge>,
    pub walk_edges: usize,
    pub walk_precision_percent: f64,
}

fn pct(num: usize, den: usize) -> f64 {
    if den == 0 { 100.0 } else { 100.0 * num as f64 / den as f64 }
}

/// Is the edge present as a reader fact from any member of `from`?
fn reader_has(inv: &Inventory, ix: &Index<'_>, e: &GtEdge) -> bool {
    let members = ix.members_of.get(e.from.as_str()).into_iter().flatten().copied().collect::<BTreeSet<&str>>();
    inv.references.iter().any(|r| {
        members.contains(r.from.as_str()) && r.to == e.target && matches!(r.kind.as_str(), "parameter" | "resolve" | "signature" | "generic-argument")
    })
}

pub fn measure(inv: &Inventory, ix: &Index<'_>, c: &Closure<'_>, gt: &GroundTruth) -> GroundTruthReport {
    let project_id = inv.projects.iter().find(|p| p.name == gt.project || p.id == gt.project).map(|p| p.id.clone()).unwrap_or_else(|| gt.project.clone());
    let truth: BTreeSet<GtEdge> = gt.edges.iter().cloned().collect();
    let reader_missing: Vec<GtEdge> = truth.iter().filter(|e| !reader_has(inv, ix, e)).cloned().collect();
    let walk: BTreeSet<GtEdge> = c
        .edges
        .iter()
        .filter(|e| ix.types.get(e.from.as_str()).is_some_and(|t| t.project == project_id))
        .map(|e| GtEdge { from: e.from.clone(), target: e.target.clone() })
        .collect();
    let walk_missing: Vec<GtEdge> = truth.difference(&walk).cloned().collect();
    let walk_extra: Vec<GtEdge> = walk.difference(&truth).cloned().collect();
    let walk_hits = truth.len() - walk_missing.len();
    GroundTruthReport {
        project: gt.project.clone(),
        source: gt.source.clone(),
        edges: truth.len(),
        reader_present: truth.len() - reader_missing.len(),
        reader_recall_percent: pct(truth.len() - reader_missing.len(), truth.len()),
        reader_missing,
        walk_hits,
        walk_recall_percent: pct(walk_hits, truth.len()),
        walk_missing,
        walk_precision_percent: pct(walk_hits, walk.len()),
        walk_extra,
        walk_edges: walk.len(),
    }
}
