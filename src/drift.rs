//! Drift detection — spec vs. implementation verification (ADR-023)
//!
//! Structural checks for D003/D004 run locally.
//! Semantic checks for D001/D002 require LLM — stubbed for now.

use crate::error::{ProductError, Result};
use crate::graph::KnowledgeGraph;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Drift types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftSeverity {
    #[serde(rename = "high")]
    High,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "low")]
    Low,
}

impl std::fmt::Display for DriftSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftFinding {
    pub id: String,
    pub code: String,
    pub severity: DriftSeverity,
    pub description: String,
    pub adr_id: String,
    pub source_files: Vec<String>,
    pub suggested_action: String,
    #[serde(default)]
    pub suppressed: bool,
}

// ---------------------------------------------------------------------------
// Drift baseline (drift.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DriftBaseline {
    #[serde(rename = "schema-version", default = "default_schema")]
    pub schema_version: String,
    #[serde(default)]
    pub suppressions: Vec<DriftSuppression>,
    #[serde(default)]
    pub resolved: Vec<DriftResolved>,
}

fn default_schema() -> String {
    "1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftSuppression {
    pub id: String,
    pub reason: String,
    #[serde(default)]
    pub suppressed_by: String,
    #[serde(default)]
    pub suppressed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftResolved {
    pub id: String,
    #[serde(default)]
    pub resolved_at: String,
}

impl DriftBaseline {
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            ProductError::IoError(format!("failed to serialize drift.json: {}", e))
        })?;
        crate::fileops::write_file_atomic(path, &json)
    }

    pub fn is_suppressed(&self, id: &str) -> bool {
        self.suppressions.iter().any(|s| s.id == id)
    }

    pub fn suppress(&mut self, id: &str, reason: &str) {
        if !self.is_suppressed(id) {
            self.suppressions.push(DriftSuppression {
                id: id.to_string(),
                reason: reason.to_string(),
                suppressed_by: String::new(),
                suppressed_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    pub fn unsuppress(&mut self, id: &str) {
        self.suppressions.retain(|s| s.id != id);
    }
}

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

fn extract_source_files_from_content(body: &str) -> Vec<String> {
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

/// Check for drift between an ADR and associated source files
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
    }

    // D003: Partial implementation heuristic — ADR mentions multiple concepts but
    // source files only reference some of them
    // (This is a simplified structural check — full D001/D002 require LLM)

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("drift.json");
        let mut baseline = DriftBaseline::default();
        baseline.suppress("DRIFT-ADR001-D003-test", "known partial");
        baseline.save(&path).expect("save");

        let loaded = DriftBaseline::load(&path);
        assert_eq!(loaded.suppressions.len(), 1);
    }

    #[test]
    fn extract_source_files() {
        let body = "Some text.\n\nsource-files:\n  - src/consensus/raft.rs\n  - src/consensus/leader.rs\n\nMore text.";
        let files = extract_source_files_from_content(body);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], "src/consensus/raft.rs");
    }

    #[test]
    fn scan_finds_adr_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src = dir.path().join("test.rs");
        std::fs::write(&src, "// Implements ADR-002 consensus\nfn leader() {}").expect("write");

        let adr = Adr {
            front: AdrFrontMatter {
                id: "ADR-002".to_string(),
                title: "Consensus".to_string(),
                status: AdrStatus::Accepted,
                features: vec![],
                supersedes: vec![],
                superseded_by: vec![],
                domains: vec![],
                scope: AdrScope::FeatureSpecific,
            },
            body: String::new(),
            path: PathBuf::from("adr.md"),
        };
        let graph = KnowledgeGraph::build(vec![], vec![adr], vec![]);
        let result = scan_source(&src, &graph);
        assert!(result.contains(&"ADR-002".to_string()));
    }
}
