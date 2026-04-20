//! FT-047 / ADR-041: removal & deprecation validation.
//!
//! Exposes two checks invoked by `KnowledgeGraph::check`:
//! - W022: ADR has `removes`/`deprecates` but no linked absence TC.
//! - W023: a front-matter field named in an accepted ADR's `deprecates` list
//!   is present in a loaded artifact.

use super::model::KnowledgeGraph;
use crate::error::{CheckResult, Diagnostic};
use crate::types::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Run both W022 and W023 checks.
pub fn check_all(graph: &KnowledgeGraph, result: &mut CheckResult) {
    check_removes_deprecates_has_absence_tc(graph, result);
    check_deprecated_fields_in_use(graph, result);
}

/// E006 (ADR-042): every Custom TC type must be in `[tc-types].custom`.
pub fn check_unknown_tc_types(
    graph: &KnowledgeGraph,
    config: &crate::config::ProductConfig,
    result: &mut CheckResult,
) {
    for t in graph.tests.values() {
        if let TestType::Custom(name) = &t.front.test_type {
            if !config.is_known_tc_type(name) {
                let msg = format!(
                    "{} declares type '{}' which is not in [tc-types].custom",
                    t.front.id, name
                );
                result.errors.push(
                    Diagnostic::error("E006", "unknown TC type")
                        .with_file(t.path.clone())
                        .with_detail(&msg)
                        .with_hint(&config.tc_type_hint()),
                );
            }
        }
    }
}

/// W022 emitter — mirrors the G009 rule in `gap::check`.
pub fn check_removes_deprecates_has_absence_tc(
    graph: &KnowledgeGraph,
    result: &mut CheckResult,
) {
    for adr in graph.adrs.values() {
        if adr.front.removes.is_empty() && adr.front.deprecates.is_empty() {
            continue;
        }
        let has_absence = graph.tests.values().any(|t| {
            t.front.test_type == TestType::Absence
                && t.front.validates.adrs.contains(&adr.front.id)
        });
        if has_absence {
            continue;
        }
        let detail = if !adr.front.removes.is_empty() && !adr.front.deprecates.is_empty() {
            format!(
                "{} declares removes/deprecates but no linked `tc-type: absence` TC",
                adr.front.id
            )
        } else if !adr.front.removes.is_empty() {
            format!(
                "{} declares `removes` but no linked `tc-type: absence` TC",
                adr.front.id
            )
        } else {
            format!(
                "{} declares `deprecates` but no linked `tc-type: absence` TC",
                adr.front.id
            )
        };
        result.warnings.push(
            Diagnostic::warning("W022", "removal/deprecation without absence TC")
                .with_file(adr.path.clone())
                .with_detail(&detail)
                .with_hint(
                    "create a TC with `tc-type: absence` whose `validates.adrs` links this ADR",
                ),
        );
    }
}

/// W023 emitter — scans every loaded artifact's front-matter for top-level
/// scalar keys that match names in any accepted ADR's `deprecates` list.
/// Non-blocking: the field is still parsed and the graph still builds.
pub fn check_deprecated_fields_in_use(graph: &KnowledgeGraph, result: &mut CheckResult) {
    let deprecated = collect_deprecated_field_map(graph);
    if deprecated.is_empty() {
        return;
    }
    for path in artifact_paths_for_scan(graph) {
        scan_file_for_deprecated_fields(&path, &deprecated, result);
    }
}

fn collect_deprecated_field_map(graph: &KnowledgeGraph) -> HashMap<String, String> {
    let mut deprecated: HashMap<String, String> = HashMap::new();
    for adr in graph.adrs.values() {
        if adr.front.status != AdrStatus::Accepted {
            continue;
        }
        for name in &adr.front.deprecates {
            let key = name.trim().to_string();
            if key.is_empty() {
                continue;
            }
            deprecated.entry(key).or_insert_with(|| adr.front.id.clone());
        }
    }
    deprecated
}

fn artifact_paths_for_scan(graph: &KnowledgeGraph) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for f in graph.features.values() {
        paths.push(f.path.clone());
    }
    for a in graph.adrs.values() {
        paths.push(a.path.clone());
    }
    for t in graph.tests.values() {
        paths.push(t.path.clone());
    }
    for d in graph.dependencies.values() {
        paths.push(d.path.clone());
    }
    paths
}

fn scan_file_for_deprecated_fields(
    path: &Path,
    deprecated: &HashMap<String, String>,
    result: &mut CheckResult,
) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let trimmed = content.trim_start();
    let after = match trimmed.strip_prefix("---") {
        Some(rest) => rest,
        None => return,
    };
    let fm = match after.find("\n---") {
        Some(end) => &after[..end],
        None => return,
    };
    let re = match regex::Regex::new(r"(?m)^([a-zA-Z_][a-zA-Z0-9_-]*)\s*:") {
        Ok(r) => r,
        Err(_) => return,
    };
    let mut emitted: HashSet<String> = HashSet::new();
    for cap in re.captures_iter(fm) {
        let key = match cap.get(1) {
            Some(m) => m.as_str().to_string(),
            None => continue,
        };
        if !deprecated.contains_key(&key) {
            continue;
        }
        if emitted.contains(&key) {
            continue;
        }
        emitted.insert(key.clone());
        let adr_id = match deprecated.get(&key) {
            Some(id) => id.clone(),
            None => continue,
        };
        result.warnings.push(
            Diagnostic::warning("W023", "deprecated front-matter field in use")
                .with_file(path.to_path_buf())
                .with_detail(&format!("field '{}' is deprecated by {}", key, adr_id))
                .with_hint(
                    "migrate away from this field; it is still parsed but scheduled for removal",
                ),
        );
    }
}
