//! Migration extraction — parse monolithic PRD/ADR documents (ADR-017)

use crate::error::Result;
use crate::parser;
use crate::types::*;
use regex::Regex;
use std::path::Path;

use super::helpers::*;
use super::types::*;

/// Non-feature heading patterns to skip during PRD migration
const EXCLUDED_HEADINGS: &[&str] = &[
    "vision", "goals", "non-goals", "target environment", "core architecture",
    "open questions", "resolved decisions", "phase plan", "overview",
    "introduction", "background", "references", "non goals",
];

/// Parse a monolithic PRD document into proposed features
pub fn migrate_from_prd(
    source: &Path,
    features_dir: &Path,
    prefix: &str,
) -> Result<MigrationPlan> {
    let content = std::fs::read_to_string(source)?;
    let mut plan = MigrationPlan {
        features: Vec::new(),
        adrs: Vec::new(),
        tests: Vec::new(),
        warnings: Vec::new(),
        conflicts: Vec::new(),
    };

    let mut current_phase: u32 = 1;
    let mut feature_counter: u32 = 0;

    // Find existing feature IDs for conflict detection
    let existing_ids = find_existing_ids(features_dir);

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Detect phase headings
        if let Some(phase) = detect_phase_heading(line) {
            current_phase = phase;
            i += 1;
            continue;
        }

        // Detect H2 headings that might be features
        if line.starts_with("## ") && !line.starts_with("### ") {
            let heading = line.trim_start_matches('#').trim();
            // Strip leading numbers: "5. Products and IAM" -> "Products and IAM"
            let title = strip_leading_number(heading);

            if is_excluded_heading(&title) {
                i += 1;
                continue;
            }

            // Collect body until next H2
            let mut body = String::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("## ") {
                body.push_str(lines[i]);
                body.push('\n');
                i += 1;
            }

            // Check for checklist items in body
            let status = infer_status_from_body(&body);

            feature_counter += 1;
            let id = format!("{}-{:03}", prefix, feature_counter);
            let filename = parser::id_to_filename(&id, &title);

            // Conflict check
            let full_path = features_dir.join(&filename);
            if full_path.exists() || existing_ids.contains(&id) {
                plan.conflicts.push(format!(
                    "{} already exists — will skip (use --overwrite to replace)",
                    filename
                ));
            }

            plan.features.push(ProposedFeature {
                id,
                title: title.to_string(),
                phase: current_phase,
                status,
                body: body.trim().to_string(),
                filename,
            });
            continue;
        }

        i += 1;
    }

    Ok(plan)
}

/// Parse a monolithic ADR document into proposed ADRs with test criteria
pub fn migrate_from_adrs(
    source: &Path,
    adrs_dir: &Path,
    tests_dir: &Path,
    adr_prefix: &str,
    test_prefix: &str,
) -> Result<MigrationPlan> {
    let content = std::fs::read_to_string(source)?;
    let mut plan = MigrationPlan {
        features: Vec::new(),
        adrs: Vec::new(),
        tests: Vec::new(),
        warnings: Vec::new(),
        conflicts: Vec::new(),
    };

    let existing_adr_ids = find_existing_ids(adrs_dir);
    let existing_test_ids = find_existing_ids(tests_dir);
    let mut test_counter: u32 = 0;

    // Find max existing test ID
    for id in &existing_test_ids {
        if let Some(num) = id.strip_prefix(test_prefix).and_then(|s| s.strip_prefix('-')).and_then(|s| s.parse::<u32>().ok()) {
            if num > test_counter {
                test_counter = num;
            }
        }
    }

    let adr_heading_re = Regex::new(r"^##\s+(?:ADR-(\d+))\s*[:\-—]\s*(.+)").expect("constant regex");

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if let Some(caps) = adr_heading_re.captures(lines[i]) {
            let adr_num: u32 = caps[1].parse().unwrap_or(0);
            let title = caps[2].trim().to_string();
            let id = format!("{}-{:03}", adr_prefix, adr_num);

            // Collect body until next H2 ADR heading
            let mut body = String::new();
            i += 1;
            while i < lines.len() && !adr_heading_re.is_match(lines[i]) {
                body.push_str(lines[i]);
                body.push('\n');
                i += 1;
            }

            // Extract status from body
            let status = extract_adr_status(&body);
            if status.is_none() {
                plan.warnings.push(format!(
                    "[W008] {}: status not found, defaulting to \"proposed\"",
                    id
                ));
            }

            // Extract test criteria from body
            let test_section = extract_test_section(&body);
            if test_section.is_none() {
                plan.warnings.push(format!(
                    "[W009] {}: no test subsection found — no test criteria extracted",
                    id
                ));
            }

            if let Some(tests_text) = test_section {
                let extracted = extract_test_items(&tests_text, &id);
                for (test_title, test_type, test_body) in extracted {
                    test_counter += 1;
                    let test_id = format!("{}-{:03}", test_prefix, test_counter);
                    let filename = parser::id_to_filename(&test_id, &test_title);

                    let full_path = tests_dir.join(&filename);
                    if full_path.exists() || existing_test_ids.contains(&test_id) {
                        plan.conflicts.push(format!("{} already exists — will skip", filename));
                    }

                    plan.tests.push(ProposedTest {
                        id: test_id,
                        title: test_title,
                        test_type,
                        adr_id: id.clone(),
                        body: test_body,
                        filename,
                    });
                }
            }

            let filename = parser::id_to_filename(&id, &title);
            let full_path = adrs_dir.join(&filename);
            if full_path.exists() || existing_adr_ids.contains(&id) {
                plan.conflicts.push(format!("{} already exists — will skip", filename));
            }

            // Strip test section from ADR body for the file
            let clean_body = remove_test_section(&body);

            plan.adrs.push(ProposedAdr {
                id,
                title,
                status: status.unwrap_or(AdrStatus::Proposed),
                body: clean_body.trim().to_string(),
                filename,
            });
            continue;
        }
        i += 1;
    }

    Ok(plan)
}

pub(crate) fn is_excluded_heading(title: &str) -> bool {
    let lower = title.to_lowercase();
    EXCLUDED_HEADINGS.iter().any(|&h| lower.starts_with(h))
}
