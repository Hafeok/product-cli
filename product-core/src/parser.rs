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

/// Result of loading all artifacts: features, ADRs, tests, dependencies,
/// patterns, and any parse errors.
pub struct LoadResult {
    pub features: Vec<Feature>,
    pub adrs: Vec<Adr>,
    pub tests: Vec<TestCriterion>,
    pub dependencies: Vec<Dependency>,
    pub patterns: Vec<Pattern>,
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
    load_all_full(features_dir, adrs_dir, tests_dir, deps_dir, None)
}

/// Load every artifact type — features, ADRs, TCs, deps, and patterns
/// (FT-070). Patterns are loaded only when `patterns_dir` is supplied.
pub fn load_all_full(
    features_dir: &Path,
    adrs_dir: &Path,
    tests_dir: &Path,
    deps_dir: Option<&Path>,
    patterns_dir: Option<&Path>,
) -> Result<LoadResult> {
    let (features, mut errs_f) = load_dir(features_dir, parse_feature)?;
    let (adrs, mut errs_a) = load_dir(adrs_dir, parse_adr)?;
    let (tests, errs_t) = load_dir(tests_dir, parse_test)?;
    let (dependencies, errs_d) = if let Some(d) = deps_dir {
        load_dir(d, parse_dependency)?
    } else {
        (Vec::new(), Vec::new())
    };
    let (patterns, errs_p) = if let Some(p) = patterns_dir {
        load_dir(p, parse_pattern)?
    } else {
        (Vec::new(), Vec::new())
    };
    errs_f.append(&mut errs_a);
    errs_f.extend(errs_t);
    errs_f.extend(errs_d);
    errs_f.extend(errs_p);
    Ok(LoadResult {
        features,
        adrs,
        tests,
        dependencies,
        patterns,
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

/// Serialize pattern front-matter + body to a markdown file string (FT-070).
pub fn render_pattern(front: &PatternFrontMatter, body: &str) -> String {
    let yaml = serde_yaml::to_string(front).unwrap_or_default();
    format!("---\n{}---\n\n{}", yaml, body)
}

/// Parse a pattern file (FT-070, ADR-050).
pub fn parse_pattern(path: &Path) -> Result<Pattern> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ProductError::IoError(format!("{}: {}", path.display(), e)))?;
    let (yaml, body) = split_front_matter(&content).ok_or_else(|| ProductError::ParseError {
        file: path.to_path_buf(),
        line: Some(1),
        message: "no YAML front-matter found".to_string(),
    })?;
    let front: PatternFrontMatter =
        serde_yaml::from_str(yaml).map_err(|e| ProductError::ParseError {
            file: path.to_path_buf(),
            line: e.location().map(|l| l.line()),
            message: format!("YAML parse error: {}", e),
        })?;
    if front.id.is_empty() {
        return Err(ProductError::MissingField {
            file: path.to_path_buf(),
            field: "id".to_string(),
        });
    }
    validate_id(&front.id, path)?;
    Ok(Pattern {
        front,
        body: body.to_string(),
        path: path.to_path_buf(),
    })
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
#[path = "parser_tests.rs"]
mod tests;
