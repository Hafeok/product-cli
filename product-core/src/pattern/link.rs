//! Pattern link operations — ADR / requires / example (FT-070).
//!
//! `--example FT-N` reciprocates by writing `FT-N.patterns` in the same
//! atomic batch (ADR-050 bidirectional materialisation).
//! `--requires PAT-Y` runs the existing depends-on cycle check generalised
//! to pattern requires.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser, types};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkWriteKind {
    Pattern,
    Feature,
}

impl LinkWriteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pattern => "pattern",
            Self::Feature => "feature",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinkWrite {
    pub path: PathBuf,
    pub content: String,
    pub kind: LinkWriteKind,
}

#[derive(Debug, Clone)]
pub struct LinkReciprocation {
    pub id: String,
    pub field: &'static str,
}

#[derive(Debug, Clone)]
pub struct LinkPlan {
    pub pattern_id: String,
    pub writes: Vec<LinkWrite>,
    pub reciprocated: Vec<LinkReciprocation>,
}

impl LinkPlan {
    pub fn is_changed(&self) -> bool {
        !self.writes.is_empty()
    }
}

/// Pure: produce a `LinkPlan` from optional adr / requires / example args.
///
/// - Validates all link targets exist (E002 / `NotFound` before any write).
/// - Cycle-checks `requires:` (E003).
/// - Reciprocates `examples:` ↔ `feature.patterns`.
pub fn plan_link(
    graph: &KnowledgeGraph,
    patterns: &HashMap<String, types::Pattern>,
    pattern_id: &str,
    adr: Option<&str>,
    requires: Option<&str>,
    example: Option<&str>,
) -> Result<LinkPlan, ProductError> {
    let pattern = patterns
        .get(pattern_id)
        .ok_or_else(|| ProductError::NotFound(format!("pattern {}", pattern_id)))?;

    if let Some(adr_id) = adr {
        if !graph.adrs.contains_key(adr_id) {
            return Err(ProductError::NotFound(format!("ADR {}", adr_id)));
        }
    }
    if let Some(target) = requires {
        if !patterns.contains_key(target) {
            return Err(ProductError::NotFound(format!("pattern {}", target)));
        }
        // E003: requires cycle.
        if would_create_cycle(patterns, pattern_id, target) {
            return Err(ProductError::DependencyCycle {
                cycle: vec![
                    pattern_id.to_string(),
                    target.to_string(),
                    pattern_id.to_string(),
                ],
            });
        }
    }
    if let Some(feature_id) = example {
        if !graph.features.contains_key(feature_id) {
            return Err(ProductError::NotFound(format!("feature {}", feature_id)));
        }
    }

    let mut front = pattern.front.clone();
    let mut writes: Vec<LinkWrite> = Vec::new();
    let mut reciprocated: Vec<LinkReciprocation> = Vec::new();
    let mut pattern_changed = false;

    if let Some(adr_id) = adr {
        if !front.adrs.contains(&adr_id.to_string()) {
            front.adrs.push(adr_id.to_string());
            pattern_changed = true;
        }
    }

    if let Some(target) = requires {
        if !front.requires.contains(&target.to_string()) {
            front.requires.push(target.to_string());
            pattern_changed = true;
        }
    }

    if let Some(feature_id) = example {
        if !front.examples.contains(&feature_id.to_string()) {
            front.examples.push(feature_id.to_string());
            pattern_changed = true;
        }
        if let Some(feature) = graph.features.get(feature_id) {
            if !feature.front.patterns.contains(&pattern_id.to_string()) {
                let mut feat_front = feature.front.clone();
                feat_front.patterns.push(pattern_id.to_string());
                let content = parser::render_feature(&feat_front, &feature.body);
                writes.push(LinkWrite {
                    path: feature.path.clone(),
                    content,
                    kind: LinkWriteKind::Feature,
                });
                reciprocated.push(LinkReciprocation {
                    id: feature_id.to_string(),
                    field: "patterns",
                });
            }
        }
    }

    if pattern_changed {
        let content = parser::render_pattern(&front, &pattern.body);
        writes.insert(
            0,
            LinkWrite {
                path: pattern.path.clone(),
                content,
                kind: LinkWriteKind::Pattern,
            },
        );
    }

    Ok(LinkPlan {
        pattern_id: pattern_id.to_string(),
        writes,
        reciprocated,
    })
}

/// I/O: write every file in the plan as one atomic batch.
pub fn apply_link(plan: &LinkPlan) -> Result<(), ProductError> {
    if plan.writes.is_empty() {
        return Ok(());
    }
    let refs: Vec<(&std::path::Path, &str)> = plan
        .writes
        .iter()
        .map(|w| (w.path.as_path(), w.content.as_str()))
        .collect();
    fileops::write_batch_atomic(&refs)?;
    Ok(())
}

/// BFS over the existing `requires` graph to see if `target → source` is
/// reachable. If so, adding `source → target` would close a cycle.
fn would_create_cycle(
    patterns: &HashMap<String, types::Pattern>,
    source: &str,
    target: &str,
) -> bool {
    if source == target {
        return true;
    }
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(target.to_string());
    while let Some(node) = queue.pop_front() {
        if !visited.insert(node.clone()) {
            continue;
        }
        if node == source {
            return true;
        }
        if let Some(pat) = patterns.get(&node) {
            for next in &pat.front.requires {
                if !visited.contains(next) {
                    queue.push_back(next.clone());
                }
            }
        }
    }
    false
}
