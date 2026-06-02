//! Feature link operations — bidirectional ADR/TC linking with reciprocation (FT-066).
//!
//! `plan_link` is the pure planning function the CLI adapter and the MCP
//! handler both call. It produces a `LinkPlan` that carries every write
//! needed to keep the graph bidirectional: the feature's own `adrs:` /
//! `tests:` arrays, plus the target ADR's `features:` array and/or the
//! target TC's `validates.features` array.
//!
//! `apply_link` writes the whole plan atomically via `write_batch_atomic`,
//! so a partial failure cannot leave one side of a reciprocal link
//! orphaned.

use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser};
use std::path::PathBuf;

/// A single file the plan will touch.
#[derive(Debug, Clone)]
pub struct LinkWrite {
    pub path: PathBuf,
    pub content: String,
    pub kind: LinkWriteKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkWriteKind {
    Feature,
    Adr,
    Tc,
    Pattern,
}

impl LinkWriteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feature => "feature",
            Self::Adr => "adr",
            Self::Tc => "tc",
            Self::Pattern => "pattern",
        }
    }
}

/// FT-073: a non-fatal warning produced by `plan_link` — currently only
/// fires when the caller cites a deprecated pattern. The write still
/// proceeds. Captured in the plan so adapters (CLI / MCP) can surface the
/// warning to the user without producing diverging output paths.
#[derive(Debug, Clone)]
pub struct LinkWarning {
    pub code: &'static str,
    pub message: String,
}

/// A reciprocal back-reference that the plan filled in on a link target.
#[derive(Debug, Clone)]
pub struct LinkReciprocation {
    pub id: String,
    pub field: &'static str,
}

/// Plan describing every write needed for a `product_feature_link` call.
#[derive(Debug, Clone)]
pub struct LinkPlan {
    pub feature_id: String,
    pub writes: Vec<LinkWrite>,
    pub reciprocated: Vec<LinkReciprocation>,
    /// FT-073: non-fatal warnings surfaced by the planner.
    #[allow(dead_code)]
    pub warnings: Vec<LinkWarning>,
}

impl LinkPlan {
    pub fn is_changed(&self) -> bool {
        !self.writes.is_empty()
    }
}

/// Pure: produce a `LinkPlan` from optional ADR / TC link arguments.
///
/// - Validates that the feature, ADR, and TC targets all exist (E002 /
///   `NotFound` before any write).
/// - Computes the feature-side update and the reciprocal back-reference
///   on each target in one pass.
/// - Idempotent: if a link is already present on both sides, no writes
///   are emitted for that target. If a link exists on the feature side
///   only (legacy data), the reciprocal write is still emitted.
pub fn plan_link(
    graph: &KnowledgeGraph,
    feature_id: &str,
    adr: Option<&str>,
    test: Option<&str>,
) -> Result<LinkPlan, ProductError> {
    plan_link_with_pattern(graph, feature_id, adr, test, None)
}

/// FT-073: extended `plan_link` that also accepts an optional `pattern:
/// PAT-YYY`. When supplied, the feature's `patterns:` array gets the entry
/// and the pattern's `examples:` array is reciprocated. Linking against a
/// deprecated pattern succeeds but emits a `LinkWarning`.
pub fn plan_link_with_pattern(
    graph: &KnowledgeGraph,
    feature_id: &str,
    adr: Option<&str>,
    test: Option<&str>,
    pattern: Option<&str>,
) -> Result<LinkPlan, ProductError> {
    let feature = graph
        .features
        .get(feature_id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", feature_id)))?;

    // Validate targets exist before computing any writes — we never want
    // to half-apply a plan because one of the targets was a typo.
    if let Some(adr_id) = adr {
        if !graph.adrs.contains_key(adr_id) {
            return Err(ProductError::NotFound(format!("ADR {}", adr_id)));
        }
    }
    if let Some(tc_id) = test {
        if !graph.tests.contains_key(tc_id) {
            return Err(ProductError::NotFound(format!("test {}", tc_id)));
        }
    }
    if let Some(pat_id) = pattern {
        if !graph.patterns.contains_key(pat_id) {
            return Err(ProductError::NotFound(format!("pattern {}", pat_id)));
        }
    }

    let mut front = feature.front.clone();
    let mut writes = Vec::new();
    let mut reciprocated = Vec::new();
    let mut warnings: Vec<LinkWarning> = Vec::new();
    let mut feature_changed = false;

    if let Some(adr_id) = adr {
        if !front.adrs.contains(&adr_id.to_string()) {
            front.adrs.push(adr_id.to_string());
            feature_changed = true;
        }
        if let Some(adr) = graph.adrs.get(adr_id) {
            if !adr.front.features.contains(&feature_id.to_string()) {
                let mut adr_front = adr.front.clone();
                adr_front.features.push(feature_id.to_string());
                let content = parser::render_adr(&adr_front, &adr.body);
                writes.push(LinkWrite {
                    path: adr.path.clone(),
                    content,
                    kind: LinkWriteKind::Adr,
                });
                reciprocated.push(LinkReciprocation {
                    id: adr_id.to_string(),
                    field: "features",
                });
            }
        }
    }

    if let Some(tc_id) = test {
        if !front.tests.contains(&tc_id.to_string()) {
            front.tests.push(tc_id.to_string());
            feature_changed = true;
        }
        if let Some(tc) = graph.tests.get(tc_id) {
            if !tc.front.validates.features.contains(&feature_id.to_string()) {
                let mut tc_front = tc.front.clone();
                tc_front.validates.features.push(feature_id.to_string());
                let content = parser::render_test(&tc_front, &tc.body);
                writes.push(LinkWrite {
                    path: tc.path.clone(),
                    content,
                    kind: LinkWriteKind::Tc,
                });
                reciprocated.push(LinkReciprocation {
                    id: tc_id.to_string(),
                    field: "validates.features",
                });
            }
        }
    }

    if let Some(pat_id) = pattern {
        if !front.patterns.contains(&pat_id.to_string()) {
            front.patterns.push(pat_id.to_string());
            feature_changed = true;
        }
        if let Some(pat) = graph.patterns.get(pat_id) {
            // Deprecation surfaced as a non-fatal warning; write still
            // proceeds because the author may intentionally cite the
            // deprecated pattern while migrating.
            if pat.front.status == crate::types::PatternStatus::Deprecated {
                let replacement = pat
                    .front
                    .deprecated_by
                    .as_deref()
                    .map(|r| format!(" (replaced by {})", r))
                    .unwrap_or_default();
                warnings.push(LinkWarning {
                    code: "W032",
                    message: format!(
                        "{} cites deprecated pattern {}{}",
                        feature_id, pat_id, replacement
                    ),
                });
            }
            if !pat.front.examples.contains(&feature_id.to_string()) {
                let mut pat_front = pat.front.clone();
                pat_front.examples.push(feature_id.to_string());
                let content = parser::render_pattern(&pat_front, &pat.body);
                writes.push(LinkWrite {
                    path: pat.path.clone(),
                    content,
                    kind: LinkWriteKind::Pattern,
                });
                reciprocated.push(LinkReciprocation {
                    id: pat_id.to_string(),
                    field: "examples",
                });
            }
        }
    }

    // The feature's own write must be inserted first so callers reporting
    // the write list lead with the primary artifact.
    if feature_changed {
        let content = parser::render_feature(&front, &feature.body);
        let feature_write = LinkWrite {
            path: feature.path.clone(),
            content,
            kind: LinkWriteKind::Feature,
        };
        writes.insert(0, feature_write);
    }

    Ok(LinkPlan {
        feature_id: feature_id.to_string(),
        writes,
        reciprocated,
        warnings,
    })
}

/// I/O: write every file in the plan as a single atomic batch.
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

