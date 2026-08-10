//! Detected-state assembly — configured plus emitted rules, per manifest.
//!
//! Walks the repo for the three configured sources (`.editorconfig`,
//! `bicepconfig.json`, `Cargo.toml` lints), ingests SARIF files for the
//! emitted source, and joins both into one rule map keyed by manifest
//! namespace. The namespace routing table below is the whole per-language
//! adapter for M2: which rule-id family lands in which manifest file. M3's
//! LSP layer joins on the same key — (namespace, rule_id) — at edit time.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::config::DddConfig;
use crate::configured::ConfiguredRule;
use crate::{bicepconfig, cargolints, editorconfig, htmlvalidateconfig, sarif, stylelintconfig, tokenfile};

/// Everything detection saw, keyed by `(namespace, rule_id)`.
#[derive(Debug, Default)]
pub struct DetectedState {
    pub rules: BTreeMap<(String, String), DetectedRule>,
    /// Namespaces with at least one detection source present — staleness
    /// is only evaluable for these.
    pub sourced: BTreeSet<String>,
    /// Unreadable inputs plus other non-finding observations.
    pub notes: Vec<String>,
}

/// One rule as detection saw it, from either or both sources.
#[derive(Debug, Default, Clone)]
pub struct DetectedRule {
    pub configured: Option<ConfiguredRule>,
    /// Emitted occurrences: (level, total, suppressed-in-source count).
    pub emitted: Option<(String, usize, usize)>,
}

impl DetectedRule {
    /// A config suppression: the configured severity turns the rule off.
    pub fn config_suppressed(&self) -> bool {
        self.configured.as_ref().map(|c| c.disabled).unwrap_or(false)
    }

    /// A source suppression: at least one emitted result was suppressed.
    pub fn source_suppressed(&self) -> bool {
        self.emitted.as_ref().map(|(_, _, s)| *s > 0).unwrap_or(false)
    }

    /// Enabled somewhere: configured on, or actually emitted un-suppressed.
    pub fn enabled(&self) -> bool {
        let configured_on = self.configured.as_ref().map(|c| !c.disabled).unwrap_or(false);
        let emitted_live = self.emitted.as_ref().map(|(_, n, s)| n > s).unwrap_or(false);
        configured_on || emitted_live
    }
}

/// The M2 adapter table: route a rule id (plus the SARIF driver that
/// emitted it) to the manifest namespace that governs it. The web tools'
/// kebab-case ids collide with Bicep's linter names, so their drivers are
/// checked first.
pub fn namespace_for(driver: &str, rule_id: &str) -> &'static str {
    if rule_id.starts_with("clippy::") {
        return "clippy";
    }
    let driver = driver.to_ascii_lowercase();
    if driver.contains("stylelint") {
        return "stylelint";
    }
    if driver.contains("html-validate") {
        return "htmlvalidate";
    }
    if driver.contains("bicep") {
        return "linter";
    }
    if is_bicep_id(rule_id) {
        return "linter";
    }
    "analyzers"
}

/// Bicep ids: `BCP<digits>` compiler codes or kebab-case linter names.
fn is_bicep_id(rule_id: &str) -> bool {
    if let Some(rest) = rule_id.strip_prefix("BCP") {
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    rule_id.contains('-')
        && rule_id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Assemble the detected state for `root`; SARIF paths come from the
/// config `detect.sarif` list (root-relative) plus `extra_sarif` verbatim.
pub fn detect(root: &Path, config: Option<&DddConfig>, extra_sarif: &[PathBuf]) -> DetectedState {
    let mut state = DetectedState::default();
    let ignore = config.map(|c| c.ignore.clone()).unwrap_or_default();
    let mut files = Vec::new();
    walk(root, root, &ignore, &mut files);
    for path in files {
        ingest_config_file(root, &path, &mut state);
    }
    let mut sarif_paths: Vec<PathBuf> = config
        .and_then(|c| c.detect.as_ref())
        .map(|d| d.sarif.iter().map(|p| root.join(p)).collect())
        .unwrap_or_default();
    sarif_paths.extend(extra_sarif.iter().cloned());
    for path in sarif_paths {
        ingest_sarif_file(&path, &mut state);
    }
    if let Some(d) = config.and_then(|c| c.detect.as_ref()) {
        ingest_web_outputs(root, d, &mut state);
        ingest_token_files(root, d, &mut state);
    }
    state
}

/// The web tools' own JSON outputs are the emitted sources for their
/// namespaces (format 4); each listed file marks its namespace sourced
/// even when the run was clean.
fn ingest_web_outputs(root: &Path, d: &crate::config::DetectConfig, state: &mut DetectedState) {
    let listed = [
        ("stylelint", &d.stylelint, stylelintconfig::parse_output as fn(&str) -> _),
        ("htmlvalidate", &d.htmlvalidate, htmlvalidateconfig::parse_output as fn(&str) -> _),
    ];
    for (ns, files, parse) in listed {
        for rel in files.iter() {
            let path = root.join(rel);
            let display = path.to_string_lossy().replace('\\', "/");
            let parsed = std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|t| parse(&t));
            match parsed {
                Ok(pairs) => {
                    state.sourced.insert(ns.to_string());
                    for (rule_id, level) in pairs {
                        let key = (ns.to_string(), rule_id);
                        let entry = state.rules.entry(key).or_default();
                        let (l, count, s) = entry.emitted.take().unwrap_or((level, 0, 0));
                        entry.emitted = Some((l, count + 1, s));
                    }
                }
                Err(e) => state.notes.push(format!("{display}: {e}")),
            }
        }
    }
}

/// Registered token stylesheets are the configured source for the
/// `tokens` namespace: one rule per custom property.
fn ingest_token_files(root: &Path, d: &crate::config::DetectConfig, state: &mut DetectedState) {
    for rel in &d.tokens {
        let path = root.join(rel);
        let display = path.to_string_lossy().replace('\\', "/");
        let parsed = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|t| tokenfile::parse(&t, rel));
        match parsed {
            Ok(rules) => {
                state.sourced.insert("tokens".to_string());
                for rule in rules {
                    let key = ("tokens".to_string(), rule.rule_id.clone());
                    state.rules.entry(key).or_default().configured = Some(rule);
                }
            }
            Err(e) => state.notes.push(format!("{display}: {e}")),
        }
    }
}

fn ingest_config_file(root: &Path, path: &Path, state: &mut DetectedState) {
    let name = path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/");
    let read = || std::fs::read_to_string(path).map_err(|e| e.to_string());
    let (namespace, parsed, dedicated) = match name.as_str() {
        ".editorconfig" => match read() {
            Ok(text) => ("analyzers", Ok(editorconfig::parse(&text, &rel)), false),
            Err(e) => ("analyzers", Err(e), false),
        },
        "bicepconfig.json" => ("linter", read().and_then(|t| bicepconfig::parse(&t, &rel)), true),
        "Cargo.toml" => ("clippy", read().and_then(|t| cargolints::parse(&t, &rel)), false),
        ".stylelintrc.json" => {
            ("stylelint", read().and_then(|t| stylelintconfig::parse_config(&t, &rel)), true)
        }
        ".htmlvalidate.json" => {
            ("htmlvalidate", read().and_then(|t| htmlvalidateconfig::parse_config(&t, &rel)), true)
        }
        _ => return,
    };
    match parsed {
        Ok(rules) => {
            // A generic file (editorconfig, Cargo.toml) only proves the
            // namespace has configured coverage when it yields rules; a
            // dedicated one (bicepconfig.json) proves it by existing.
            if dedicated || !rules.is_empty() {
                state.sourced.insert(namespace.to_string());
            }
            for rule in rules {
                let key = (namespace.to_string(), rule.rule_id.clone());
                state.rules.entry(key).or_default().configured = Some(rule);
            }
        }
        Err(e) => state.notes.push(format!("{rel}: unreadable — {e}")),
    }
}

fn ingest_sarif_file(path: &Path, state: &mut DetectedState) {
    let display = path.to_string_lossy().replace('\\', "/");
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            state.notes.push(format!("{display}: unreadable — {e}"));
            return;
        }
    };
    match sarif::parse(&text) {
        Ok(runs) => {
            for run in runs {
                // A run is an emitted source for its driver's namespace even
                // when the build was clean (zero results).
                state.sourced.insert(namespace_for(&run.driver, "").to_string());
                for d in run.results {
                    let ns = namespace_for(&run.driver, &d.rule_id);
                    state.sourced.insert(ns.to_string());
                    let key = (ns.to_string(), d.rule_id.clone());
                    let entry = state.rules.entry(key).or_default();
                    let (level, count, suppressed) =
                        entry.emitted.take().unwrap_or((d.level.clone(), 0, 0));
                    entry.emitted =
                        Some((level, count + 1, suppressed + usize::from(d.suppressed)));
                }
            }
        }
        Err(e) => state.notes.push(format!("{display}: {e}")),
    }
}

/// Directory names never worth descending into.
const SKIP_DIRS: &[&str] = &[".git", ".ddd", "target", "node_modules", "bin", "obj"];

fn walk(root: &Path, dir: &Path, ignore: &[String], out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        if ignore.iter().any(|g| glob_match(g, &rel)) {
            continue;
        }
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !SKIP_DIRS.contains(&name.as_str()) {
                walk(root, &path, ignore, out);
            }
        } else if ft.is_file() {
            out.push(path);
        }
    }
}

/// Match a repo-relative path against an ignore glob: `**` crosses `/`,
/// `*` plus `?` stay within one segment. A bare prefix glob (`dir/**`)
/// also matches the directory itself so the walk prunes it.
pub fn glob_match(glob: &str, path: &str) -> bool {
    if let Some(prefix) = glob.strip_suffix("/**") {
        if path == prefix {
            return true;
        }
    }
    segments_match(&glob.split('/').collect::<Vec<_>>(), &path.split('/').collect::<Vec<_>>())
}

fn segments_match(glob: &[&str], path: &[&str]) -> bool {
    match (glob.first(), path.first()) {
        (None, None) => true,
        (Some(&"**"), _) => {
            segments_match(&glob[1..], path)
                || (!path.is_empty() && segments_match(glob, &path[1..]))
        }
        (Some(g), Some(p)) => segment_match(g, p) && segments_match(&glob[1..], &path[1..]),
        _ => false,
    }
}

fn segment_match(glob: &str, s: &str) -> bool {
    let (g, c): (Vec<char>, Vec<char>) = (glob.chars().collect(), s.chars().collect());
    chars_match(&g, &c)
}

fn chars_match(glob: &[char], s: &[char]) -> bool {
    match (glob.first(), s.first()) {
        (None, None) => true,
        (Some('*'), _) => {
            chars_match(&glob[1..], s) || (!s.is_empty() && chars_match(glob, &s[1..]))
        }
        (Some('?'), Some(_)) => chars_match(&glob[1..], &s[1..]),
        (Some(g), Some(c)) => g == c && chars_match(&glob[1..], &s[1..]),
        _ => false,
    }
}

#[cfg(test)]
#[path = "detect_tests.rs"]
mod tests;
