//! Git tag operations for the `product/` namespace (ADR-036)

use crate::config::ProductConfig;
use crate::error::{ProductError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Format a tag name: `product/{artifact-id}/{event}`
pub fn format_tag_name(artifact_id: &str, event: &str) -> String {
    format!("product/{}/{}", artifact_id, event)
}

/// Parse `product/{id}/{event}` into (artifact_id, event).
pub fn parse_tag_name(tag_name: &str) -> Option<(String, String)> {
    let rest = tag_name.strip_prefix("product/")?;
    let slash_pos = rest.rfind('/')?;
    if slash_pos == 0 || slash_pos == rest.len() - 1 { return None; }
    Some((rest[..slash_pos].to_string(), rest[slash_pos + 1..].to_string()))
}

/// Check if the given root is inside a git repository.
pub fn is_git_repo(root: &Path) -> bool {
    Command::new("git").args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(root).stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null()).status()
        .map(|s| s.success()).unwrap_or(false)
}

/// Create an annotated tag. Returns the full tag name.
pub fn create_tag(root: &Path, artifact_id: &str, event: &str, message: &str) -> Result<String> {
    let tag_name = format_tag_name(artifact_id, event);
    let output = Command::new("git").args(["tag", "-a", &tag_name, "-m", message])
        .current_dir(root).output()
        .map_err(|e| ProductError::IoError(format!("failed to run git tag: {}", e)))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ProductError::IoError(format!("git tag failed: {}", stderr.trim())));
    }
    Ok(tag_name)
}

/// Check if a tag exists.
pub fn tag_exists(root: &Path, tag_name: &str) -> bool {
    Command::new("git").args(["tag", "-l", tag_name]).current_dir(root).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == tag_name)
        .unwrap_or(false)
}

/// Find the next event version (complete -> complete-v2 -> complete-v3...).
pub fn next_event_version(root: &Path, artifact_id: &str, base_event: &str) -> String {
    if !tag_exists(root, &format_tag_name(artifact_id, base_event)) {
        return base_event.to_string();
    }
    let mut v = 2;
    loop {
        let versioned = format!("{}-v{}", base_event, v);
        if !tag_exists(root, &format_tag_name(artifact_id, &versioned)) { return versioned; }
        v += 1;
    }
}

/// Find the latest completion tag for a feature.
pub fn find_completion_tag(root: &Path, feature_id: &str) -> Option<String> {
    let base = format_tag_name(feature_id, "complete");
    if !tag_exists(root, &base) { return None; }
    let mut latest = base;
    let mut v = 2;
    loop {
        let next = format_tag_name(feature_id, &format!("complete-v{}", v));
        if tag_exists(root, &next) { latest = next; v += 1; } else { break; }
    }
    Some(latest)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub artifact_id: String,
    pub event: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDetail {
    pub name: String,
    pub artifact_id: String,
    pub event: String,
    pub timestamp: String,
    pub message: String,
    pub tagger: String,
}

pub struct TagFilter {
    pub feature: Option<String>,
    pub event_type: Option<String>,
}

/// List all product/* tags with metadata.
pub fn list_tags(root: &Path, filter: &TagFilter) -> Vec<TagInfo> {
    let pattern = match &filter.feature {
        Some(feat) => format!("product/{}/*", feat),
        None => "product/*".to_string(),
    };
    let output = match Command::new("git")
        .args(["tag", "-l", &pattern, "--sort=-creatordate",
               "--format=%(refname:short)\t%(creatordate:iso8601)"])
        .current_dir(root).output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut tags = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        let name = parts.first().copied().unwrap_or_default();
        let timestamp = parts.get(1).copied().unwrap_or_default();
        if let Some((artifact_id, event)) = parse_tag_name(name) {
            if let Some(ref tf) = filter.event_type {
                if event.split('-').next().unwrap_or(&event) != tf.as_str() { continue; }
            }
            tags.push(TagInfo {
                name: name.to_string(), artifact_id, event,
                timestamp: timestamp.trim().to_string(),
            });
        }
    }
    tags
}

/// Show detailed info for a tag.
pub fn show_tag(root: &Path, tag_name: &str) -> Option<TagDetail> {
    let output = Command::new("git")
        .args(["tag", "-l", tag_name,
               "--format=%(refname:short)\t%(creatordate:iso8601)\t%(contents)\t%(taggername) %(taggeremail)"])
        .current_dir(root).output().ok()?;
    if !output.status.success() { return None; }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();
    if line.is_empty() { return None; }
    let parts: Vec<&str> = line.splitn(4, '\t').collect();
    let name = parts.first().copied().unwrap_or_default();
    let (artifact_id, event) = parse_tag_name(name)?;
    Some(TagDetail {
        name: name.to_string(), artifact_id, event,
        timestamp: parts.get(1).copied().unwrap_or_default().trim().to_string(),
        message: parts.get(2).copied().unwrap_or_default().trim().to_string(),
        tagger: parts.get(3).copied().unwrap_or_default().trim().to_string(),
    })
}

/// Show all tags for a specific artifact.
pub fn show_artifact_tags(root: &Path, artifact_id: &str) -> Vec<TagDetail> {
    let filter = TagFilter { feature: Some(artifact_id.to_string()), event_type: None };
    list_tags(root, &filter).iter()
        .filter_map(|t| show_tag(root, &t.name))
        .collect()
}

/// Get the timestamp of a tag.
pub fn tag_timestamp(root: &Path, tag_name: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["tag", "-l", tag_name, "--format=%(creatordate:iso8601)"])
        .current_dir(root).output().ok()?;
    if !output.status.success() { return None; }
    let ts = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if ts.is_empty() { None } else { Some(ts) }
}

/// Find files touched in commits near the tag.
pub fn implementation_files(root: &Path, tag_name: &str, depth: usize) -> Vec<PathBuf> {
    let output = match Command::new("git")
        .args(["rev-list", tag_name, &format!("--max-count={}", depth)])
        .current_dir(root).output()
    {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files: Vec<PathBuf> = Vec::new();
    for commit in stdout.lines().filter(|l| !l.trim().is_empty()) {
        // --root handles the initial commit (no parent)
        let diff = match Command::new("git")
            .args(["diff-tree", "--root", "--no-commit-id", "-r", "--name-only", commit])
            .current_dir(root).output()
        {
            Ok(o) if o.status.success() => o,
            _ => continue,
        };
        for line in String::from_utf8_lossy(&diff.stdout).lines() {
            let p = PathBuf::from(line.trim());
            if !line.trim().is_empty() && !files.contains(&p) { files.push(p); }
        }
    }
    files
}

/// Check for changes to implementation files since a tag.
/// Returns (changed_files, diff_text).
pub fn check_drift_since_tag(root: &Path, tag_name: &str, depth: usize) -> (Vec<String>, String) {
    let impl_files = implementation_files(root, tag_name, depth);
    if impl_files.is_empty() { return (Vec::new(), String::new()); }
    let file_args: Vec<String> = impl_files.iter()
        .filter_map(|p| p.to_str().map(String::from)).collect();
    let mut log_args = vec!["log".into(), "--name-only".into(),
        "--pretty=format:".into(), format!("{}..HEAD", tag_name), "--".into()];
    log_args.extend(file_args.clone());
    let output = match Command::new("git").args(&log_args).current_dir(root).output() {
        Ok(o) if o.status.success() => o,
        _ => return (Vec::new(), String::new()),
    };
    let changed: Vec<String> = String::from_utf8_lossy(&output.stdout).lines()
        .map(|l| l.trim().to_string()).filter(|l| !l.is_empty())
        .collect::<std::collections::HashSet<_>>().into_iter().collect();
    if changed.is_empty() { return (Vec::new(), String::new()); }
    let mut diff_args = vec!["diff".into(), format!("{}..HEAD", tag_name), "--".into()];
    diff_args.extend(file_args);
    let diff_output = match Command::new("git").args(&diff_args).current_dir(root).output() {
        Ok(o) if o.status.success() => o,
        _ => return (changed, String::new()),
    };
    (changed, String::from_utf8_lossy(&diff_output.stdout).to_string())
}

/// Per-file insertion/deletion counts between `TAG` and `HEAD`. Returns a
/// map keyed by file path with `(insertions, deletions)` values. Unknown /
/// binary / renamed files are returned with zeros.
pub fn diff_stats_since_tag(root: &Path, tag_name: &str) -> std::collections::HashMap<String, (u64, u64)> {
    let mut out = std::collections::HashMap::new();
    let output = Command::new("git")
        .args(["diff", "--numstat", &format!("{}..HEAD", tag_name)])
        .current_dir(root)
        .output();
    let stdout = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return out,
    };
    for line in stdout.lines() {
        let mut parts = line.split_whitespace();
        let ins = parts.next().unwrap_or("0").parse::<u64>().unwrap_or(0);
        let del = parts.next().unwrap_or("0").parse::<u64>().unwrap_or(0);
        let path = parts.collect::<Vec<_>>().join(" ");
        if !path.is_empty() {
            out.insert(path, (ins, del));
        }
    }
    out
}

/// Create a completion tag for a feature after verification.
pub fn create_completion_tag(
    root: &Path, feature_id: &str, tc_ids: &[String], config: &ProductConfig,
) -> Result<String> {
    let event = next_event_version(root, feature_id, "complete");
    let total = tc_ids.len();
    let message = format!("{} complete: {}/{} TCs passing ({})",
        feature_id, total, total, tc_ids.join(", "));
    let _auto_push = config.tags.auto_push_tags;
    create_tag(root, feature_id, &event, &message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_tag_name_basic() {
        assert_eq!(format_tag_name("FT-001", "complete"), "product/FT-001/complete");
        assert_eq!(format_tag_name("ADR-002", "accepted"), "product/ADR-002/accepted");
    }

    #[test]
    fn parse_tag_name_valid() {
        let (id, event) = parse_tag_name("product/FT-001/complete").expect("valid");
        assert_eq!(id, "FT-001");
        assert_eq!(event, "complete");
    }

    #[test]
    fn parse_tag_name_versioned() {
        let (id, event) = parse_tag_name("product/FT-001/complete-v2").expect("valid");
        assert_eq!(id, "FT-001");
        assert_eq!(event, "complete-v2");
    }

    #[test]
    fn parse_tag_name_invalid() {
        assert!(parse_tag_name("v1.0.0").is_none());
        assert!(parse_tag_name("product/").is_none());
        assert!(parse_tag_name("product/FT-001").is_none());
    }

    #[test]
    fn tag_namespace_regex_validation() {
        let cases = [("FT-001","complete"),("FT-037","complete-v2"),("FT-100","complete-v3"),
            ("ADR-002","accepted"),("ADR-036","superseded"),("DEP-001","active")];
        let re = regex::Regex::new(r"^product/[A-Z]+-\d{3,}/[a-z][a-z0-9-]*$").expect("regex");
        for (id, event) in cases {
            let tag = format_tag_name(id, event);
            assert!(re.is_match(&tag), "Tag '{}' should match namespace format", tag);
        }
    }

    #[test]
    fn next_event_version_no_git() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert_eq!(next_event_version(tmp.path(), "FT-001", "complete"), "complete");
    }
}
