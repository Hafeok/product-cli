//! Apply pipeline orchestration (FT-041, ADR-038, FT-064).
//!
//! Composes the 13-step pipeline (pre-checksum, validate, lock, sort, resolve
//! refs, plan writes, batch write, batch rename, deletion unlink, graph
//! check, log append, summary). Result and option types live in `types.rs`;
//! deletion bookkeeping lives in `delete.rs`; started-tag emission lives in
//! `started_tags.rs`.

pub mod assign;
pub mod checksum;
pub mod delete;
pub mod mutate;
pub mod plan;
pub mod render;
pub mod started_tags;
pub mod types;

use super::types::*;
use super::validate::{self, ValidationContext};
use crate::config::ProductConfig;
use crate::fileops;
use crate::graph::KnowledgeGraph;
use crate::parser;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub use self::types::{
    ApplyOptions, ApplyResult, ChangedArtifact, CreatedArtifact, DeletedArtifact,
};

fn empty_failure(findings: Vec<Finding>) -> ApplyResult {
    ApplyResult {
        applied: false,
        created: Vec::new(),
        changed: Vec::new(),
        deleted: Vec::new(),
        findings,
        graph_check_clean: false,
        started_tags: Vec::new(),
        started_tag_warnings: Vec::new(),
    }
}

pub fn apply_request(
    request: &Request,
    config: &ProductConfig,
    repo_root: &Path,
    options: ApplyOptions,
) -> ApplyResult {
    let features_dir = config.resolve_path(repo_root, &config.paths.features);
    let adrs_dir = config.resolve_path(repo_root, &config.paths.adrs);
    let tests_dir = config.resolve_path(repo_root, &config.paths.tests);
    let deps_dir = config.resolve_path(repo_root, &config.paths.dependencies);

    let loaded = match parser::load_all_with_deps(
        &features_dir, &adrs_dir, &tests_dir, Some(&deps_dir),
    ) {
        Ok(l) => l,
        Err(e) => {
            return empty_failure(vec![Finding::error(
                "E001",
                format!("failed to load graph: {}", e),
                "$",
            )]);
        }
    };
    let graph = KnowledgeGraph::build_with_deps(
        loaded.features, loaded.adrs, loaded.tests, loaded.dependencies,
    );

    // FT-053 / ADR-045 — snapshot pre-apply feature statuses so we can detect
    // `planned → in-progress` transitions and emit started tags.
    let pre_feature_statuses: HashMap<String, crate::types::FeatureStatus> = graph
        .features
        .iter()
        .map(|(id, f)| (id.clone(), f.front.status))
        .collect();

    let ctx = ValidationContext { config, graph: &graph };
    let mut findings = validate::validate_request(request, &ctx);

    let mut refs: HashMap<String, (ArtifactType, usize)> = HashMap::new();
    for a in &request.artifacts {
        if let Some(ref n) = a.ref_name {
            refs.entry(n.clone()).or_insert((a.artifact_type, a.index));
        }
    }
    validate::check_dep_governance(request, &refs, &graph, &mut findings);

    // ADR-039 decision 8: git identity is required for apply (not dry-run).
    let applied_by = if options.dry_run || options.skip_git_identity {
        crate::request_log::git_identity::resolve_applied_by(repo_root)
            .unwrap_or_else(|_| "local:unknown".into())
    } else {
        match crate::request_log::git_identity::resolve_applied_by(repo_root) {
            Ok(s) => s,
            Err(msg) => {
                findings.push(Finding::error("E009", msg, "$"));
                return empty_failure(findings);
            }
        }
    };

    let has_errors = findings.iter().any(|f| f.is_error());
    if has_errors || options.dry_run {
        return ApplyResult {
            applied: false, created: Vec::new(), changed: Vec::new(),
            deleted: Vec::new(),
            findings, graph_check_clean: !has_errors,
            started_tags: Vec::new(), started_tag_warnings: Vec::new(),
        };
    }

    let ref_to_id = match assign::assign_ids(&request.artifacts, &graph, config) {
        Ok(m) => m,
        Err(f) => {
            findings.push(f);
            return empty_failure(findings);
        }
    };

    let (new_writes, mutation_results) = match plan::plan_writes(
        request, &ref_to_id, &graph, config, repo_root,
    ) {
        Ok(v) => v,
        Err(mut fs) => {
            findings.append(&mut fs);
            return empty_failure(findings);
        }
    };

    // FT-064 — resolve every deletion's on-disk path from the live graph.
    let deletion_targets = match delete::resolve_deletion_targets(request, &graph) {
        Ok(t) => t,
        Err(mut fs) => {
            findings.append(&mut fs);
            return empty_failure(findings);
        }
    };

    let touched_dirs = [&features_dir, &adrs_dir, &tests_dir, &deps_dir];
    let pre_hashes = checksum::checksum_all(&touched_dirs);

    let mut writes: Vec<(PathBuf, String)> = Vec::new();
    for nw in &new_writes {
        writes.push((nw.path.clone(), nw.content.clone()));
    }
    for mu in &mutation_results {
        writes.push((mu.path.clone(), mu.content.clone()));
    }
    let write_refs: Vec<(&Path, &str)> = writes
        .iter()
        .map(|(p, c)| (p.as_path(), c.as_str()))
        .collect();

    if let Err(e) = fileops::write_batch_atomic(&write_refs) {
        let post_hashes = checksum::checksum_all(&touched_dirs);
        let msg = if pre_hashes != post_hashes {
            format!("apply failed and zero-files-changed invariant violated: {}", e)
        } else {
            format!("apply failed (zero files changed): {}", e)
        };
        findings.push(Finding::error("E009", msg, "$"));
        return empty_failure(findings);
    }

    // FT-064 — unlink deletion targets. The pre-write batch has already
    // committed, but deletions only fire when validation passed (the target
    // exists with no inbound links). Per-file unlink failures surface as
    // E-class findings but do not block the rest.
    let deleted = delete::unlink_targets(&deletion_targets, &mut findings);

    let created: Vec<CreatedArtifact> = new_writes
        .iter()
        .map(|nw| CreatedArtifact {
            ref_name: nw.assigned_id.0.clone(),
            id: nw.assigned_id.1.clone(),
            file: nw.path.display().to_string(),
        })
        .collect();
    let changed: Vec<ChangedArtifact> = mutation_results
        .iter()
        .map(|m| ChangedArtifact {
            id: m.target_id.clone(),
            mutations: m.mutation_count,
            file: m.path.display().to_string(),
        })
        .collect();

    let (graph_check_clean, post_feature_statuses) = match parser::load_all_with_deps(
        &features_dir, &adrs_dir, &tests_dir, Some(&deps_dir),
    ) {
        Ok(l) => {
            let post_statuses: HashMap<String, crate::types::FeatureStatus> = l
                .features
                .iter()
                .map(|f| (f.front.id.clone(), f.front.status))
                .collect();
            let g = KnowledgeGraph::build_with_deps(
                l.features, l.adrs, l.tests, l.dependencies,
            );
            (g.check().errors.is_empty(), post_statuses)
        }
        Err(_) => (false, HashMap::new()),
    };

    // FT-053 / ADR-045 — best-effort started-tag emission.
    let (started_tags, started_tag_warnings) = started_tags::emit_started_tags(
        repo_root,
        &pre_feature_statuses,
        &post_feature_statuses,
    );

    // FT-041 compat: append to legacy `.product/request-log.jsonl` too.
    let _ = super::log::append_log(repo_root, request, &created, &changed);

    // FT-042: append hash-chained entry to `requests.jsonl` (committed log).
    append_log_entry(
        request,
        config,
        repo_root,
        &applied_by,
        &created,
        &changed,
        &deleted,
    );

    ApplyResult {
        applied: true,
        created,
        changed,
        deleted,
        findings,
        graph_check_clean,
        started_tags,
        started_tag_warnings,
    }
}

/// Append the canonical-JSON `requests.jsonl` entry for this apply (FT-042,
/// FT-064). Caller has already determined success — this is best-effort log
/// write and any failure is non-fatal.
fn append_log_entry(
    request: &Request,
    config: &ProductConfig,
    repo_root: &Path,
    applied_by: &str,
    created: &[CreatedArtifact],
    changed: &[ChangedArtifact],
    deleted: &[DeletedArtifact],
) {
    let requests_rel = &config.paths.requests;
    let log_p = crate::request_log::log_path(repo_root, Some(requests_rel));
    let commit = crate::request_log::git_identity::resolve_commit(repo_root);
    let entry_type = match request.request_type {
        RequestType::Create => crate::request_log::entry::EntryType::Create,
        RequestType::Change => crate::request_log::entry::EntryType::Change,
        RequestType::CreateAndChange => crate::request_log::entry::EntryType::CreateAndChange,
        RequestType::Delete => crate::request_log::entry::EntryType::Delete,
    };
    let created_refs: Vec<crate::request_log::ArtifactRef> = created
        .iter()
        .map(|c| crate::request_log::ArtifactRef::new(c.id.clone(), c.file.clone()))
        .collect();
    let changed_refs: Vec<crate::request_log::ArtifactRef> = changed
        .iter()
        .map(|c| crate::request_log::ArtifactRef::new(c.id.clone(), c.file.clone()))
        .collect();
    let deleted_refs: Vec<crate::request_log::ArtifactRef> = deleted
        .iter()
        .map(|c| crate::request_log::ArtifactRef::new(c.id.clone(), c.file.clone()))
        .collect();
    let request_json = serde_json::json!({
        "type": request.request_type.to_string(),
        "reason": request.reason,
    });
    let _ = crate::request_log::append::append_apply_entry(
        &log_p,
        crate::request_log::append::ApplyEntryParams {
            entry_type,
            repo_root,
            applied_by,
            commit: &commit,
            reason: &request.reason,
            request_json,
            created: created_refs,
            changed: changed_refs,
            deleted: deleted_refs,
        },
    );
}
