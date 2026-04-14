//! Front-matter parser — reads YAML front-matter from markdown files (ADR-002)

use crate::error::{ProductError, Result};
use crate::formal;
use crate::types::*;
use regex::Regex;
use std::path::Path;

/// Validate an artifact ID matches the PREFIX-NNN format
pub fn validate_id(id: &str, path: &Path) -> Result<()> {
    let re = Regex::new(r"^[A-Z]+-\d{3,}$").expect("constant regex");
    if !re.is_match(id) {
        return Err(ProductError::InvalidId {
            file: path.to_path_buf(),
            id: id.to_string(),
        });
    }
    Ok(())
}

/// Split a markdown file into YAML front-matter and body
fn split_front_matter(content: &str) -> Option<(&str, &str)> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let yaml = &rest[..end];
    let body_start = end + 4; // skip \n---
    let body = if body_start < rest.len() {
        // skip the newline after closing ---
        rest[body_start..].trim_start_matches('\n')
    } else {
        ""
    };
    Some((yaml, body))
}

/// Parse a feature file
pub fn parse_feature(path: &Path) -> Result<Feature> {
    let content = std::fs::read_to_string(path).map_err(|e| ProductError::IoError(format!("{}: {}", path.display(), e)))?;
    let (yaml, body) = split_front_matter(&content).ok_or_else(|| {
        ProductError::ParseError {
            file: path.to_path_buf(),
            line: Some(1),
            message: "no YAML front-matter found (expected --- delimiters)".to_string(),
        }
    })?;
    let front: FeatureFrontMatter = serde_yaml::from_str(yaml).map_err(|e| {
        ProductError::ParseError {
            file: path.to_path_buf(),
            line: e.location().map(|l| l.line()),
            message: format!("YAML parse error: {}", e),
        }
    })?;
    if front.id.is_empty() {
        return Err(ProductError::MissingField {
            file: path.to_path_buf(),
            field: "id".to_string(),
        });
    }
    validate_id(&front.id, path)?;
    Ok(Feature {
        front,
        body: body.to_string(),
        path: path.to_path_buf(),
    })
}

/// Parse an ADR file
pub fn parse_adr(path: &Path) -> Result<Adr> {
    let content = std::fs::read_to_string(path).map_err(|e| ProductError::IoError(format!("{}: {}", path.display(), e)))?;
    let (yaml, body) = split_front_matter(&content).ok_or_else(|| {
        ProductError::ParseError {
            file: path.to_path_buf(),
            line: Some(1),
            message: "no YAML front-matter found".to_string(),
        }
    })?;
    let front: AdrFrontMatter = serde_yaml::from_str(yaml).map_err(|e| {
        ProductError::ParseError {
            file: path.to_path_buf(),
            line: e.location().map(|l| l.line()),
            message: format!("YAML parse error: {}", e),
        }
    })?;
    if front.id.is_empty() {
        return Err(ProductError::MissingField {
            file: path.to_path_buf(),
            field: "id".to_string(),
        });
    }
    Ok(Adr {
        front,
        body: body.to_string(),
        path: path.to_path_buf(),
    })
}

/// Parse a test criterion file
pub fn parse_test(path: &Path) -> Result<TestCriterion> {
    let content = std::fs::read_to_string(path).map_err(|e| ProductError::IoError(format!("{}: {}", path.display(), e)))?;
    let (yaml, body) = split_front_matter(&content).ok_or_else(|| {
        ProductError::ParseError {
            file: path.to_path_buf(),
            line: Some(1),
            message: "no YAML front-matter found".to_string(),
        }
    })?;
    let front: TestFrontMatter = serde_yaml::from_str(yaml).map_err(|e| {
        ProductError::ParseError {
            file: path.to_path_buf(),
            line: e.location().map(|l| l.line()),
            message: format!("YAML parse error: {}", e),
        }
    })?;
    if front.id.is_empty() {
        return Err(ProductError::MissingField {
            file: path.to_path_buf(),
            field: "id".to_string(),
        });
    }

    let formal_blocks = formal::parse_formal_blocks(body);

    Ok(TestCriterion {
        front,
        body: body.to_string(),
        path: path.to_path_buf(),
        formal_blocks,
    })
}

/// Parse a dependency file (ADR-030)
pub fn parse_dependency(path: &Path) -> Result<Dependency> {
    let content = std::fs::read_to_string(path).map_err(|e| ProductError::IoError(format!("{}: {}", path.display(), e)))?;
    let (yaml, body) = split_front_matter(&content).ok_or_else(|| {
        ProductError::ParseError {
            file: path.to_path_buf(),
            line: Some(1),
            message: "no YAML front-matter found".to_string(),
        }
    })?;
    let front: DependencyFrontMatter = serde_yaml::from_str(yaml).map_err(|e| {
        ProductError::ParseError {
            file: path.to_path_buf(),
            line: e.location().map(|l| l.line()),
            message: format!("YAML parse error: {}", e),
        }
    })?;
    if front.id.is_empty() {
        return Err(ProductError::MissingField {
            file: path.to_path_buf(),
            field: "id".to_string(),
        });
    }
    validate_id(&front.id, path)?;
    Ok(Dependency {
        front,
        body: body.to_string(),
        path: path.to_path_buf(),
    })
}

/// Result of loading all artifacts: features, ADRs, tests, dependencies, and any parse errors.
pub struct LoadResult {
    pub features: Vec<Feature>,
    pub adrs: Vec<Adr>,
    pub tests: Vec<TestCriterion>,
    pub dependencies: Vec<Dependency>,
    pub parse_errors: Vec<ProductError>,
}

/// Load all artifacts from the configured directories.
/// Returns a `LoadResult` — parse errors are collected rather than printed,
/// so the caller can decide how to present them (ADR-013).
pub fn load_all(
    features_dir: &Path,
    adrs_dir: &Path,
    tests_dir: &Path,
) -> Result<LoadResult> {
    load_all_with_deps(features_dir, adrs_dir, tests_dir, None)
}

/// Load all artifacts including dependencies from an optional deps directory.
pub fn load_all_with_deps(
    features_dir: &Path,
    adrs_dir: &Path,
    tests_dir: &Path,
    deps_dir: Option<&Path>,
) -> Result<LoadResult> {
    let (features, mut errs_f) = load_dir(features_dir, parse_feature)?;
    let (adrs, mut errs_a) = load_dir(adrs_dir, parse_adr)?;
    let (tests, errs_t) = load_dir(tests_dir, parse_test)?;
    let (dependencies, errs_d) = if let Some(d) = deps_dir {
        load_dir(d, parse_dependency)?
    } else {
        (Vec::new(), Vec::new())
    };
    errs_f.append(&mut errs_a);
    errs_f.extend(errs_t);
    errs_f.extend(errs_d);
    Ok(LoadResult {
        features,
        adrs,
        tests,
        dependencies,
        parse_errors: errs_f,
    })
}

fn load_dir<T>(dir: &Path, parser: fn(&Path) -> Result<T>) -> Result<(Vec<T>, Vec<ProductError>)> {
    if !dir.exists() {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut items = Vec::new();
    let mut errors = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| ProductError::IoError(format!("{}: {}", dir.display(), e)))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        match parser(&entry.path()) {
            Ok(item) => items.push(item),
            Err(e) => {
                errors.push(e);
            }
        }
    }
    Ok((items, errors))
}

/// Serialize front-matter + body back to a markdown file string
pub fn render_feature(front: &FeatureFrontMatter, body: &str) -> String {
    let yaml = serde_yaml::to_string(front).unwrap_or_default();
    format!("---\n{}---\n\n{}", yaml, body)
}

pub fn render_adr(front: &AdrFrontMatter, body: &str) -> String {
    let yaml = serde_yaml::to_string(front).unwrap_or_default();
    format!("---\n{}---\n\n{}", yaml, body)
}

pub fn render_test(front: &TestFrontMatter, body: &str) -> String {
    let yaml = serde_yaml::to_string(front).unwrap_or_default();
    format!("---\n{}---\n\n{}", yaml, body)
}

pub fn render_dependency(front: &DependencyFrontMatter, body: &str) -> String {
    let yaml = serde_yaml::to_string(front).unwrap_or_default();
    format!("---\n{}---\n\n{}", yaml, body)
}

/// Extract the next sequential ID from a list of existing IDs
pub fn next_id(prefix: &str, existing: &[String]) -> String {
    let max_num = existing
        .iter()
        .filter_map(|id| {
            id.strip_prefix(prefix)
                .and_then(|rest| rest.strip_prefix('-'))
                .and_then(|num| num.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    format!("{}-{:03}", prefix, max_num + 1)
}

/// Generate a filename from an ID and title
pub fn id_to_filename(id: &str, title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let slug = if slug.len() > 50 { &slug[..50] } else { &slug };
    let slug = slug.trim_end_matches('-');
    format!("{}-{}.md", id, slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_front_matter() {
        let content = "---\nid: FT-001\ntitle: Test\n---\n\nBody content here.";
        let (yaml, body) = split_front_matter(content).unwrap();
        assert!(yaml.contains("id: FT-001"));
        assert_eq!(body, "Body content here.");
    }

    #[test]
    fn test_split_no_front_matter() {
        let content = "No front matter here.";
        assert!(split_front_matter(content).is_none());
    }

    #[test]
    fn test_next_id() {
        let existing = vec!["FT-001".to_string(), "FT-003".to_string()];
        assert_eq!(next_id("FT", &existing), "FT-004");
    }

    #[test]
    fn test_next_id_empty() {
        let existing: Vec<String> = vec![];
        assert_eq!(next_id("ADR", &existing), "ADR-001");
    }

    #[test]
    fn test_id_to_filename() {
        assert_eq!(id_to_filename("FT-001", "Cluster Foundation"), "FT-001-cluster-foundation.md");
        assert_eq!(id_to_filename("ADR-002", "openraft for Consensus"), "ADR-002-openraft-for-consensus.md");
    }

    #[test]
    fn test_feature_parse_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("FT-001-test.md");
        let content = "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n";
        std::fs::write(&path, content).unwrap();
        let feature = parse_feature(&path).unwrap();
        assert_eq!(feature.front.id, "FT-001");
        assert_eq!(feature.front.title, "Test Feature");
        assert_eq!(feature.front.status, FeatureStatus::InProgress);
        assert_eq!(feature.front.adrs, vec!["ADR-001"]);
        assert_eq!(feature.body, "Feature body.\n");
    }

    #[test]
    fn test_adr_parse() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ADR-001-test.md");
        let content = "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nDecision body.\n";
        std::fs::write(&path, content).unwrap();
        let adr = parse_adr(&path).unwrap();
        assert_eq!(adr.front.id, "ADR-001");
        assert_eq!(adr.front.status, AdrStatus::Accepted);
    }

    #[test]
    fn test_test_parse() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("TC-001-test.md");
        let content = "---\nid: TC-001\ntitle: Test Criterion\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nDescription.\n";
        std::fs::write(&path, content).unwrap();
        let tc = parse_test(&path).unwrap();
        assert_eq!(tc.front.id, "TC-001");
        assert_eq!(tc.front.test_type, TestType::Scenario);
        assert_eq!(tc.front.validates.features, vec!["FT-001"]);
    }

    #[test]
    fn validate_id_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        assert!(validate_id("FT-001", &path).is_ok());
        assert!(validate_id("ADR-123", &path).is_ok());
        assert!(validate_id("TC-0001", &path).is_ok());
    }

    #[test]
    fn validate_id_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        assert!(validate_id("bad-id", &path).is_err());
        assert!(validate_id("FT001", &path).is_err());
        assert!(validate_id("FT-1", &path).is_err()); // needs 3+ digits
        assert!(validate_id("", &path).is_err());
    }
}
