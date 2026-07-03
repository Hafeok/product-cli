//! Live repository scan for the §4.3 layout view's tree pane — "the tree, as the
//! model sees it". Walks the real files the blueprint's layout rules cover,
//! attributes each to the allow rule that admits it (or flags it as a no-orphans
//! failure), and emits an indented tree with per-file verdicts.

use std::collections::BTreeMap;
use std::path::Path;

use product_core::pf::blueprint::Blueprint;
use product_core::pf::layout::LayoutModel;
use serde_json::{json, Value};

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
