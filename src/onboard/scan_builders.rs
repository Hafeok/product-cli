//! Candidate builder helpers for the onboard scan pipeline.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::evidence::{extract_import_target, truncate_str};
use super::types::{Candidate, Evidence};

/// Build import frequency counts from file contents.
pub(crate) fn build_import_counts(
    file_contents: &[(PathBuf, String)],
) -> HashMap<String, Vec<(PathBuf, usize)>> {
    let mut import_counts: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();
    for (path, content) in file_contents {
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") || trimmed.starts_with("import ")
                || trimmed.starts_with("from ") || trimmed.starts_with("require(")
                || trimmed.starts_with("const ") && trimmed.contains("require(")
            {
                import_counts
                    .entry(trimmed.to_string())
                    .or_default()
                    .push((path.clone(), line_no + 1));
            }
        }
    }
    import_counts
}

/// Build a consistency Candidate from an import pattern and its locations.
pub(crate) fn build_consistency_candidate(
    source_dir: &Path,
    counter: u32,
    pattern: &str,
    locations: &[(PathBuf, usize)],
) -> Candidate {
    let evidence: Vec<Evidence> = locations
        .iter()
        .take(3)
        .map(|(path, line)| {
            let rel = path.strip_prefix(source_dir).unwrap_or(path);
            Evidence {
                file: rel.to_string_lossy().to_string(),
                line: *line,
                snippet: pattern.to_string(),
                evidence_valid: true,
            }
        })
        .collect();

    Candidate {
        id: format!("DC-{:03}", counter),
        signal_type: "consistency".to_string(),
        title: format!("Consistent use of pattern: {}", truncate_str(pattern, 60)),
        observation: format!(
            "The pattern '{}' appears in {} files across the codebase, suggesting a deliberate consistency decision.",
            truncate_str(pattern, 80),
            locations.len()
        ),
        evidence,
        hypothesised_consequence: format!(
            "Deviating from this pattern would break the consistency contract assumed by {} files.",
            locations.len()
        ),
        confidence: if locations.len() >= 5 { "high" } else { "medium" }.to_string(),
        warnings: Vec::new(),
    }
}

/// Build per-directory import tracking.
pub(crate) fn build_dir_imports(
    source_dir: &Path,
    file_contents: &[(PathBuf, String)],
) -> HashMap<String, HashMap<String, Vec<(PathBuf, usize)>>> {
    let mut dir_imports: HashMap<String, HashMap<String, Vec<(PathBuf, usize)>>> = HashMap::new();
    for (path, content) in file_contents {
        let rel = path.strip_prefix(source_dir).unwrap_or(path);
        let parent_dir = rel
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(import) = extract_import_target(trimmed) {
                dir_imports
                    .entry(import)
                    .or_default()
                    .entry(parent_dir.clone())
                    .or_default()
                    .push((path.clone(), line_no + 1));
            }
        }
    }
    dir_imports
}

/// Build a boundary Candidate from an import confined to a single directory.
pub(crate) fn build_boundary_candidate(
    source_dir: &Path,
    counter: u32,
    import: &str,
    dir: &str,
    locs: &[(PathBuf, usize)],
) -> Candidate {
    let evidence: Vec<Evidence> = locs
        .iter()
        .take(3)
        .map(|(path, line)| {
            let rel = path.strip_prefix(source_dir).unwrap_or(path);
            Evidence {
                file: rel.to_string_lossy().to_string(),
                line: *line,
                snippet: format!("imports {}", import),
                evidence_valid: true,
            }
        })
        .collect();
    let dir_label = if dir.is_empty() { "root" } else { dir };

    Candidate {
        id: format!("DC-{:03}", counter),
        signal_type: "boundary".to_string(),
        title: format!("{} access exclusively through {}", import, dir_label),
        observation: format!(
            "All {} imports of '{}' are confined to the '{}' directory. No other module accesses this resource directly.",
            locs.len(), import, dir_label
        ),
        evidence,
        hypothesised_consequence: format!(
            "Accessing '{}' outside '{}' would bypass the boundary and any guarantees it provides.",
            import, dir_label
        ),
        confidence: "high".to_string(),
        warnings: Vec::new(),
    }
}

/// Build a Candidate from a single comment marker match.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_marker_candidate(
    counter: u32,
    signal_type: &str,
    label: &str,
    confidence: &str,
    consequence: &str,
    rel_path: &Path,
    trimmed_line: &str,
    line_no: usize,
) -> Candidate {
    let observation = if signal_type == "constraint" {
        format!(
            "An explicit constraint marker found at {}:{}: '{}'",
            rel_path.display(), line_no, trimmed_line
        )
    } else {
        format!(
            "A {} marker found at {}:{}",
            signal_type, rel_path.display(), line_no
        )
    };

    Candidate {
        id: format!("DC-{:03}", counter),
        signal_type: signal_type.to_string(),
        title: format!("{}: {}", label, truncate_str(trimmed_line, 60)),
        observation,
        evidence: vec![Evidence {
            file: rel_path.to_string_lossy().to_string(),
            line: line_no,
            snippet: trimmed_line.to_string(),
            evidence_valid: true,
        }],
        hypothesised_consequence: consequence.to_string(),
        confidence: confidence.to_string(),
        warnings: Vec::new(),
    }
}

/// Build a Candidate for a pinned dependency.
pub(crate) fn build_dependency_candidate(
    counter: u32,
    rel_path: &Path,
    trimmed_line: &str,
    line_no: usize,
) -> Candidate {
    Candidate {
        id: format!("DC-{:03}", counter),
        signal_type: "dependency".to_string(),
        title: format!("Pinned dependency: {}", truncate_str(trimmed_line, 60)),
        observation: format!(
            "A dependency is pinned with an explanatory comment at {}:{}",
            rel_path.display(), line_no
        ),
        evidence: vec![Evidence {
            file: rel_path.to_string_lossy().to_string(),
            line: line_no,
            snippet: trimmed_line.to_string(),
            evidence_valid: true,
        }],
        hypothesised_consequence: "Upgrading this dependency past the pinned version would break assumptions the codebase relies on.".to_string(),
        confidence: "medium".to_string(),
        warnings: Vec::new(),
    }
}
