//! Apply pipeline (FT-041, ADR-038).
//!
//! The 13-step pipeline:
//! 1. Pre-apply checksum snapshot
//! 2. Validate (exit early on E-class)
//! 3. Advisory lock (held by caller)
//! 4. Topological sort and ID assignment
//! 5. Ref resolution
//! 6. Compute new-file writes
//! 7. Compute mutated-file writes
//! 8. Batch atomic rename (commit)
//! 9. Run `graph check` as a health monitor
//! 10. Append to `.product/request-log.jsonl`
//! 11. Return summary

pub mod assign;
pub mod checksum;
pub mod mutate;
pub mod plan;
pub mod render;

use super::types::*;
use super::validate::{self, ValidationContext};
use crate::config::ProductConfig;
use crate::fileops;
use crate::graph::KnowledgeGraph;
use crate::parser;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct ApplyOptions {
    /// Never write files — validate only.
    pub dry_run: bool,
    /// Skip git identity check (used by tests and migration).
    pub skip_git_identity: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatedArtifact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,
    pub id: String,
    pub file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangedArtifact {
    pub id: String,
    pub mutations: usize,
    pub file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplyResult {
    pub applied: bool,
    pub created: Vec<CreatedArtifact>,
    pub changed: Vec<ChangedArtifact>,
    pub findings: Vec<Finding>,
    pub graph_check_clean: bool,
    /// FT-053 / ADR-045 — features whose `started` tag was created as part
    /// of this apply (first `planned → in-progress` transition, or direct
    /// creation with `status: in-progress`). Empty when git is unavailable.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub started_tags: Vec<String>,
    /// FT-053 / ADR-045 — W-class messages emitted when started-tag creation
    /// was skipped (git unavailable) or failed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub started_tag_warnings: Vec<String>,
}

impl ApplyResult {
    #[allow(dead_code)]
    pub fn errors(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.is_error()).collect()
    }
    #[allow(dead_code)]
    pub fn warnings(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| !f.is_error()).collect()
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
            return ApplyResult {
                applied: false, created: Vec::new(), changed: Vec::new(),
                findings: vec![Finding::error("E001", format!("failed to load graph: {}", e), "$")],
                graph_check_clean: false,
                started_tags: Vec::new(), started_tag_warnings: Vec::new(),
            };
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
                return ApplyResult {
                    applied: false, created: Vec::new(), changed: Vec::new(),
                    findings, graph_check_clean: false,
                    started_tags: Vec::new(), started_tag_warnings: Vec::new(),
                };
            }
        }
    };

    let has_errors = findings.iter().any(|f| f.is_error());
    if has_errors || options.dry_run {
        return ApplyResult {
            applied: false, created: Vec::new(), changed: Vec::new(),
            findings, graph_check_clean: !has_errors,
            started_tags: Vec::new(), started_tag_warnings: Vec::new(),
        };
    }

    let ref_to_id = match assign::assign_ids(&request.artifacts, &graph, config) {
        Ok(m) => m,
        Err(f) => {
            findings.push(f);
            return ApplyResult {
                applied: false, created: Vec::new(), changed: Vec::new(),
                findings, graph_check_clean: false,
                started_tags: Vec::new(), started_tag_warnings: Vec::new(),
            };
        }
    };

    let (new_writes, mutation_results) = match plan::plan_writes(
        request, &ref_to_id, &graph, config, repo_root,
    ) {
        Ok(v) => v,
        Err(mut fs) => {
            findings.append(&mut fs);
            return ApplyResult {
                applied: false, created: Vec::new(), changed: Vec::new(),
                findings, graph_check_clean: false,
                started_tags: Vec::new(), started_tag_warnings: Vec::new(),
            };
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
        return ApplyResult {
            applied: false, created: Vec::new(), changed: Vec::new(),
            findings, graph_check_clean: false,
            started_tags: Vec::new(), started_tag_warnings: Vec::new(),
        };
    }

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

    // FT-053 / ADR-045 — best-effort started-tag creation for every feature
    // whose status is now `in-progress` and was either not previously present
    // or was previously `planned`. Tag creation is skipped (W-class warning)
    // when git is unavailable, and never overwrites an existing tag.
    let (started_tags, started_tag_warnings) =
        emit_started_tags(repo_root, &pre_feature_statuses, &post_feature_statuses);

    // FT-041 compat: append to legacy `.product/request-log.jsonl` too.
    let _ = super::log::append_log(repo_root, request, &created, &changed);

    // FT-042: append hash-chained entry to `requests.jsonl` (committed log).
    let requests_rel = &config.paths.requests;
    let log_p = crate::request_log::log_path(repo_root, Some(requests_rel));
    let commit = crate::request_log::git_identity::resolve_commit(repo_root);
    let entry_type = match request.request_type {
        RequestType::Create => crate::request_log::entry::EntryType::Create,
        RequestType::Change => crate::request_log::entry::EntryType::Change,
        RequestType::CreateAndChange => crate::request_log::entry::EntryType::CreateAndChange,
    };
    let created_refs: Vec<crate::request_log::ArtifactRef> = created
        .iter()
        .map(|c| crate::request_log::ArtifactRef::new(c.id.clone(), c.file.clone()))
        .collect();
    let changed_refs: Vec<crate::request_log::ArtifactRef> = changed
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
            applied_by: &applied_by,
            commit: &commit,
            reason: &request.reason,
            request_json,
            created: created_refs,
            changed: changed_refs,
        },
    );

    ApplyResult {
        applied: true,
        created,
        changed,
        findings,
        graph_check_clean,
        started_tags,
        started_tag_warnings,
    }
}

/// FT-053 / ADR-045 — create `product/FT-XXX/started` tags for features that
/// have entered `in-progress` as a result of this apply. Only the first
/// `planned → in-progress` transition (or a fresh creation already at
/// `in-progress`) triggers tag creation; pre-existing `in-progress` features
/// are ignored so replans don't overwrite the earliest-start anchor.
fn emit_started_tags(
    repo_root: &Path,
    pre: &HashMap<String, crate::types::FeatureStatus>,
    post: &HashMap<String, crate::types::FeatureStatus>,
) -> (Vec<String>, Vec<String>) {
    use crate::types::FeatureStatus;
    let mut tags: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut ids: Vec<&String> = post.keys().collect();
    ids.sort();
    for id in ids {
        let post_status = match post.get(id) {
            Some(s) => s,
            None => continue,
        };
        if *post_status != FeatureStatus::InProgress {
            continue;
        }
        // Either the feature is brand new (no pre-apply entry) or was `planned`
        // before. If it was already `in-progress` pre-apply, skip.
        let pre_status = pre.get(id).copied();
        let is_fresh_start = match pre_status {
            None => true,
            Some(FeatureStatus::Planned) => true,
            Some(_) => false,
        };
        if !is_fresh_start {
            continue;
        }
        match crate::tags::create_started_tag(repo_root, id) {
            crate::tags::StartedTagOutcome::Created(name) => tags.push(name),
            crate::tags::StartedTagOutcome::AlreadyExists => {
                // Idempotent no-op — earliest start preserved (decision 5).
            }
            crate::tags::StartedTagOutcome::SkippedNoGit => {
                warnings.push(format!(
                    "warning[W030]: started tag for {} skipped — not a git repository",
                    id
                ));
            }
            crate::tags::StartedTagOutcome::Failed(msg) => {
                warnings.push(format!(
                    "warning[W030]: started tag for {} not created — {}",
                    id, msg
                ));
            }
        }
    }
    (tags, warnings)
}
