//! Evidence validation helpers for onboarding (ADR-027)

use std::path::Path;

use super::types::Candidate;

/// Post-validate evidence: check that cited files and lines exist.
pub fn validate_all_evidence(source_dir: &Path, candidates: &mut [Candidate]) {
    for candidate in candidates.iter_mut() {
        for ev in candidate.evidence.iter_mut() {
            let full_path = source_dir.join(&ev.file);
            if !full_path.exists() {
                ev.evidence_valid = false;
                if !candidate.warnings.contains(&format!("Evidence file does not exist: {}", ev.file)) {
                    candidate.warnings.push(format!(
                        "Evidence file does not exist: {}",
                        ev.file
                    ));
                }
            } else if let Ok(content) = std::fs::read_to_string(&full_path) {
                let line_count = content.lines().count();
                if ev.line > line_count {
                    ev.evidence_valid = false;
                    if !candidate.warnings.contains(&format!(
                        "Evidence line {} exceeds file length {} in {}",
                        ev.line, line_count, ev.file
                    )) {
                        candidate.warnings.push(format!(
                            "Evidence line {} exceeds file length {} in {}",
                            ev.line, line_count, ev.file
                        ));
                    }
                }
            }
        }
    }
}

/// Truncate a string to max_len, appending "..." if truncated.
pub(crate) fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Extract the import target from an import statement.
pub(crate) fn extract_import_target(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // Rust: use foo::bar;
    if let Some(rest) = trimmed.strip_prefix("use ") {
        let target = rest.split("::").next().unwrap_or("").trim_end_matches(';');
        if !target.is_empty() && target != "std" && target != "core" && target != "alloc" && target != "super" && target != "self" && target != "crate" {
            return Some(target.to_string());
        }
    }
    // Python: import foo / from foo import bar
    if let Some(rest) = trimmed.strip_prefix("import ") {
        let target = rest.split_whitespace().next().unwrap_or("").split('.').next().unwrap_or("");
        if !target.is_empty() {
            return Some(target.to_string());
        }
    }
    if let Some(rest) = trimmed.strip_prefix("from ") {
        let target = rest.split_whitespace().next().unwrap_or("").split('.').next().unwrap_or("");
        if !target.is_empty() && target != "." && target != ".." {
            return Some(target.to_string());
        }
    }
    None
}
