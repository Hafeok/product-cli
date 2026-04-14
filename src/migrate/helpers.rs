//! Migration helper functions — parsing utilities (ADR-017)

use crate::types::*;
use regex::Regex;
use std::path::Path;

pub(crate) fn find_existing_ids(dir: &Path) -> Vec<String> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut ids = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Extract ID from filename pattern like "FT-001-some-title.md"
            if let Some(dash_pos) = name.find('-') {
                if let Some(second_dash) = name[dash_pos + 1..].find('-') {
                    let id = &name[..dash_pos + 1 + second_dash];
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids
}

pub(crate) fn detect_phase_heading(line: &str) -> Option<u32> {
    let re = Regex::new(r"^###?\s+Phase\s+(\d+)").expect("constant regex");
    re.captures(line).and_then(|c| c[1].parse().ok())
}

pub(crate) fn strip_leading_number(heading: &str) -> String {
    let re = Regex::new(r"^\d+[\.\)]\s*").expect("constant regex");
    re.replace(heading, "").to_string()
}

pub(crate) fn infer_status_from_body(body: &str) -> FeatureStatus {
    let checked = body.matches("- [x]").count();
    let unchecked = body.matches("- [ ]").count();
    if checked > 0 && unchecked == 0 {
        FeatureStatus::Complete
    } else if checked > 0 {
        FeatureStatus::InProgress
    } else {
        FeatureStatus::Planned
    }
}

pub(crate) fn extract_adr_status(body: &str) -> Option<AdrStatus> {
    for line in body.lines() {
        if line.contains("**Status:**") || line.contains("*Status:*") {
            let lower = line.to_lowercase();
            if lower.contains("accepted") {
                return Some(AdrStatus::Accepted);
            }
            if lower.contains("superseded") {
                return Some(AdrStatus::Superseded);
            }
            if lower.contains("proposed") {
                return Some(AdrStatus::Proposed);
            }
            if lower.contains("abandoned") {
                return Some(AdrStatus::Abandoned);
            }
        }
    }
    None
}

pub(crate) fn extract_test_section(body: &str) -> Option<String> {
    let patterns = [
        "### Test coverage",
        "### Tests",
        "### Test Coverage",
        "### Exit criteria",
        "### Exit Criteria",
        "**Test coverage:**",
    ];
    let lower = body.to_lowercase();
    for pattern in &patterns {
        let lower_pattern = pattern.to_lowercase();
        if let Some(pos) = lower.find(&lower_pattern) {
            let rest = &body[pos..];
            // Find the end: next H3/H2 or end of body
            let end = rest[3..]
                .find("\n## ")
                .or_else(|| rest[3..].find("\n### "))
                .map(|p| p + 3)
                .unwrap_or(rest.len());
            return Some(rest[..end].to_string());
        }
    }
    None
}

pub(crate) fn remove_test_section(body: &str) -> String {
    let patterns = [
        "### Test coverage",
        "### Tests",
        "### Test Coverage",
        "**Test coverage:**",
    ];
    let mut result = body.to_string();
    let lower = result.to_lowercase();
    for pattern in &patterns {
        let lower_pattern = pattern.to_lowercase();
        if let Some(pos) = lower.find(&lower_pattern) {
            let rest = &body[pos..];
            let end = rest[3..]
                .find("\n## ")
                .or_else(|| rest[3..].find("\n### "))
                .map(|p| p + 3)
                .unwrap_or(rest.len());
            result = format!("{}{}", &body[..pos], &body[pos + end..]);
            break;
        }
    }
    result
}

pub(crate) fn extract_test_items(section: &str, _adr_id: &str) -> Vec<(String, TestType, String)> {
    let mut items = Vec::new();
    let lines: Vec<&str> = section.lines().collect();
    let mut i = 0;

    let section_lower = section.to_lowercase();
    let is_exit_criteria_section = section_lower.starts_with("### exit criteria");

    while i < lines.len() {
        let line = lines[i].trim();
        let is_bullet = line.starts_with("- `") || line.starts_with("- **");
        let is_subheading = line.starts_with("#### ");

        if is_bullet || is_subheading {
            let title = extract_item_title(line, is_subheading);
            if title.is_empty() {
                i += 1;
                continue;
            }

            let test_type = infer_test_type(&title, is_exit_criteria_section);
            let (desc, new_i) = collect_item_description(line, is_bullet, &lines, i);
            i = new_i;
            items.push((title, test_type, desc));
            continue;
        }
        i += 1;
    }

    items
}

fn extract_item_title(line: &str, is_subheading: bool) -> String {
    if is_subheading {
        line.trim_start_matches('#').trim().to_string()
    } else {
        let cleaned = line.trim_start_matches("- ");
        let cleaned = cleaned.trim_start_matches('`');
        let end = cleaned.find('`')
            .or_else(|| cleaned.find('\u{2014}'))
            .or_else(|| cleaned.find(" \u{2014} "))
            .unwrap_or(cleaned.len());
        cleaned[..end].trim().trim_end_matches(".rs").to_string()
    }
}

fn infer_test_type(title: &str, is_exit_criteria_section: bool) -> TestType {
    let lower_title = title.to_lowercase();
    if lower_title.contains("chaos") {
        TestType::Chaos
    } else if lower_title.contains("invariant") {
        TestType::Invariant
    } else if lower_title.contains("exit") || is_exit_criteria_section {
        TestType::ExitCriteria
    } else {
        TestType::Scenario
    }
}

fn collect_item_description(line: &str, is_bullet: bool, lines: &[&str], mut i: usize) -> (String, usize) {
    if is_bullet {
        let after_title = line
            .find('\u{2014}')
            .map(|p| &line[p + '\u{2014}'.len_utf8()..])
            .unwrap_or("")
            .trim();
        (after_title.to_string(), i + 1)
    } else {
        let mut desc = String::new();
        i += 1;
        while i < lines.len() {
            let next = lines[i].trim();
            if next.starts_with("- ") || next.starts_with("#### ") || next.starts_with("### ") {
                break;
            }
            desc.push_str(next);
            desc.push('\n');
            i += 1;
        }
        (desc.trim().to_string(), i)
    }
}
