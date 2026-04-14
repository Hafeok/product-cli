//! Drift checks — source file resolution, structural analysis (ADR-023)

use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::path::{Path, PathBuf};

use super::{DriftBaseline, DriftFinding, DriftSeverity};

// ---------------------------------------------------------------------------
// Source file resolution
// ---------------------------------------------------------------------------

/// Resolve source files for an ADR
pub fn resolve_source_files(
    adr: &Adr,
    _graph: &KnowledgeGraph,
    root: &Path,
    source_roots: &[String],
    ignore: &[String],
    explicit_files: &[String],
) -> Vec<PathBuf> {
    // Explicit --files override
    if !explicit_files.is_empty() {
        return explicit_files.iter().map(|f| root.join(f)).collect();
    }

    // Check ADR front-matter for source-files field
    // (This is an extra YAML field not in our struct — check body for it)
    let source_files_from_body = extract_source_files_from_content(&adr.body);
    if !source_files_from_body.is_empty() {
        return source_files_from_body.iter().map(|f| root.join(f)).collect();
    }

    // Pattern-based discovery: search source roots for files mentioning the ADR ID
    let mut found = Vec::new();
    for src_root in source_roots {
        let search_dir = root.join(src_root);
        if search_dir.exists() {
            find_files_mentioning(&search_dir, &adr.front.id, ignore, &mut found, 20);
        }
    }
    found
}

pub(crate) fn extract_source_files_from_content(body: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut in_source_section = false;
    for line in body.lines() {
        if line.trim().starts_with("source-files:") || line.trim().starts_with("source_files:") {
            in_source_section = true;
            continue;
        }
        if in_source_section {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("- ") {
                files.push(rest.trim().to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                in_source_section = false;
            }
        }
    }
    files
}

fn find_files_mentioning(dir: &Path, pattern: &str, ignore: &[String], results: &mut Vec<PathBuf>, max: usize) {
    if results.len() >= max {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if results.len() >= max {
            return;
        }
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();

        // Skip ignored dirs
        if path.is_dir() {
            if ignore.iter().any(|ig| name == ig.trim_end_matches('/')) {
                continue;
            }
            find_files_mentioning(&path, pattern, ignore, results, max);
            continue;
        }

        // Only check source files
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        if !matches!(ext, "rs" | "py" | "ts" | "js" | "go" | "java" | "cs") {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&path) {
            if content.contains(pattern) {
                results.push(path);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Structural drift checks
// ---------------------------------------------------------------------------

/// Check for drift between an ADR its associated source files
pub fn check_adr(
    adr_id: &str,
    graph: &KnowledgeGraph,
    root: &Path,
    baseline: &DriftBaseline,
    source_roots: &[String],
    ignore: &[String],
    explicit_files: &[String],
) -> Vec<DriftFinding> {
    let mut findings = Vec::new();

    let adr = match graph.adrs.get(adr_id) {
        Some(a) => a,
        None => return findings,
    };

    let source_files = resolve_source_files(adr, graph, root, source_roots, ignore, explicit_files);

    if source_files.is_empty() {
        // D004: No source files found — either not implemented or undocumented
        let id = format!("DRIFT-{}-D004-nofiles", adr_id);
        let suppressed = baseline.is_suppressed(&id);
        findings.push(DriftFinding {
            id,
            code: "D004".to_string(),
            severity: DriftSeverity::Low,
            description: format!("No source files found for {} — decision may not be implemented yet", adr_id),
            adr_id: adr_id.to_string(),
            source_files: vec![],
            suggested_action: "Add source-files to ADR front-matter or check drift.source-roots config".to_string(),
            suppressed,
        });
    } else {
        // Structural D001/D002 checks — compare ADR decision keywords against source
        let decision_keywords = extract_decision_keywords(&adr.body);
        if !decision_keywords.is_empty() {
            let source_contents: Vec<(String, String)> = source_files
                .iter()
                .filter_map(|p| {
                    std::fs::read_to_string(p).ok().map(|c| {
                        (p.to_string_lossy().to_string(), c)
                    })
                })
                .collect();

            if !source_contents.is_empty() {
                let file_names: Vec<String> = source_contents.iter().map(|(p, _)| p.clone()).collect();
                let all_source = source_contents.iter().map(|(_, c)| c.as_str()).collect::<Vec<_>>().join("\n");

                // Check which decision keywords appear in source
                let matched: Vec<&str> = decision_keywords
                    .iter()
                    .filter(|kw| all_source.contains(kw.as_str()))
                    .map(|s| s.as_str())
                    .collect();
                let unmatched: Vec<&str> = decision_keywords
                    .iter()
                    .filter(|kw| !all_source.contains(kw.as_str()))
                    .map(|s| s.as_str())
                    .collect();

                if !unmatched.is_empty() && matched.is_empty() {
                    // No decision keywords found in source at all
                    // Distinguish D001 vs D002 by checking if source has substantial code
                    let has_substantial_code = source_contents.iter().any(|(_, c)| {
                        let code_lines = c.lines()
                            .filter(|l| {
                                let t = l.trim();
                                !t.is_empty() && !t.starts_with("//") && !t.starts_with('#')
                            })
                            .count();
                        code_lines > 3
                    });

                    if has_substantial_code {
                        // D002: Source has code but doesn't use the mandated approach
                        let hash = compute_short_hash(&format!("{}-D002-{}", adr_id, unmatched.join(",")));
                        let id = format!("DRIFT-{}-D002-{}", adr_id, hash);
                        let suppressed = baseline.is_suppressed(&id);
                        findings.push(DriftFinding {
                            id,
                            code: "D002".to_string(),
                            severity: DriftSeverity::High,
                            description: format!(
                                "Decision overridden — {} mandates [{}] but source files use a different approach",
                                adr_id,
                                unmatched.join(", ")
                            ),
                            adr_id: adr_id.to_string(),
                            source_files: file_names,
                            suggested_action: "Update source to match the ADR decision, or update the ADR to reflect the actual approach".to_string(),
                            suppressed,
                        });
                    } else {
                        // D001: Source files exist but are minimal — decision not implemented
                        let hash = compute_short_hash(&format!("{}-D001-{}", adr_id, unmatched.join(",")));
                        let id = format!("DRIFT-{}-D001-{}", adr_id, hash);
                        let suppressed = baseline.is_suppressed(&id);
                        findings.push(DriftFinding {
                            id,
                            code: "D001".to_string(),
                            severity: DriftSeverity::High,
                            description: format!(
                                "Decision not implemented — {} mandates [{}] but no code implements it",
                                adr_id,
                                unmatched.join(", ")
                            ),
                            adr_id: adr_id.to_string(),
                            source_files: file_names,
                            suggested_action: "Implement the decision described in the ADR".to_string(),
                            suppressed,
                        });
                    }
                }
            }
        }
    }

    findings
}

/// Scan a source file to find which ADRs govern it
pub fn scan_source(
    source_path: &Path,
    graph: &KnowledgeGraph,
) -> Vec<String> {
    let content = match std::fs::read_to_string(source_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut relevant = Vec::new();
    for adr in graph.adrs.values() {
        // Check if the source file mentions the ADR ID or key terms from the ADR
        if content.contains(&adr.front.id) {
            relevant.push(adr.front.id.clone());
        }
    }

    relevant
}

// ---------------------------------------------------------------------------
// Decision keyword extraction
// ---------------------------------------------------------------------------

/// Extract key technical terms from the ADR's Decision section.
/// Looks for identifiers like crate names, type names, interface names.
fn extract_decision_keywords(body: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    // Find the Decision section
    let decision_text = extract_decision_section(body);
    if decision_text.is_empty() {
        return keywords;
    }

    // Extract backtick-quoted terms (e.g., `openraft`, `ConsensusInterface`)
    let backtick_re = regex::Regex::new(r"`([A-Za-z_][A-Za-z0-9_:.-]*)`").unwrap_or_else(|_| {
        // Fallback: this regex is always valid
        regex::Regex::new(r"`([A-Za-z_]\w*)`").expect("valid regex")
    });
    for cap in backtick_re.captures_iter(&decision_text) {
        if let Some(m) = cap.get(1) {
            let term = m.as_str().to_string();
            // Filter out very short or common terms
            if term.len() >= 3 && !is_common_word(&term) && !keywords.contains(&term) {
                keywords.push(term);
            }
        }
    }

    // Extract "use X" patterns (e.g., "use openraft", "use Oxigraph")
    let use_re = regex::Regex::new(r"(?i)\buse\s+([A-Za-z_][A-Za-z0-9_-]+)").unwrap_or_else(|_| {
        regex::Regex::new(r"use\s+(\w+)").expect("valid regex")
    });
    for cap in use_re.captures_iter(&decision_text) {
        if let Some(m) = cap.get(1) {
            let term = m.as_str().to_string();
            if term.len() >= 3 && !is_common_word(&term) && !keywords.contains(&term) {
                keywords.push(term);
            }
        }
    }

    keywords
}

/// Extract the Decision section from an ADR body
fn extract_decision_section(body: &str) -> String {
    let mut in_decision = false;
    let mut lines = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        // Look for Decision header (markdown: ## Decision, **Decision:**, etc.)
        if trimmed.contains("Decision") && (trimmed.starts_with('#') || trimmed.starts_with("**") || trimmed.starts_with("Decision")) {
            in_decision = true;
            continue;
        }
        if in_decision {
            // Stop at next section header
            if (trimmed.starts_with('#') || (trimmed.starts_with("**") && trimmed.ends_with("**")))
                && !trimmed.contains("Decision")
            {
                break;
            }
            lines.push(line);
        }
    }

    // If no Decision section found, use the full body
    if lines.is_empty() {
        body.to_string()
    } else {
        lines.join("\n")
    }
}

fn is_common_word(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "the" | "and" | "for" | "not" | "are" | "was" | "has" | "had" | "but"
            | "all" | "can" | "her" | "his" | "one" | "our" | "out" | "you"
            | "use" | "may" | "will" | "shall" | "should" | "must" | "this"
            | "that" | "with" | "from" | "into" | "when" | "each" | "which"
            | "their" | "there" | "these" | "those" | "been" | "have" | "does"
            | "code" | "file" | "data" | "type" | "test" | "true" | "false"
            | "none" | "some" | "impl" | "self" | "pub" | "mod" | "let"
            | "mut" | "ref" | "str" | "any" | "new"
    )
}

fn compute_short_hash(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(input.as_bytes());
    format!("{:x}", hash)[..4].to_string()
}
