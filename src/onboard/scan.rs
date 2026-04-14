//! Phase 1: Scan — heuristic decision candidate extraction (ADR-027)

use crate::error::{ProductError, Result};
use std::path::{Path, PathBuf};

use super::evidence::validate_all_evidence;
use super::scan_builders::{
    build_boundary_candidate, build_consistency_candidate, build_dependency_candidate,
    build_dir_imports, build_import_counts, build_marker_candidate,
};
use super::types::{Candidate, ScanMetadata, ScanOutput};

/// Scan a source directory for decision candidates.
pub fn scan(
    source_dir: &Path,
    max_candidates: Option<usize>,
    evidence_validation: bool,
) -> Result<ScanOutput> {
    if !source_dir.exists() {
        return Err(ProductError::NotFound(format!(
            "source directory does not exist: {}",
            source_dir.display()
        )));
    }

    let files = collect_source_files(source_dir)?;
    let files_scanned = files.len();
    let mut candidates = detect_candidates(source_dir, &files)?;

    if evidence_validation {
        validate_all_evidence(source_dir, &mut candidates);
    }

    let cap = max_candidates.unwrap_or(30);
    candidates.truncate(cap);

    Ok(ScanOutput {
        candidates,
        scan_metadata: ScanMetadata {
            files_scanned,
            prompt_version: "onboard-scan-v1".to_string(),
        },
    })
}

/// Collect all source files in a directory tree.
fn collect_source_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        ProductError::IoError(format!("cannot read directory {}: {}", dir.display(), e))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| ProductError::IoError(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "target" && name != "node_modules" {
                collect_files_recursive(&path, out)?;
            }
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// Detect decision candidates through heuristic pattern analysis.
fn detect_candidates(source_dir: &Path, files: &[PathBuf]) -> Result<Vec<Candidate>> {
    let file_contents = read_all_file_contents(files);
    let mut candidates = Vec::new();
    let mut counter = 0u32;

    detect_consistency(source_dir, &file_contents, &mut candidates, &mut counter);
    detect_boundary(source_dir, &file_contents, files, &mut candidates, &mut counter);
    detect_comment_signal(
        source_dir, &file_contents, &mut candidates, &mut counter,
        "constraint", &["CONSTRAINT:", "INVARIANT:", "MUST NOT", "ALWAYS USE"],
        "Constraint", "high",
        "Violating this explicit constraint would break an assumption the codebase relies on.",
    );
    detect_comment_signal(
        source_dir, &file_contents, &mut candidates, &mut counter,
        "convention", &["CONVENTION:", "BY CONVENTION"],
        "Convention", "medium",
        "Violating this convention would break the codebase's implicit contracts.",
    );
    detect_comment_signal(
        source_dir, &file_contents, &mut candidates, &mut counter,
        "absence", &["DO NOT USE", "NEVER USE", "DELIBERATELY AVOIDED"],
        "Absence", "medium",
        "Introducing the avoided element would conflict with the codebase's chosen approach.",
    );
    detect_dependency(source_dir, &file_contents, &mut candidates, &mut counter);

    for (i, c) in candidates.iter_mut().enumerate() {
        c.id = format!("DC-{:03}", i + 1);
    }

    Ok(candidates)
}

fn read_all_file_contents(files: &[PathBuf]) -> Vec<(PathBuf, String)> {
    let mut file_contents = Vec::new();
    for f in files {
        if let Ok(content) = std::fs::read_to_string(f) {
            file_contents.push((f.clone(), content));
        }
    }
    file_contents
}

fn detect_consistency(
    source_dir: &Path,
    file_contents: &[(PathBuf, String)],
    candidates: &mut Vec<Candidate>,
    counter: &mut u32,
) {
    let import_counts = build_import_counts(file_contents);
    let mut patterns: Vec<(String, Vec<(PathBuf, usize)>)> = import_counts
        .into_iter()
        .filter(|(_, locs)| locs.len() >= 3)
        .collect();
    patterns.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (pattern, locations) in patterns.iter().take(3) {
        *counter += 1;
        candidates.push(build_consistency_candidate(source_dir, *counter, pattern, locations));
    }
}

fn detect_boundary(
    source_dir: &Path,
    file_contents: &[(PathBuf, String)],
    files: &[PathBuf],
    candidates: &mut Vec<Candidate>,
    counter: &mut u32,
) {
    let dir_imports = build_dir_imports(source_dir, file_contents);

    for (import, dirs) in &dir_imports {
        if dirs.len() == 1 && files.len() > 3 {
            let empty_str = String::new();
            let empty_vec = Vec::new();
            let (dir, locs) = dirs.iter().next().unwrap_or((&empty_str, &empty_vec));
            if locs.len() >= 2 {
                *counter += 1;
                candidates.push(build_boundary_candidate(source_dir, *counter, import, dir, locs));
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn detect_comment_signal(
    source_dir: &Path,
    file_contents: &[(PathBuf, String)],
    candidates: &mut Vec<Candidate>,
    counter: &mut u32,
    signal_type: &str,
    markers: &[&str],
    label: &str,
    confidence: &str,
    consequence: &str,
) {
    for (path, content) in file_contents {
        for (line_no, line) in content.lines().enumerate() {
            let upper = line.to_uppercase();
            if markers.iter().any(|m| upper.contains(m)) {
                *counter += 1;
                let rel = path.strip_prefix(source_dir).unwrap_or(path);
                candidates.push(build_marker_candidate(
                    *counter, signal_type, label, confidence, consequence,
                    rel, line.trim(), line_no + 1,
                ));
            }
        }
    }
}

fn detect_dependency(
    source_dir: &Path,
    file_contents: &[(PathBuf, String)],
    candidates: &mut Vec<Candidate>,
    counter: &mut u32,
) {
    for (path, content) in file_contents {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !is_dependency_file(filename) {
            continue;
        }
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains('#') && (trimmed.contains('=') || trimmed.contains(':')) {
                *counter += 1;
                let rel = path.strip_prefix(source_dir).unwrap_or(path);
                candidates.push(build_dependency_candidate(*counter, rel, trimmed, line_no + 1));
            }
        }
    }
}

fn is_dependency_file(filename: &str) -> bool {
    matches!(filename, "Cargo.toml" | "package.json" | "requirements.txt" | "go.mod")
}
