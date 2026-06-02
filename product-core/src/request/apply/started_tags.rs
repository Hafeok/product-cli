//! Best-effort started-tag emission for features transitioning into
//! `in-progress` as a result of an apply (FT-053, ADR-045).
//!
//! Only the first `planned → in-progress` transition (or a fresh creation
//! already at `in-progress`) triggers tag creation; pre-existing
//! `in-progress` features are ignored so replans don't overwrite the
//! earliest-start anchor.

use crate::types::FeatureStatus;
use std::collections::HashMap;
use std::path::Path;

/// Create `product/FT-XXX/started` tags for every feature that has just
/// entered `in-progress`. Returns `(created tags, warnings)`. Warnings are
/// W024-formatted strings — caller decides where to surface them.
pub fn emit_started_tags(
    repo_root: &Path,
    pre: &HashMap<String, FeatureStatus>,
    post: &HashMap<String, FeatureStatus>,
) -> (Vec<String>, Vec<String>) {
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
        // Either the feature is brand new (no pre-apply entry) or was
        // `planned` before. If it was already `in-progress` pre-apply, skip.
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
                    "warning[W024]: started tag for {} skipped — not a git repository",
                    id
                ));
            }
            crate::tags::StartedTagOutcome::Failed(msg) => {
                warnings.push(format!(
                    "warning[W024]: started tag for {} not created — {}",
                    id, msg
                ));
            }
        }
    }
    (tags, warnings)
}
