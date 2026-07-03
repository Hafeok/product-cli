//! Live repository scan for the §4.3 layout view's tree pane — "the tree, as the
//! model sees it". Walks the real files the blueprint's layout rules cover,
//! attributes each to the allow rule that admits it (or flags it as a no-orphans
//! failure), and emits an indented tree with per-file verdicts.

use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use product_core::pf::blueprint::Blueprint;
use product_core::pf::layout::LayoutModel;
use serde_json::{json, Value};

/// The subset of `rel_paths` git ignores (batched through `git check-ignore`).
/// Empty if git is unavailable or this is not a git work tree — the scan then
/// degrades to showing every file with its rule verdict.
fn git_ignored(repo_root: &Path, rel_paths: &[String]) -> HashSet<String> {
    if rel_paths.is_empty() { return HashSet::new(); }
    let child = Command::new("git")
        .arg("-C").arg(repo_root)
        .args(["check-ignore", "--stdin"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = child else { return HashSet::new() };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(rel_paths.join("\n").as_bytes());
    }
    match child.wait_with_output() {
        Ok(o) => String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().replace('\\', "/")).collect(),
        Err(_) => HashSet::new(),
    }
}

/// A trailing `/**` matches files as `/**/*`.
fn norm(p: &str) -> String {
    p.strip_suffix("/**").map(|b| format!("{b}/**/*")).unwrap_or_else(|| p.to_string())
}

fn glob_rel(root: &Path, pattern: &str) -> Vec<String> {
    let joined = root.join(norm(pattern));
    glob::glob(&joined.to_string_lossy())
        .map(|it| it.flatten().filter(|p| p.is_file())
            .filter_map(|p| p.strip_prefix(root).ok().map(|r| r.to_string_lossy().replace('\\', "/")))
            .collect())
        .unwrap_or_default()
}

/// Project the live repo tree (rows: `{line, dir?, rule?, verdict, note?}`).
pub fn project_repo_tree(repo_root: &Path, product_name: &str) -> Value {
    let Some(model) = load_layout(repo_root) else { return json!([]) };

    // file → (rule id, verdict, note). Allow rules first (may-exist / must-exist),
    // then no-orphans scopes flag anything an allow rule didn't claim.
    let mut files: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    for r in &model.layout {
        if let Some(g) = r.may_exist_here.as_deref().or(r.must_exist.as_deref()) {
            for f in glob_rel(repo_root, g) {
                files.entry(f).or_insert_with(|| (r.id.clone(), "ok".into(), String::new()));
            }
        }
    }
    for r in &model.layout {
        if let Some(scope) = &r.no_orphans {
            for f in glob_rel(repo_root, scope) {
                files.entry(f).or_insert_with(|| (r.id.clone(), "fail".into(), "matches no allow rule".into()));
            }
        }
    }
    if files.is_empty() { return json!([]); }

    // Flag anything git ignores — shown, but marked, not silently admitted.
    let paths: Vec<String> = files.keys().cloned().collect();
    for p in git_ignored(repo_root, &paths) {
        if let Some(v) = files.get_mut(&p) {
            *v = (v.0.clone(), "ignored".into(), "ignored by git".into());
        }
    }

    build_tree_rows(product_name, &files)
}

/// The blueprint's layout model, if any.
fn load_layout(repo_root: &Path) -> Option<LayoutModel> {
    let base = repo_root.join(".product");
    let dir = if base.join("blueprints").is_dir() { base.join("blueprints") } else { base.join("archetypes") };
    let name = std::fs::read_dir(&dir).ok()?.flatten().find(|e| e.path().is_dir())?.file_name().into_string().ok()?;
    Blueprint::load_from_dir(&dir.join(&name), &name).ok()?.layout
}

/// Turn the attributed file set into indented tree rows with connectors.
fn build_tree_rows(root_label: &str, files: &BTreeMap<String, (String, String, String)>) -> Value {
    // dir → child names (dirs + files), built from the relative paths.
    let mut kids: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in files.keys() {
        let parts: Vec<&str> = path.split('/').collect();
        for i in 0..parts.len() {
            let parent = if i == 0 { String::new() } else { parts[..i].join("/") };
            let child = parts[..=i].join("/");
            let entry = kids.entry(parent).or_default();
            if !entry.contains(&child) { entry.push(child); }
        }
    }
    let mut rows: Vec<Value> = vec![json!({ "line": format!("{root_label}/"), "dir": true })];
    emit(&mut rows, &kids, files, "", "");
    Value::Array(rows)
}

fn emit(rows: &mut Vec<Value>, kids: &BTreeMap<String, Vec<String>>, files: &BTreeMap<String, (String, String, String)>, parent: &str, prefix: &str) {
    let Some(children) = kids.get(parent) else { return };
    for (i, child) in children.iter().enumerate() {
        let last = i == children.len() - 1;
        let name = child.rsplit('/').next().unwrap_or(child);
        let connector = if last { "└─ " } else { "├─ " };
        let is_dir = kids.contains_key(child);
        if is_dir {
            rows.push(json!({ "line": format!("{prefix}{connector}{name}/"), "dir": true }));
        } else if let Some((rule, verdict, note)) = files.get(child) {
            let mut row = json!({ "line": format!("{prefix}{connector}{name}"), "rule": rule, "verdict": verdict });
            if !note.is_empty() { row["note"] = json!(note); }
            rows.push(row);
        }
        let next_prefix = format!("{prefix}{}", if last { "   " } else { "│  " });
        emit(rows, kids, files, child, &next_prefix);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_ignore_flags_ignored_paths_only() {
        // repo root is the workspace dir (one above this crate).
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let ig = git_ignored(root, &["target/x".into(), "product-mcp/src/lib.rs".into()]);
        // a tracked source file is never ignored…
        assert!(!ig.contains("product-mcp/src/lib.rs"));
        // …and when git is available, the build dir is flagged (empty set only if
        // git is absent, in which case the scan degrades gracefully).
        if !ig.is_empty() {
            assert!(ig.contains("target/x"), "expected target/x to be git-ignored, got {ig:?}");
        }
    }
}
