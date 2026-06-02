//! Artifact deletion (FT-064).
//!
//! A `type: delete` request carries a `deletions:` list of artifact IDs.
//! Validation guarantees every target exists and has no inbound links
//! before the apply pipeline calls into this module. The actual unlink is a
//! best-effort per-file operation: an I/O failure on one file surfaces as an
//! E-class finding but does not block the other deletions in the same
//! request.

use super::super::types::*;
use super::types::DeletedArtifact;
use crate::graph::KnowledgeGraph;
use std::path::PathBuf;

/// Look up an artifact's on-disk path from the loaded graph. Returns `None`
/// when the ID does not exist in any of the four artifact directories.
pub fn lookup_artifact_path(graph: &KnowledgeGraph, id: &str) -> Option<PathBuf> {
    if let Some(f) = graph.features.get(id) {
        return Some(f.path.clone());
    }
    if let Some(a) = graph.adrs.get(id) {
        return Some(a.path.clone());
    }
    if let Some(t) = graph.tests.get(id) {
        return Some(t.path.clone());
    }
    if let Some(d) = graph.dependencies.get(id) {
        return Some(d.path.clone());
    }
    None
}

/// Resolve every `deletions[]` entry to an `(id, path)` pair using the
/// pre-loaded graph. Surfaces **E002** for any target the graph cannot
/// resolve (this should already have been caught by validation but the
/// defensive check prevents corruption on a logic bug).
pub fn resolve_deletion_targets(
    request: &Request,
    graph: &KnowledgeGraph,
) -> Result<Vec<(String, PathBuf)>, Vec<Finding>> {
    let mut targets: Vec<(String, PathBuf)> = Vec::new();
    let mut errors: Vec<Finding> = Vec::new();
    for d in &request.deletions {
        match lookup_artifact_path(graph, &d.target) {
            Some(p) => targets.push((d.target.clone(), p)),
            None => errors.push(Finding::error(
                "E002",
                format!(
                    "deletion target '{}' has no resolvable on-disk path",
                    d.target
                ),
                format!("$.deletions[{}].target", d.index),
            )),
        }
    }
    if errors.is_empty() {
        Ok(targets)
    } else {
        Err(errors)
    }
}

/// Unlink every resolved deletion target. Each failure is recorded as an
/// `E009` finding pushed into `findings`; the caller decides how to propagate.
/// Returns the list of artifacts whose files were successfully removed.
pub fn unlink_targets(
    targets: &[(String, PathBuf)],
    findings: &mut Vec<Finding>,
) -> Vec<DeletedArtifact> {
    let mut deleted: Vec<DeletedArtifact> = Vec::new();
    for (id, path) in targets {
        match std::fs::remove_file(path) {
            Ok(_) => deleted.push(DeletedArtifact {
                id: id.clone(),
                file: path.display().to_string(),
            }),
            Err(e) => {
                findings.push(Finding::error(
                    "E009",
                    format!("failed to delete {}: {}", path.display(), e),
                    format!("$.deletions[?(@.target=='{}')]", id),
                ));
            }
        }
    }
    deleted
}
