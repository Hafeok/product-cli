//! Codebase onboarding — decision discovery from existing code (ADR-027)
//!
//! Three-phase pipeline:
//! 1. **Scan** — detect decision candidates from code patterns (heuristic + evidence validation)
//! 2. **Triage** — structured team review: confirm, reject, merge, skip
//! 3. **Seed** — convert confirmed candidates into ADR files + feature stubs

use crate::error::{ProductError, Result};
use crate::fileops;
use crate::parser;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A piece of evidence grounding a decision candidate in source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub file: String,
    pub line: usize,
    pub snippet: String,
    #[serde(default = "default_evidence_valid")]
    pub evidence_valid: bool,
}

fn default_evidence_valid() -> bool {
    true
}

/// A decision candidate produced by the scan phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub signal_type: String,
    pub title: String,
    pub observation: String,
    pub evidence: Vec<Evidence>,
    pub hypothesised_consequence: String,
    pub confidence: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Metadata about a scan run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    pub files_scanned: usize,
    pub prompt_version: String,
}

/// The output of a scan phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOutput {
    pub candidates: Vec<Candidate>,
    pub scan_metadata: ScanMetadata,
}

/// Triage action for a candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriageAction {
    Confirm,
    Reject,
    Merge(String), // target candidate ID
    Skip,
}

/// Status of a candidate after triage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TriageStatus {
    Confirmed,
    Rejected,
    Merged,
    Skipped,
    Pending,
}

/// A triaged candidate — the original candidate plus triage metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriagedCandidate {
    #[serde(flatten)]
    pub candidate: Candidate,
    pub triage_status: TriageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_into: Option<String>,
}

/// Output of the triage phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageOutput {
    pub candidates: Vec<TriagedCandidate>,
}

/// A proposed feature stub from the seed phase.
#[derive(Debug, Clone)]
pub struct ProposedFeatureStub {
    pub id: String,
    pub title: String,
    pub adr_ids: Vec<String>,
    pub filename: String,
}

/// A proposed ADR from the seed phase.
#[derive(Debug, Clone)]
pub struct ProposedAdr {
    pub id: String,
    pub title: String,
    pub observation: String,
    pub evidence: Vec<Evidence>,
    pub hypothesised_consequence: String,
    pub filename: String,
}

/// Result of the seed phase.
#[derive(Debug, Clone)]
pub struct SeedResult {
    pub adrs: Vec<ProposedAdr>,
    pub features: Vec<ProposedFeatureStub>,
}

// ---------------------------------------------------------------------------
// Phase 1: Scan — heuristic decision candidate extraction
// ---------------------------------------------------------------------------

/// Scan a source directory for decision candidates.
///
/// Uses heuristic pattern detection to identify load-bearing architectural
/// decisions. In production, an LLM could augment or replace this; for
/// deterministic testing, heuristics produce stable candidates.
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

    // Collect all source files
    let files = collect_source_files(source_dir)?;
    let files_scanned = files.len();

    // Run heuristic pattern detection
    let mut candidates = detect_candidates(source_dir, &files)?;

    // Post-validate evidence
    if evidence_validation {
        validate_all_evidence(source_dir, &mut candidates);
    }

    // Apply max-candidates cap
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
            // Skip hidden dirs and common build dirs
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
///
/// Looks for six signal types:
/// - Consistency: same pattern repeated across files
/// - Boundary: only certain modules access a resource
/// - Constraint: all X comes from Y, never Z
/// - Convention: different treatment for categories
/// - Absence: something deliberately not used
/// - Dependency: pinned dependency with explanation
fn detect_candidates(source_dir: &Path, files: &[PathBuf]) -> Result<Vec<Candidate>> {
    let mut candidates = Vec::new();
    let mut candidate_counter = 0u32;

    // Read all file contents
    let mut file_contents: Vec<(PathBuf, String)> = Vec::new();
    for f in files {
        if let Ok(content) = std::fs::read_to_string(f) {
            file_contents.push((f.clone(), content));
        }
    }

    // --- Signal 1: Consistency — same import/pattern across many files ---
    let mut import_counts: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();
    for (path, content) in &file_contents {
        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Look for import/use patterns
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

    // Find imports used in 3+ files — suggests a consistency decision
    let mut consistency_patterns: Vec<(String, Vec<(PathBuf, usize)>)> = import_counts
        .into_iter()
        .filter(|(_, locs)| locs.len() >= 3)
        .collect();
    consistency_patterns.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    for (pattern, locations) in consistency_patterns.iter().take(3) {
        candidate_counter += 1;
        let evidence: Vec<Evidence> = locations
            .iter()
            .take(3)
            .map(|(path, line)| {
                let rel = path.strip_prefix(source_dir).unwrap_or(path);
                Evidence {
                    file: rel.to_string_lossy().to_string(),
                    line: *line,
                    snippet: pattern.clone(),
                    evidence_valid: true,
                }
            })
            .collect();

        candidates.push(Candidate {
            id: format!("DC-{:03}", candidate_counter),
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
        });
    }

    // --- Signal 2: Boundary — only certain modules/directories use a resource ---
    let mut dir_imports: HashMap<String, HashMap<String, Vec<(PathBuf, usize)>>> = HashMap::new();
    for (path, content) in &file_contents {
        let rel = path.strip_prefix(source_dir).unwrap_or(path);
        let parent_dir = rel
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        for (line_no, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Track which directories import which modules/crates
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

    // Find imports confined to a single directory — suggests a boundary decision
    for (import, dirs) in &dir_imports {
        if dirs.len() == 1 && files.len() > 3 {
            let empty_str = String::new();
            let empty_vec = Vec::new();
            let (dir, locs) = dirs.iter().next().unwrap_or((&empty_str, &empty_vec));
            if locs.len() >= 2 {
                candidate_counter += 1;
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

                candidates.push(Candidate {
                    id: format!("DC-{:03}", candidate_counter),
                    signal_type: "boundary".to_string(),
                    title: format!(
                        "{} access exclusively through {}",
                        import,
                        if dir.is_empty() { "root" } else { dir.as_str() }
                    ),
                    observation: format!(
                        "All {} imports of '{}' are confined to the '{}' directory. No other module accesses this resource directly.",
                        locs.len(),
                        import,
                        if dir.is_empty() { "root" } else { dir.as_str() }
                    ),
                    evidence,
                    hypothesised_consequence: format!(
                        "Accessing '{}' outside '{}' would bypass the boundary and any guarantees it provides.",
                        import,
                        if dir.is_empty() { "root" } else { dir.as_str() }
                    ),
                    confidence: "high".to_string(),
                    warnings: Vec::new(),
                });
            }
        }
    }

    // --- Signal 3: Constraint — markers like "// NOTE:", "// CONSTRAINT:" etc. ---
    for (path, content) in &file_contents {
        for (line_no, line) in content.lines().enumerate() {
            let upper = line.to_uppercase();
            if upper.contains("CONSTRAINT:") || upper.contains("INVARIANT:") || upper.contains("MUST NOT") || upper.contains("ALWAYS USE") {
                candidate_counter += 1;
                let rel = path.strip_prefix(source_dir).unwrap_or(path);
                candidates.push(Candidate {
                    id: format!("DC-{:03}", candidate_counter),
                    signal_type: "constraint".to_string(),
                    title: format!("Constraint: {}", truncate_str(line.trim(), 60)),
                    observation: format!(
                        "An explicit constraint marker found at {}:{}: '{}'",
                        rel.display(),
                        line_no + 1,
                        line.trim()
                    ),
                    evidence: vec![Evidence {
                        file: rel.to_string_lossy().to_string(),
                        line: line_no + 1,
                        snippet: line.trim().to_string(),
                        evidence_valid: true,
                    }],
                    hypothesised_consequence: "Violating this explicit constraint would break an assumption the codebase relies on.".to_string(),
                    confidence: "high".to_string(),
                    warnings: Vec::new(),
                });
            }
        }
    }

    // --- Signal 4: Convention — comment markers like "// Convention:" or doc comments ---
    for (path, content) in &file_contents {
        for (line_no, line) in content.lines().enumerate() {
            let upper = line.to_uppercase();
            if upper.contains("CONVENTION:") || upper.contains("BY CONVENTION") {
                candidate_counter += 1;
                let rel = path.strip_prefix(source_dir).unwrap_or(path);
                candidates.push(Candidate {
                    id: format!("DC-{:03}", candidate_counter),
                    signal_type: "convention".to_string(),
                    title: format!("Convention: {}", truncate_str(line.trim(), 60)),
                    observation: format!(
                        "A convention marker found at {}:{}",
                        rel.display(),
                        line_no + 1
                    ),
                    evidence: vec![Evidence {
                        file: rel.to_string_lossy().to_string(),
                        line: line_no + 1,
                        snippet: line.trim().to_string(),
                        evidence_valid: true,
                    }],
                    hypothesised_consequence: "Violating this convention would break the codebase's implicit contracts.".to_string(),
                    confidence: "medium".to_string(),
                    warnings: Vec::new(),
                });
            }
        }
    }

    // --- Signal 5: Absence — "do not use", "never use", "avoid" markers ---
    for (path, content) in &file_contents {
        for (line_no, line) in content.lines().enumerate() {
            let upper = line.to_uppercase();
            if upper.contains("DO NOT USE") || upper.contains("NEVER USE") || upper.contains("DELIBERATELY AVOIDED") {
                candidate_counter += 1;
                let rel = path.strip_prefix(source_dir).unwrap_or(path);
                candidates.push(Candidate {
                    id: format!("DC-{:03}", candidate_counter),
                    signal_type: "absence".to_string(),
                    title: format!("Absence: {}", truncate_str(line.trim(), 60)),
                    observation: format!(
                        "An absence signal found at {}:{}",
                        rel.display(),
                        line_no + 1
                    ),
                    evidence: vec![Evidence {
                        file: rel.to_string_lossy().to_string(),
                        line: line_no + 1,
                        snippet: line.trim().to_string(),
                        evidence_valid: true,
                    }],
                    hypothesised_consequence: "Introducing the avoided element would conflict with the codebase's chosen approach.".to_string(),
                    confidence: "medium".to_string(),
                    warnings: Vec::new(),
                });
            }
        }
    }

    // --- Signal 6: Dependency — pinned versions with comments ---
    for (path, content) in &file_contents {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if filename == "Cargo.toml" || filename == "package.json" || filename == "requirements.txt" || filename == "go.mod" {
            for (line_no, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                // Look for pinned versions with comments
                if trimmed.contains('#') && (trimmed.contains('=') || trimmed.contains(':')) {
                    candidate_counter += 1;
                    let rel = path.strip_prefix(source_dir).unwrap_or(path);
                    candidates.push(Candidate {
                        id: format!("DC-{:03}", candidate_counter),
                        signal_type: "dependency".to_string(),
                        title: format!("Pinned dependency: {}", truncate_str(trimmed, 60)),
                        observation: format!(
                            "A dependency is pinned with an explanatory comment at {}:{}",
                            rel.display(),
                            line_no + 1
                        ),
                        evidence: vec![Evidence {
                            file: rel.to_string_lossy().to_string(),
                            line: line_no + 1,
                            snippet: trimmed.to_string(),
                            evidence_valid: true,
                        }],
                        hypothesised_consequence: "Upgrading this dependency past the pinned version would break assumptions the codebase relies on.".to_string(),
                        confidence: "medium".to_string(),
                        warnings: Vec::new(),
                    });
                }
            }
        }
    }

    // Re-number candidates sequentially
    for (i, c) in candidates.iter_mut().enumerate() {
        c.id = format!("DC-{:03}", i + 1);
    }

    Ok(candidates)
}

/// Extract the import target from an import statement.
fn extract_import_target(line: &str) -> Option<String> {
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

/// Truncate a string to max_len, appending "..." if truncated.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

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

// ---------------------------------------------------------------------------
// Phase 2: Triage — structured review
// ---------------------------------------------------------------------------

/// Triage candidates interactively, reading actions from a BufRead source (stdin or test harness).
pub fn triage_interactive<R: BufRead>(
    scan_output: &ScanOutput,
    input: &mut R,
) -> Result<TriageOutput> {
    let mut triaged = Vec::new();

    // Build a map of pending candidates for merge lookups
    let candidate_map: HashMap<String, &Candidate> = scan_output
        .candidates
        .iter()
        .map(|c| (c.id.clone(), c))
        .collect();

    let mut merged_ids: Vec<String> = Vec::new();

    for candidate in &scan_output.candidates {
        // Skip candidates that have been merged into another
        if merged_ids.contains(&candidate.id) {
            continue;
        }

        // Print candidate info to stderr for interactive display
        eprintln!(
            "--- {} [{}] confidence: {} ---",
            candidate.id, candidate.signal_type, candidate.confidence
        );
        eprintln!("{}", candidate.title);
        eprintln!();
        eprintln!("Observation: {}", candidate.observation);
        eprintln!();
        eprintln!("Evidence:");
        for ev in &candidate.evidence {
            let valid_marker = if ev.evidence_valid { "" } else { " [INVALID]" };
            eprintln!("  {}:{}    {}{}", ev.file, ev.line, ev.snippet, valid_marker);
        }
        eprintln!();
        eprintln!(
            "Hypothesised consequence: {}",
            candidate.hypothesised_consequence
        );
        eprintln!();
        eprintln!("  [c]onfirm  [m]erge with DC-XXX  [r]eject  [s]kip");

        // Read action
        let mut line_buf = String::new();
        let bytes_read = input
            .read_line(&mut line_buf)
            .map_err(|e| ProductError::IoError(format!("failed to read triage input: {}", e)))?;

        if bytes_read == 0 {
            // EOF — skip remaining
            triaged.push(TriagedCandidate {
                candidate: candidate.clone(),
                triage_status: TriageStatus::Skipped,
                merged_into: None,
            });
            continue;
        }

        let action = line_buf.trim().to_lowercase();
        match action.as_str() {
            "c" | "confirm" => {
                triaged.push(TriagedCandidate {
                    candidate: candidate.clone(),
                    triage_status: TriageStatus::Confirmed,
                    merged_into: None,
                });
            }
            "r" | "reject" => {
                triaged.push(TriagedCandidate {
                    candidate: candidate.clone(),
                    triage_status: TriageStatus::Rejected,
                    merged_into: None,
                });
            }
            "s" | "skip" => {
                triaged.push(TriagedCandidate {
                    candidate: candidate.clone(),
                    triage_status: TriageStatus::Skipped,
                    merged_into: None,
                });
            }
            s if s.starts_with('m') || s.starts_with("merge") => {
                // Parse merge target: "m\nDC-001" or "merge DC-001"
                let target_id = s
                    .strip_prefix("merge")
                    .or_else(|| s.strip_prefix("m"))
                    .map(|rest| rest.trim().to_string())
                    .unwrap_or_default();

                // If target wasn't on same line, read next line
                let target_id = if target_id.is_empty() {
                    let mut target_buf = String::new();
                    let _ = input.read_line(&mut target_buf);
                    target_buf.trim().to_string()
                } else {
                    target_id
                };

                if let Some(target) = candidate_map.get(&target_id) {
                    // Merge: current candidate is absorbed into target
                    // Mark this candidate as merged
                    triaged.push(TriagedCandidate {
                        candidate: candidate.clone(),
                        triage_status: TriageStatus::Merged,
                        merged_into: Some(target_id.clone()),
                    });

                    // Find or create the target in triaged list and add evidence
                    let mut found = false;
                    for tc in triaged.iter_mut() {
                        if tc.candidate.id == target_id {
                            // Add evidence from merged candidate
                            tc.candidate
                                .evidence
                                .extend(candidate.evidence.clone());
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        // Target hasn't been processed yet — create a confirmed entry with merged evidence
                        let mut merged_candidate = (*target).clone();
                        merged_candidate
                            .evidence
                            .extend(candidate.evidence.clone());
                        triaged.push(TriagedCandidate {
                            candidate: merged_candidate,
                            triage_status: TriageStatus::Confirmed,
                            merged_into: None,
                        });
                        merged_ids.push(target_id);
                    }
                } else {
                    // Invalid merge target — treat as skip
                    eprintln!(
                        "warning: merge target '{}' not found, skipping {}",
                        target_id, candidate.id
                    );
                    triaged.push(TriagedCandidate {
                        candidate: candidate.clone(),
                        triage_status: TriageStatus::Skipped,
                        merged_into: None,
                    });
                }
            }
            _ => {
                // Unknown action — skip
                triaged.push(TriagedCandidate {
                    candidate: candidate.clone(),
                    triage_status: TriageStatus::Skipped,
                    merged_into: None,
                });
            }
        }
    }

    Ok(TriageOutput {
        candidates: triaged,
    })
}

/// Batch-confirm all candidates (non-interactive triage).
pub fn triage_batch_confirm(scan_output: &ScanOutput) -> TriageOutput {
    TriageOutput {
        candidates: scan_output
            .candidates
            .iter()
            .map(|c| TriagedCandidate {
                candidate: c.clone(),
                triage_status: TriageStatus::Confirmed,
                merged_into: None,
            })
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Phase 3: Seed — create ADR files + feature stubs
// ---------------------------------------------------------------------------

/// Plan the seed phase: determine what files would be created.
pub fn plan_seed(
    triage_output: &TriageOutput,
    existing_adr_ids: &[String],
    existing_feature_ids: &[String],
    adr_prefix: &str,
    feature_prefix: &str,
) -> SeedResult {
    let confirmed: Vec<&TriagedCandidate> = triage_output
        .candidates
        .iter()
        .filter(|c| c.triage_status == TriageStatus::Confirmed)
        .collect();

    // Assign ADR IDs
    let mut adr_ids_used: Vec<String> = existing_adr_ids.to_vec();
    let mut proposed_adrs = Vec::new();

    for tc in &confirmed {
        let adr_id = parser::next_id(adr_prefix, &adr_ids_used);
        adr_ids_used.push(adr_id.clone());

        let filename = parser::id_to_filename(&adr_id, &tc.candidate.title);
        proposed_adrs.push(ProposedAdr {
            id: adr_id,
            title: tc.candidate.title.clone(),
            observation: tc.candidate.observation.clone(),
            evidence: tc.candidate.evidence.clone(),
            hypothesised_consequence: tc.candidate.hypothesised_consequence.clone(),
            filename,
        });
    }

    // Group candidates into feature stubs by evidence file proximity
    let features = group_into_features(&proposed_adrs, existing_feature_ids, feature_prefix);

    SeedResult {
        adrs: proposed_adrs,
        features,
    }
}

/// Group ADRs into feature stubs based on evidence file proximity.
///
/// ADRs whose evidence files share the same parent directory are grouped together.
fn group_into_features(
    adrs: &[ProposedAdr],
    existing_feature_ids: &[String],
    feature_prefix: &str,
) -> Vec<ProposedFeatureStub> {
    if adrs.is_empty() {
        return Vec::new();
    }

    // Build a map: directory → ADR IDs
    let mut dir_to_adrs: HashMap<String, Vec<String>> = HashMap::new();
    for adr in adrs {
        // Get the primary evidence directory
        let primary_dir = adr
            .evidence
            .first()
            .map(|ev| {
                let p = PathBuf::from(&ev.file);
                p.parent()
                    .map(|d| d.to_string_lossy().to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        dir_to_adrs
            .entry(if primary_dir.is_empty() {
                "root".to_string()
            } else {
                primary_dir
            })
            .or_default()
            .push(adr.id.clone());
    }

    // Create one feature stub per directory cluster
    let mut feature_ids_used: Vec<String> = existing_feature_ids.to_vec();
    let mut features = Vec::new();

    let mut dirs: Vec<_> = dir_to_adrs.into_iter().collect();
    dirs.sort_by(|a, b| a.0.cmp(&b.0));

    for (dir, adr_ids) in dirs {
        let feature_id = parser::next_id(feature_prefix, &feature_ids_used);
        feature_ids_used.push(feature_id.clone());

        let title = format!("Onboarded decisions — {}", dir);
        let filename = parser::id_to_filename(&feature_id, &title);

        features.push(ProposedFeatureStub {
            id: feature_id,
            title,
            adr_ids,
            filename,
        });
    }

    features
}

/// Execute the seed phase: write ADR files and feature stubs to disk.
pub fn execute_seed(
    seed_result: &SeedResult,
    adrs_dir: &Path,
    features_dir: &Path,
) -> Result<()> {
    // Write ADR files
    for adr in &seed_result.adrs {
        let path = adrs_dir.join(&adr.filename);
        let content = render_seeded_adr(adr);
        fileops::write_file_atomic(&path, &content)?;
        println!("  created {}", path.display());
    }

    // Write feature stubs
    for feature in &seed_result.features {
        let path = features_dir.join(&feature.filename);
        let content = render_feature_stub(feature);
        fileops::write_file_atomic(&path, &content)?;
        println!("  created {}", path.display());
    }

    Ok(())
}

/// Render an ADR file from a proposed ADR.
fn render_seeded_adr(adr: &ProposedAdr) -> String {
    let front = AdrFrontMatter {
        id: adr.id.clone(),
        title: adr.title.clone(),
        status: AdrStatus::Proposed,
        features: Vec::new(),
        supersedes: Vec::new(),
        superseded_by: Vec::new(),
        domains: Vec::new(),
        scope: AdrScope::FeatureSpecific,
    };

    let mut body = String::new();

    // Context section
    body.push_str("## Context\n\n");
    body.push_str(&adr.observation);
    body.push('\n');
    if !adr.evidence.is_empty() {
        body.push_str("\n**Evidence:**\n");
        for ev in &adr.evidence {
            body.push_str(&format!("- `{}:{}` — {}\n", ev.file, ev.line, ev.snippet));
        }
    }
    body.push('\n');

    // Decision section
    body.push_str("## Decision\n\n");
    body.push_str(&adr.title);
    body.push_str(".\n\n");

    // Rationale
    body.push_str("## Rationale\n\n");
    body.push_str("<!-- TODO: add rationale -->\n\n");

    // Consequence
    body.push_str("## Consequence\n\n");
    body.push_str(&adr.hypothesised_consequence);
    body.push_str("\n\n");

    // Rejected alternatives
    body.push_str("**Rejected alternatives:**\n\n");
    body.push_str("<!-- TODO: add rejected alternatives -->\n");

    parser::render_adr(&front, &body)
}

/// Render a feature stub from a proposed feature.
fn render_feature_stub(feature: &ProposedFeatureStub) -> String {
    let front = FeatureFrontMatter {
        id: feature.id.clone(),
        title: feature.title.clone(),
        phase: 1,
        status: FeatureStatus::Planned,
        depends_on: Vec::new(),
        adrs: feature.adr_ids.clone(),
        tests: Vec::new(),
        domains: Vec::new(),
        domains_acknowledged: std::collections::HashMap::new(),
    };

    let body = format!(
        "Feature stub created by codebase onboarding.\n\nLinked ADRs: {}\n",
        feature.adr_ids.join(", ")
    );

    parser::render_feature(&front, &body)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }

    #[test]
    fn test_extract_import_target() {
        assert_eq!(extract_import_target("use sqlx::PgPool;"), Some("sqlx".to_string()));
        assert_eq!(extract_import_target("import os"), Some("os".to_string()));
        assert_eq!(extract_import_target("from flask import Flask"), Some("flask".to_string()));
        assert_eq!(extract_import_target("use std::path::Path;"), None); // std excluded
        assert_eq!(extract_import_target("let x = 5;"), None);
    }

    #[test]
    fn test_validate_evidence_nonexistent_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut candidates = vec![Candidate {
            id: "DC-001".to_string(),
            signal_type: "boundary".to_string(),
            title: "Test".to_string(),
            observation: "Test observation".to_string(),
            evidence: vec![Evidence {
                file: "nonexistent.rs".to_string(),
                line: 1,
                snippet: "test".to_string(),
                evidence_valid: true,
            }],
            hypothesised_consequence: "Bad things".to_string(),
            confidence: "high".to_string(),
            warnings: Vec::new(),
        }];

        validate_all_evidence(dir.path(), &mut candidates);
        assert!(!candidates[0].evidence[0].evidence_valid);
        assert!(!candidates[0].warnings.is_empty());
    }

    #[test]
    fn test_validate_evidence_valid_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("test.rs"), "line 1\nline 2\nline 3\n").expect("write");

        let mut candidates = vec![Candidate {
            id: "DC-001".to_string(),
            signal_type: "boundary".to_string(),
            title: "Test".to_string(),
            observation: "Test observation".to_string(),
            evidence: vec![Evidence {
                file: "test.rs".to_string(),
                line: 2,
                snippet: "line 2".to_string(),
                evidence_valid: true,
            }],
            hypothesised_consequence: "Bad things".to_string(),
            confidence: "high".to_string(),
            warnings: Vec::new(),
        }];

        validate_all_evidence(dir.path(), &mut candidates);
        assert!(candidates[0].evidence[0].evidence_valid);
        assert!(candidates[0].warnings.is_empty());
    }

    #[test]
    fn test_validate_evidence_line_exceeds() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("test.rs"), "line 1\nline 2\n").expect("write");

        let mut candidates = vec![Candidate {
            id: "DC-001".to_string(),
            signal_type: "boundary".to_string(),
            title: "Test".to_string(),
            observation: "Test observation".to_string(),
            evidence: vec![Evidence {
                file: "test.rs".to_string(),
                line: 99,
                snippet: "does not exist".to_string(),
                evidence_valid: true,
            }],
            hypothesised_consequence: "Bad things".to_string(),
            confidence: "high".to_string(),
            warnings: Vec::new(),
        }];

        validate_all_evidence(dir.path(), &mut candidates);
        assert!(!candidates[0].evidence[0].evidence_valid);
    }

    #[test]
    fn test_triage_confirm() {
        let scan = ScanOutput {
            candidates: vec![Candidate {
                id: "DC-001".to_string(),
                signal_type: "boundary".to_string(),
                title: "Test decision".to_string(),
                observation: "Observed pattern".to_string(),
                evidence: vec![],
                hypothesised_consequence: "Bad things".to_string(),
                confidence: "high".to_string(),
                warnings: Vec::new(),
            }],
            scan_metadata: ScanMetadata {
                files_scanned: 1,
                prompt_version: "test".to_string(),
            },
        };

        let mut input = Cursor::new("c\n");
        let result = triage_interactive(&scan, &mut input).expect("triage");
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].triage_status, TriageStatus::Confirmed);
    }

    #[test]
    fn test_triage_reject() {
        let scan = ScanOutput {
            candidates: vec![Candidate {
                id: "DC-001".to_string(),
                signal_type: "boundary".to_string(),
                title: "Test decision".to_string(),
                observation: "Observed pattern".to_string(),
                evidence: vec![],
                hypothesised_consequence: "Bad things".to_string(),
                confidence: "high".to_string(),
                warnings: Vec::new(),
            }],
            scan_metadata: ScanMetadata {
                files_scanned: 1,
                prompt_version: "test".to_string(),
            },
        };

        let mut input = Cursor::new("r\n");
        let result = triage_interactive(&scan, &mut input).expect("triage");
        assert_eq!(result.candidates[0].triage_status, TriageStatus::Rejected);
    }

    #[test]
    fn test_batch_confirm() {
        let scan = ScanOutput {
            candidates: vec![
                Candidate {
                    id: "DC-001".to_string(),
                    signal_type: "boundary".to_string(),
                    title: "Test 1".to_string(),
                    observation: "Obs 1".to_string(),
                    evidence: vec![],
                    hypothesised_consequence: "Bad 1".to_string(),
                    confidence: "high".to_string(),
                    warnings: Vec::new(),
                },
                Candidate {
                    id: "DC-002".to_string(),
                    signal_type: "consistency".to_string(),
                    title: "Test 2".to_string(),
                    observation: "Obs 2".to_string(),
                    evidence: vec![],
                    hypothesised_consequence: "Bad 2".to_string(),
                    confidence: "medium".to_string(),
                    warnings: Vec::new(),
                },
            ],
            scan_metadata: ScanMetadata {
                files_scanned: 2,
                prompt_version: "test".to_string(),
            },
        };

        let result = triage_batch_confirm(&scan);
        assert_eq!(result.candidates.len(), 2);
        assert!(result
            .candidates
            .iter()
            .all(|c| c.triage_status == TriageStatus::Confirmed));
    }

    #[test]
    fn test_plan_seed_ids() {
        let triage = TriageOutput {
            candidates: vec![TriagedCandidate {
                candidate: Candidate {
                    id: "DC-001".to_string(),
                    signal_type: "boundary".to_string(),
                    title: "Test decision".to_string(),
                    observation: "Observation".to_string(),
                    evidence: vec![Evidence {
                        file: "src/test.rs".to_string(),
                        line: 1,
                        snippet: "test".to_string(),
                        evidence_valid: true,
                    }],
                    hypothesised_consequence: "Bad".to_string(),
                    confidence: "high".to_string(),
                    warnings: Vec::new(),
                },
                triage_status: TriageStatus::Confirmed,
                merged_into: None,
            }],
        };

        let result = plan_seed(
            &triage,
            &["ADR-001".to_string(), "ADR-002".to_string()],
            &["FT-001".to_string()],
            "ADR",
            "FT",
        );

        assert_eq!(result.adrs.len(), 1);
        assert_eq!(result.adrs[0].id, "ADR-003");
        assert_eq!(result.features.len(), 1);
        assert_eq!(result.features[0].id, "FT-002");
    }

    #[test]
    fn test_group_into_features_by_directory() {
        let adrs = vec![
            ProposedAdr {
                id: "ADR-001".to_string(),
                title: "API decision".to_string(),
                observation: "obs".to_string(),
                evidence: vec![Evidence {
                    file: "src/api/handler.rs".to_string(),
                    line: 1,
                    snippet: "test".to_string(),
                    evidence_valid: true,
                }],
                hypothesised_consequence: "bad".to_string(),
                filename: "ADR-001-api.md".to_string(),
            },
            ProposedAdr {
                id: "ADR-002".to_string(),
                title: "Storage decision".to_string(),
                observation: "obs".to_string(),
                evidence: vec![Evidence {
                    file: "src/storage/db.rs".to_string(),
                    line: 1,
                    snippet: "test".to_string(),
                    evidence_valid: true,
                }],
                hypothesised_consequence: "bad".to_string(),
                filename: "ADR-002-storage.md".to_string(),
            },
        ];

        let features = group_into_features(&adrs, &[], "FT");
        assert_eq!(features.len(), 2);
        assert!(features[0].adr_ids.contains(&"ADR-001".to_string()));
        assert!(features[1].adr_ids.contains(&"ADR-002".to_string()));
    }
}
