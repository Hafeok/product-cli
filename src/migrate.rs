//! Migration commands — parse monolithic PRD/ADR files into structured artifacts (ADR-017)

use crate::error::Result;
use crate::parser;
use crate::types::*;
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub struct MigrationPlan {
    pub features: Vec<ProposedFeature>,
    pub adrs: Vec<ProposedAdr>,
    pub tests: Vec<ProposedTest>,
    pub warnings: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug)]
pub struct ProposedFeature {
    pub id: String,
    pub title: String,
    pub phase: u32,
    pub status: FeatureStatus,
    pub body: String,
    pub filename: String,
}

#[derive(Debug)]
pub struct ProposedAdr {
    pub id: String,
    pub title: String,
    pub status: AdrStatus,
    pub body: String,
    pub filename: String,
}

#[derive(Debug)]
pub struct ProposedTest {
    pub id: String,
    pub title: String,
    pub test_type: TestType,
    pub adr_id: String,
    pub body: String,
    pub filename: String,
}

impl MigrationPlan {
    pub fn print_summary(&self) {
        println!(
            "Migration plan: {} features, {} ADRs, {} test criteria",
            self.features.len(),
            self.adrs.len(),
            self.tests.len()
        );
        println!();

        if !self.features.is_empty() {
            println!("Feature files to create:");
            for f in &self.features {
                println!("  {} (phase: {}, status: {})", f.filename, f.phase, f.status);
            }
            println!();
        }

        if !self.adrs.is_empty() {
            println!("ADR files to create:");
            for a in &self.adrs {
                println!("  {} (status: {})", a.filename, a.status);
            }
            println!();
        }

        if !self.tests.is_empty() {
            println!("Test criteria files to create:");
            for t in &self.tests {
                println!("  {} (type: {}, adr: {})", t.filename, t.test_type, t.adr_id);
            }
            println!();
        }

        if !self.warnings.is_empty() {
            println!("Warnings:");
            for w in &self.warnings {
                println!("  {}", w);
            }
            println!();
        }

        if !self.conflicts.is_empty() {
            println!("Conflicts:");
            for c in &self.conflicts {
                println!("  {}", c);
            }
            println!();
        }
    }
}

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

/// Parse a monolithic ADR document into proposed ADRs and test criteria
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

/// Execute a migration plan: write files
/// If `interactive` is true, prompt for each artifact before writing.
pub fn execute_plan(
    plan: &MigrationPlan,
    features_dir: &Path,
    adrs_dir: &Path,
    tests_dir: &Path,
    overwrite: bool,
    interactive: bool,
) -> Result<(usize, usize)> {
    let mut written = 0;
    let mut skipped = 0;

    for f in &plan.features {
        let path = features_dir.join(&f.filename);
        if path.exists() && !overwrite {
            skipped += 1;
            println!("  skip: {} (exists)", f.filename);
            continue;
        }
        if interactive {
            println!("\n--- Feature: {} — {} (phase {}) ---", f.id, f.title, f.phase);
            let preview = if f.body.len() > 200 { &f.body[..200] } else { &f.body };
            println!("{}", preview);
            match prompt_interactive()? {
                InteractiveChoice::Accept => {}
                InteractiveChoice::Skip => { skipped += 1; continue; }
                InteractiveChoice::Quit => return Ok((written, skipped)),
            }
        }
        let front = FeatureFrontMatter {
            id: f.id.clone(),
            title: f.title.clone(),
            phase: f.phase,
            status: f.status,
            depends_on: vec![],
            adrs: vec![],
            tests: vec![],
            domains: vec![],
            domains_acknowledged: std::collections::HashMap::new(),
        };
        let content = crate::parser::render_feature(&front, &f.body);
        crate::fileops::write_file_atomic(&path, &content)?;
        written += 1;
        println!("  wrote: {}", f.filename);
    }

    for a in &plan.adrs {
        let path = adrs_dir.join(&a.filename);
        if path.exists() && !overwrite {
            skipped += 1;
            println!("  skip: {} (exists)", a.filename);
            continue;
        }
        if interactive {
            println!("\n--- ADR: {} — {} ({}) ---", a.id, a.title, a.status);
            let preview = if a.body.len() > 200 { &a.body[..200] } else { &a.body };
            println!("{}", preview);
            match prompt_interactive()? {
                InteractiveChoice::Accept => {}
                InteractiveChoice::Skip => { skipped += 1; continue; }
                InteractiveChoice::Quit => return Ok((written, skipped)),
            }
        }
        let front = AdrFrontMatter {
            id: a.id.clone(),
            title: a.title.clone(),
            status: a.status,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope: crate::types::AdrScope::Domain,
        };
        let content = crate::parser::render_adr(&front, &a.body);
        crate::fileops::write_file_atomic(&path, &content)?;
        written += 1;
        println!("  wrote: {}", a.filename);
    }

    for t in &plan.tests {
        let path = tests_dir.join(&t.filename);
        if path.exists() && !overwrite {
            skipped += 1;
            println!("  skip: {} (exists)", t.filename);
            continue;
        }
        if interactive {
            println!("\n--- Test: {} — {} ({}, adr: {}) ---", t.id, t.title, t.test_type, t.adr_id);
            let preview = if t.body.len() > 200 { &t.body[..200] } else { &t.body };
            println!("{}", preview);
            match prompt_interactive()? {
                InteractiveChoice::Accept => {}
                InteractiveChoice::Skip => { skipped += 1; continue; }
                InteractiveChoice::Quit => return Ok((written, skipped)),
            }
        }
        let front = TestFrontMatter {
            id: t.id.clone(),
            title: t.title.clone(),
            test_type: t.test_type,
            status: TestStatus::Unimplemented,
            validates: ValidatesBlock {
                features: vec![],
                adrs: vec![t.adr_id.clone()],
            },
            phase: 1,
        };
        let content = crate::parser::render_test(&front, &t.body);
        crate::fileops::write_file_atomic(&path, &content)?;
        written += 1;
        println!("  wrote: {}", t.filename);
    }

    Ok((written, skipped))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn find_existing_ids(dir: &Path) -> Vec<String> {
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

fn detect_phase_heading(line: &str) -> Option<u32> {
    let re = Regex::new(r"^###?\s+Phase\s+(\d+)").expect("constant regex");
    re.captures(line).and_then(|c| c[1].parse().ok())
}

fn strip_leading_number(heading: &str) -> String {
    let re = Regex::new(r"^\d+[\.\)]\s*").expect("constant regex");
    re.replace(heading, "").to_string()
}

fn is_excluded_heading(title: &str) -> bool {
    let lower = title.to_lowercase();
    EXCLUDED_HEADINGS.iter().any(|&h| lower.starts_with(h))
}

fn infer_status_from_body(body: &str) -> FeatureStatus {
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

fn extract_adr_status(body: &str) -> Option<AdrStatus> {
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

fn extract_test_section(body: &str) -> Option<String> {
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

fn remove_test_section(body: &str) -> String {
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

fn extract_test_items(section: &str, _adr_id: &str) -> Vec<(String, TestType, String)> {
    let mut items = Vec::new();
    let lines: Vec<&str> = section.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Match bullet points that look like test items
        let is_bullet = line.starts_with("- `") || line.starts_with("- **");
        let is_subheading = line.starts_with("#### ");

        if is_bullet || is_subheading {
            let title = if is_subheading {
                line.trim_start_matches('#').trim().to_string()
            } else {
                // Extract title from bullet: "- `test_name.rs` — description"
                let cleaned = line.trim_start_matches("- ");
                let cleaned = cleaned.trim_start_matches('`');
                // Take until the first ` or —
                let end = cleaned.find('`')
                    .or_else(|| cleaned.find('—'))
                    .or_else(|| cleaned.find(" — "))
                    .unwrap_or(cleaned.len());
                cleaned[..end].trim().trim_end_matches(".rs").to_string()
            };

            if title.is_empty() {
                i += 1;
                continue;
            }

            // Infer type from title keywords
            let lower_title = title.to_lowercase();
            let test_type = if lower_title.contains("chaos") {
                TestType::Chaos
            } else if lower_title.contains("invariant") {
                TestType::Invariant
            } else if lower_title.contains("exit") {
                TestType::ExitCriteria
            } else {
                TestType::Scenario
            };

            // Collect description
            let desc = if is_bullet {
                let after_title = line
                    .find('—')
                    .map(|p| &line[p + '—'.len_utf8()..])
                    .unwrap_or("")
                    .trim();
                after_title.to_string()
            } else {
                // Collect until next bullet or subheading
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
                desc.trim().to_string()
            };

            items.push((title, test_type, desc));
            if !is_subheading {
                i += 1;
            }
            continue;
        }
        i += 1;
    }

    items
}

// ---------------------------------------------------------------------------
// Interactive migration prompt (ADR-017)
// ---------------------------------------------------------------------------

enum InteractiveChoice {
    Accept,
    Skip,
    Quit,
}

fn prompt_interactive() -> Result<InteractiveChoice> {
    use std::io::{self, BufRead, Write};

    loop {
        print!("[a]ccept / [s]kip / [q]uit: ");
        io::stdout().flush().map_err(|e| crate::error::ProductError::IoError(e.to_string()))?;

        let mut input = String::new();
        io::stdin()
            .lock()
            .read_line(&mut input)
            .map_err(|e| crate::error::ProductError::IoError(e.to_string()))?;

        match input.trim().to_lowercase().as_str() {
            "a" | "accept" => return Ok(InteractiveChoice::Accept),
            "s" | "skip" => return Ok(InteractiveChoice::Skip),
            "q" | "quit" => return Ok(InteractiveChoice::Quit),
            _ => println!("  Invalid choice. Enter a, s, or q."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_leading_number_works() {
        assert_eq!(strip_leading_number("5. Products and IAM"), "Products and IAM");
        assert_eq!(strip_leading_number("12) Storage"), "Storage");
        assert_eq!(strip_leading_number("No Number"), "No Number");
    }

    #[test]
    fn excluded_headings_detected() {
        assert!(is_excluded_heading("Vision"));
        assert!(is_excluded_heading("Non-Goals"));
        assert!(is_excluded_heading("Core Architecture"));
        assert!(!is_excluded_heading("Cluster Foundation"));
    }

    #[test]
    fn detect_phase() {
        assert_eq!(detect_phase_heading("### Phase 1 — MVP"), Some(1));
        assert_eq!(detect_phase_heading("## Phase 3 — RDF"), Some(3));
        assert_eq!(detect_phase_heading("Some other line"), None);
    }

    #[test]
    fn infer_status() {
        assert_eq!(
            infer_status_from_body("- [x] done\n- [x] also done\n"),
            FeatureStatus::Complete
        );
        assert_eq!(
            infer_status_from_body("- [x] done\n- [ ] not done\n"),
            FeatureStatus::InProgress
        );
        assert_eq!(
            infer_status_from_body("no checklist here"),
            FeatureStatus::Planned
        );
    }

    #[test]
    fn extract_adr_status_works() {
        assert_eq!(
            extract_adr_status("**Status:** Accepted\n"),
            Some(AdrStatus::Accepted)
        );
        assert_eq!(
            extract_adr_status("**Status:** Proposed\n"),
            Some(AdrStatus::Proposed)
        );
        assert_eq!(extract_adr_status("no status\n"), None);
    }

    #[test]
    fn migrate_prd_detects_features() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("test-prd.md");
        std::fs::write(&source, "# PRD\n\n## Vision\n\nHello.\n\n## Resource Model\n\nStuff.\n\n## Storage Model\n\nMore stuff.\n").unwrap();
        let features_dir = dir.path().join("features");
        let plan = migrate_from_prd(&source, &features_dir, "FT").unwrap();
        assert_eq!(plan.features.len(), 2, "should detect 2 features (Vision excluded)");
        assert_eq!(plan.features[0].title, "Resource Model");
        assert_eq!(plan.features[1].title, "Storage Model");
    }

    #[test]
    fn migrate_adrs_extracts_tests() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("test-adrs.md");
        std::fs::write(&source, r#"# ADRs

## ADR-001: Rust Language

**Status:** Accepted

Some context.

**Test coverage:**

Scenario tests:
- `binary_compiles.rs` — compiles on ARM64
- `binary_no_deps.rs` — no dynamic deps

Exit criteria:
- Binary size < 20 MB.

---

## ADR-002: YAML Front-Matter

**Status:** Accepted

More context.
"#).unwrap();
        let adrs_dir = dir.path().join("adrs");
        let tests_dir = dir.path().join("tests");
        let plan = migrate_from_adrs(&source, &adrs_dir, &tests_dir, "ADR", "TC").unwrap();
        assert_eq!(plan.adrs.len(), 2, "should extract 2 ADRs");
        assert!(plan.tests.len() >= 2, "should extract test criteria from ADR-001");
        assert_eq!(plan.adrs[0].status, AdrStatus::Accepted);
    }

    #[test]
    fn migrate_validate_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("test.md");
        std::fs::write(&source, "# PRD\n\n## Feature One\n\nContent.\n").unwrap();
        let features_dir = dir.path().join("features");
        let plan = migrate_from_prd(&source, &features_dir, "FT").unwrap();
        // Don't call execute_plan — just verify plan exists and no files were created
        assert_eq!(plan.features.len(), 1);
        assert!(!features_dir.exists(), "features dir should not exist (validate only)");
    }

    #[test]
    fn migrate_execute_creates_files() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("test.md");
        std::fs::write(&source, "# PRD\n\n## Feature One\n\nContent.\n").unwrap();
        let features_dir = dir.path().join("features");
        let adrs_dir = dir.path().join("adrs");
        let tests_dir = dir.path().join("tests");
        let plan = migrate_from_prd(&source, &features_dir, "FT").unwrap();
        std::fs::create_dir_all(&features_dir).unwrap();
        let (written, _skipped) = super::execute_plan(&plan, &features_dir, &adrs_dir, &tests_dir, false, false).unwrap();
        assert_eq!(written, 1);
        assert!(features_dir.read_dir().unwrap().count() > 0, "should have created files");
    }
}
