//! Layout-conformance check — apply a layout model to a repository tree (§4.3).
//!
//! The §6.2 "cheapest gate, run first": walk the filesystem under a root and
//! apply each glob-based rule — `must_exist` (with cardinality), `must_not_exist`,
//! `must_co_exist`, and the allowlist-anchoring `no_orphans`. `may_exist_here`
//! and `must_exist` globs form the allow set that `no_orphans` enforces, so the
//! *unanticipated* file is the failure case. Globs match paths, not meaning.

use std::path::Path;

use super::layout::{LayoutModel, LayoutRule};
use super::validate::Violation;

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation { focus: focus.to_string(), path: path.to_string(), message: message.to_string(), severity: "violation".to_string() }
}

/// Apply `model`'s rules to the tree under `root`; returns all violations.
pub fn check_layout(model: &LayoutModel, root: &Path) -> Vec<Violation> {
    let mut out = Vec::new();
    let allow = allow_patterns(model, root);
    for rule in &model.layout {
        if rule.must_exist.is_some() {
            check_must_exist(rule, root, &mut out);
        }
        if let Some(pat) = &rule.must_not_exist {
            check_must_not_exist(rule, pat, root, &mut out);
        }
        if let Some(co) = &rule.must_co_exist {
            check_co_exist(rule, co, root, &mut out);
        }
        if let Some(scope) = &rule.no_orphans {
            check_no_orphans(rule, scope, root, &allow, &mut out);
        }
    }
    out
}

/// Normalize a directory-recursive glob so it matches files: a trailing `/**`
/// (the natural "everything under here") becomes `/**/*`, which the glob engine
/// matches against actual files.
fn norm(pattern: &str) -> String {
    match pattern.strip_suffix("/**") {
        Some(base) => format!("{base}/**/*"),
        None => pattern.to_string(),
    }
}

/// Glob a pattern relative to `root`, returning matched paths.
fn matches(root: &Path, pattern: &str) -> Vec<std::path::PathBuf> {
    let joined = root.join(norm(pattern));
    match glob::glob(&joined.to_string_lossy()) {
        Ok(paths) => paths.flatten().collect(),
        Err(_) => Vec::new(),
    }
}

/// The allow set `no_orphans` checks against: every `may_exist_here` and
/// `must_exist` glob, as absolute `glob::Pattern`s.
fn allow_patterns(model: &LayoutModel, root: &Path) -> Vec<glob::Pattern> {
    model.layout.iter()
        .filter_map(|r| r.may_exist_here.as_deref().or(r.must_exist.as_deref()))
        .filter_map(|p| glob::Pattern::new(&root.join(norm(p)).to_string_lossy()).ok())
        .collect()
}

fn check_must_exist(rule: &LayoutRule, root: &Path, out: &mut Vec<Violation>) {
    let pattern = rule.must_exist.as_deref().unwrap_or_default();
    // "1 per scope": quantify over each directory matched by for_each.
    if let Some(scope) = &rule.for_each {
        for dir in matches(root, scope).into_iter().filter(|p| p.is_dir()) {
            let resolved = pattern.replace("{dir}", &dir.to_string_lossy());
            let n = glob::glob(&resolved).map(|p| p.flatten().count()).unwrap_or(0);
            if n != 1 {
                out.push(v(&rule.id, "must_exist",
                    &format!("§4.3 must_exist '{pattern}' (1 per scope): expected exactly 1 under {}, found {n}", dir.display())));
            }
        }
        return;
    }
    let n = matches(root, pattern).len();
    let ok = match rule.cardinality.as_deref() {
        Some("exactly 1") => n == 1,
        Some("at least 1") => n >= 1,
        _ => n >= 1,
    };
    if !ok {
        out.push(v(&rule.id, "must_exist",
            &format!("§4.3 must_exist '{pattern}' ({}): found {n}", rule.cardinality.as_deref().unwrap_or("at least 1"))));
    }
}

fn check_must_not_exist(rule: &LayoutRule, pattern: &str, root: &Path, out: &mut Vec<Violation>) {
    let hits = matches(root, pattern);
    if !hits.is_empty() {
        out.push(v(&rule.id, "must_not_exist",
            &format!("§4.3 must_not_exist '{pattern}': forbidden file(s) present ({})", hits.len())));
    }
}

fn check_co_exist(rule: &LayoutRule, co: &super::layout::CoExist, root: &Path, out: &mut Vec<Violation>) {
    for scope in matches(root, &co.when) {
        for required in &co.require {
            if matches(&scope, required).is_empty() {
                out.push(v(&rule.id, "must_co_exist",
                    &format!("§4.3 must_co_exist: '{}' present but required sibling '{required}' missing under {}", co.when, scope.display())));
            }
        }
    }
}

fn check_no_orphans(rule: &LayoutRule, scope: &str, root: &Path, allow: &[glob::Pattern], out: &mut Vec<Violation>) {
    for file in matches(root, scope).into_iter().filter(|p| p.is_file()) {
        if !allow.iter().any(|p| p.matches_path(&file)) {
            out.push(v(&rule.id, "no_orphans",
                &format!("§4.3 no_orphans '{scope}': {} matches no allow rule (allowlist semantics)", file.display())));
        }
    }
}

#[cfg(test)]
#[path = "layout_check_tests.rs"]
mod tests;
