//! The pair contract check — orphan classes, dead selectors, pair facts.
//!
//! The interceptor sees one file per edit; the pair's failures live
//! *between* the files. This module owns the cross-file knowledge: the
//! pair map from `.ddd/config.yaml` (`pair.units`), the enrichment that
//! stamps pair-level facts onto surface events (both files, direction,
//! token/class id), and the bidirectional class-contract check whose two
//! finding classes — `orphan-class` and `dead-selector` — join
//! `ddd report escapes`. Matching is necessary-condition decomposition;
//! what the checker cannot model it reports as skipped rather than silently
//! passing (`dec/ddd/report-coverage-explicit`).

use std::path::{Path, PathBuf};

use ddd_core::config::{DddConfig, PairConfig};
use ddd_core::detect::glob_match;
use serde::Serialize;

use crate::surface::SurfaceEvent;

/// Stamp pair-level facts onto classified events, and drop decorative
/// classes (config globs — thresholds are data, not code). Called by the
/// interceptor after classification, before demands and rows.
pub fn enrich(root: &Path, file: &Path, config: &DddConfig, events: &mut Vec<SurfaceEvent>) {
    let pair = config.pair.clone().unwrap_or_default();
    let rel = rel_of(root, file);
    let direction = if rel.ends_with(".css") { "definition" } else { "consumption" };
    events.retain(|e| {
        !(e.facts.kind == "class"
            && pair.ignore_classes.iter().any(|g| glob_match(g, &e.facts.name)))
    });
    let counterpart = counterpart_of(root, &rel, &pair).join(";");
    for e in events.iter_mut() {
        e.facts.extra.insert("pair_direction".into(), direction.to_string());
        e.facts.extra.insert("pair_counterpart".into(), counterpart.clone());
    }
}

/// The other side of every pair unit this file belongs to.
fn counterpart_of(root: &Path, rel: &str, pair: &PairConfig) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for unit in &pair.units {
        let (own, other) = if rel.ends_with(".css") {
            (&unit.css, &unit.html)
        } else {
            (&unit.html, &unit.css)
        };
        if own.iter().any(|g| glob_match(g, rel)) {
            for path in resolve(root, other) {
                if !out.contains(&path) {
                    out.push(path);
                }
            }
        }
    }
    out
}

/// Repo-relative paths matching a glob list, in walk order.
fn resolve(root: &Path, globs: &[String]) -> Vec<String> {
    let mut files = Vec::new();
    walk(root, root, &mut files);
    files
        .into_iter()
        .filter(|rel| globs.iter().any(|g| glob_match(g, rel)))
        .collect()
}

const SKIP_DIRS: &[&str] = &[".git", ".ddd", "target", "node_modules", "bin", "obj"];

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !SKIP_DIRS.contains(&name.as_str()) {
                walk(root, &path, out);
            }
        } else if ft.is_file() {
            out.push(rel_of(root, &path));
        }
    }
}

fn rel_of(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

/// One pair-contract violation.
#[derive(Debug, Clone, Serialize)]
pub struct PairFinding {
    /// `orphan-class` (HTML consumes it, no owned CSS defines it) or
    /// `dead-selector` (owned CSS targets structure no paired HTML has).
    pub kind: String,
    /// The class or selector at fault.
    pub name: String,
    /// The file the fault was observed in.
    pub file: String,
    pub detail: String,
}

/// The check's own coverage, stated with its findings.
#[derive(Debug, Default, Serialize)]
pub struct PairCoverage {
    pub units: usize,
    pub html_files: usize,
    pub css_files: usize,
    pub selectors_checked: usize,
    /// Selectors containing only constructs the checker does not model.
    pub selectors_skipped: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct PairReport {
    pub findings: Vec<PairFinding>,
    pub coverage: PairCoverage,
}

impl PairReport {
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Run both directions of the class contract over every configured pair.
pub fn check_pairs(root: &Path, config: &DddConfig) -> PairReport {
    let mut report = PairReport::default();
    let Some(pair) = &config.pair else { return report };
    report.coverage.units = pair.units.len();
    for unit in &pair.units {
        let html_files = resolve(root, &unit.html);
        let css_files = resolve(root, &unit.css);
        report.coverage.html_files += html_files.len();
        report.coverage.css_files += css_files.len();
        check_unit(root, pair, &html_files, &css_files, &mut report);
    }
    report.findings.sort_by(|a, b| (&a.kind, &a.name).cmp(&(&b.kind, &b.name)));
    report
}

fn check_unit(
    root: &Path,
    pair: &PairConfig,
    html_files: &[String],
    css_files: &[String],
    report: &mut PairReport,
) {
    let mut defined: Vec<String> = Vec::new();
    let mut selectors: Vec<(String, String)> = Vec::new();
    for css in css_files {
        let text = read(root, css);
        for (_, name) in super::htmlcss_facts::selector_class_names(&text) {
            if !defined.contains(&name) {
                defined.push(name);
            }
        }
        for sel in top_level_selectors(&text) {
            selectors.push((css.clone(), sel));
        }
    }
    let mut consumed: Vec<(String, String)> = Vec::new();
    let mut structure = Structure::default();
    for html in html_files {
        let text = read(root, html);
        collect_structure(&text, html, &mut consumed, &mut structure);
    }
    // Direction 1: classes consumed by HTML ⊆ classes defined by owned CSS.
    for (file, class) in &consumed {
        let decorative = pair.ignore_classes.iter().any(|g| glob_match(g, class));
        if !decorative && !defined.contains(class) {
            report.findings.push(PairFinding {
                kind: "orphan-class".into(),
                name: class.clone(),
                file: file.clone(),
                detail: format!("consumed in {file}, defined in no owned stylesheet"),
            });
        }
    }
    // Direction 2: selectors in owned CSS ⊆ structures present in the HTML.
    let classes_consumed: Vec<&str> = consumed.iter().map(|(_, c)| c.as_str()).collect();
    for (file, sel) in &selectors {
        match selector_alive(sel, &classes_consumed, &structure) {
            Verdict::Alive => report.coverage.selectors_checked += 1,
            Verdict::Unmodelled => report.coverage.selectors_skipped.push(sel.clone()),
            Verdict::Dead(part) => {
                report.coverage.selectors_checked += 1;
                report.findings.push(PairFinding {
                    kind: "dead-selector".into(),
                    name: sel.clone(),
                    file: file.clone(),
                    detail: format!("`{part}` matches nothing in the paired HTML"),
                });
            }
        }
    }
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(PathBuf::from(root).join(rel)).unwrap_or_default()
}

/// Everything the modelled structure of the paired HTML contains.
#[derive(Debug, Default)]
struct Structure {
    tags: Vec<String>,
    ids: Vec<String>,
}

fn collect_structure(
    text: &str,
    file: &str,
    consumed: &mut Vec<(String, String)>,
    structure: &mut Structure,
) {
    for (_, tag) in super::htmlcss_facts::tags(text) {
        let name = tag.name.to_ascii_lowercase();
        if !structure.tags.contains(&name) {
            structure.tags.push(name);
        }
        if let Some(id) = tag.attr("id") {
            if !structure.ids.contains(&id) {
                structure.ids.push(id);
            }
        }
        if let Some(classes) = tag.attr("class") {
            for class in classes.split_whitespace() {
                if !consumed.iter().any(|(_, c)| c == class) {
                    consumed.push((file.to_string(), class.to_string()));
                }
            }
        }
    }
}

/// Selector texts heading top-level (or @-group-nested) rule bodies.
fn top_level_selectors(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut head_start = 0usize;
    let mut depth = 0i32;
    for (pos, c) in text.char_indices() {
        match c {
            '{' => {
                if depth >= 0 {
                    let head = text[head_start..pos].trim();
                    if !head.starts_with('@') && !head.is_empty() {
                        for sel in head.split(',') {
                            let sel = sel.trim();
                            if !sel.is_empty() {
                                out.push(sel.to_string());
                            }
                        }
                    }
                }
                depth += 1;
                head_start = pos + 1;
            }
            '}' | ';' => {
                depth -= i32::from(c == '}');
                head_start = pos + 1;
            }
            _ => {}
        }
    }
    out
}

enum Verdict {
    Alive,
    Unmodelled,
    Dead(String),
}

/// Necessary-condition check: split on combinators, decompose each
/// compound into simple parts, strip pseudo-classes/-elements; a modelled
/// part absent from the HTML kills the selector, a selector with no
/// modelled parts at all is reported as skipped.
fn selector_alive(sel: &str, classes: &[&str], structure: &Structure) -> Verdict {
    let mut any_modelled = false;
    for compound in sel.split([' ', '>', '+', '~']).filter(|s| !s.is_empty()) {
        let base = compound.split(':').next().unwrap_or("");
        for part in simple_parts(base) {
            match part {
                Part::Class(c) => {
                    any_modelled = true;
                    if !classes.contains(&c.as_str()) {
                        return Verdict::Dead(format!(".{c}"));
                    }
                }
                Part::Id(id) => {
                    any_modelled = true;
                    if !structure.ids.contains(&id) {
                        return Verdict::Dead(format!("#{id}"));
                    }
                }
                Part::Tag(t) => {
                    any_modelled = true;
                    if !structure.tags.contains(&t) {
                        return Verdict::Dead(t);
                    }
                }
                Part::Unmodelled => {}
            }
        }
    }
    if any_modelled {
        Verdict::Alive
    } else {
        Verdict::Unmodelled
    }
}

enum Part {
    Class(String),
    Id(String),
    Tag(String),
    Unmodelled,
}

/// `div.card#main[data-x]` → Tag(div), Class(card), Id(main), Unmodelled.
fn simple_parts(compound: &str) -> Vec<Part> {
    let mut out = Vec::new();
    let mut rest = compound;
    while !rest.is_empty() {
        let (head, tail) = split_simple(rest);
        rest = tail;
        let part = match head.chars().next() {
            Some('.') => Part::Class(head[1..].to_string()),
            Some('#') => Part::Id(head[1..].to_string()),
            Some('[') | Some('*') => Part::Unmodelled,
            Some(c) if c.is_ascii_alphabetic() => Part::Tag(head.to_ascii_lowercase()),
            _ => Part::Unmodelled,
        };
        if !matches!(&part, Part::Class(x) | Part::Id(x) | Part::Tag(x) if x.is_empty()) {
            out.push(part);
        }
    }
    out
}

/// One simple selector off the front: its text plus the remainder.
fn split_simple(s: &str) -> (&str, &str) {
    if let Some(rest) = s.strip_prefix('[') {
        let end = rest.find(']').map(|p| p + 2).unwrap_or(s.len());
        return (&s[..end], &s[end..]);
    }
    let start = usize::from(s.starts_with(['.', '#']));
    let len = s[start..]
        .find(['.', '#', '['])
        .map(|p| p + start)
        .unwrap_or(s.len());
    let len = len.max(1);
    (&s[..len], &s[len..])
}

#[cfg(test)]
#[path = "htmlcss_pair_tests.rs"]
mod tests;
