//! W019 validation — feature outside product responsibility (FT-039)

use crate::error::{Diagnostic, CheckResult};
use crate::graph::KnowledgeGraph;
use crate::types::FeatureStatus;

/// Validate that features are within the declared product responsibility.
/// W019 is only emitted when `responsibility` is `Some`.
/// Returns diagnostics appended to the provided `CheckResult`.
pub fn check_responsibility(
    graph: &KnowledgeGraph,
    responsibility: Option<&str>,
    result: &mut CheckResult,
) {
    let responsibility = match responsibility {
        Some(r) if !r.trim().is_empty() => r,
        _ => return, // W019 suppressed when responsibility absent
    };

    let keywords = extract_keywords(responsibility);
    if keywords.is_empty() {
        return;
    }

    for feature in graph.features.values() {
        if feature.front.status == FeatureStatus::Abandoned {
            continue;
        }

        let feature_words = extract_keywords_from_feature(
            &feature.front.title,
            &feature.body,
        );

        // Check if the feature has ANY overlap with the responsibility keywords
        let overlap = feature_words.iter().any(|w| keywords.contains(w));

        // Also check if the feature is clearly infrastructure/tooling
        let is_infra = is_infrastructure_feature(&feature.front.title, &feature.body);

        if !overlap && !is_infra {
            result.warnings.push(
                Diagnostic::warning("W019", "feature outside product responsibility")
                    .with_file(feature.path.clone())
                    .with_detail(&format!(
                        "{} — \"{}\" does not appear related to the declared product responsibility",
                        feature.front.id, feature.front.title
                    ))
                    .with_hint("if this feature is intentional scaffolding, you can ignore this warning"),
            );
        }
    }
}

/// Extract meaningful keywords from a responsibility statement.
/// Filters out common stop words and short words, applies basic stemming.
fn extract_keywords(text: &str) -> Vec<String> {
    let stop_words: std::collections::HashSet<&str> = [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "shall", "can", "that", "which", "who",
        "whom", "this", "these", "those", "it", "its", "of", "in", "to",
        "for", "with", "on", "at", "by", "from", "as", "into", "through",
        "during", "before", "after", "above", "below", "between", "and",
        "or", "but", "not", "no", "nor", "so", "yet", "both", "either",
        "neither", "each", "every", "all", "any", "few", "more", "most",
        "other", "some", "such", "than", "too", "very", "just", "also",
        "about", "up", "out", "off", "over", "under", "again", "further",
        "then", "once", "here", "there", "when", "where", "why", "how",
        "what", "turns", "into", "environment",
    ].iter().copied().collect();

    text.split(|c: char| !c.is_alphanumeric() && c != '-')
        .map(|w| basic_stem(&w.to_lowercase()))
        .filter(|w| w.len() > 2 && !stop_words.contains(w.as_str()))
        .collect()
}

/// Minimal suffix stemming: strip common English suffixes so "clusters" matches "cluster", etc.
fn basic_stem(word: &str) -> String {
    let w = word.to_lowercase();
    // Order matters: check longer suffixes first
    for suffix in &["iness", "ation", "ment", "ness", "able", "ible", "ting", "ing", "ies", "ous", "ful", "ers", "ure", "ive", "ely", "ory", "ary", "ion", "ed", "es", "er", "ly", "al", "en", "ty"] {
        if w.len() > suffix.len() + 2 {
            if let Some(stem) = w.strip_suffix(suffix) {
                return stem.to_string();
            }
        }
    }
    // Handle trailing 's' last (only if word > 3 chars to avoid stripping "bus" → "bu")
    if w.len() > 3 {
        if let Some(stem) = w.strip_suffix('s') {
            return stem.to_string();
        }
    }
    w
}

/// Extract keywords from feature title and first ~200 chars of body
fn extract_keywords_from_feature(title: &str, body: &str) -> Vec<String> {
    let preview = if body.len() > 200 { &body[..200] } else { body };
    let combined = format!("{} {}", title, preview);
    extract_keywords(&combined)
}

/// Check if a feature title/body indicates infrastructure or tooling.
/// These are expected even when they don't directly overlap with the responsibility.
fn is_infrastructure_feature(title: &str, body: &str) -> bool {
    let infra_indicators = [
        "cli", "tool", "tooling", "infrastructure", "scaffold", "migration",
        "error", "diagnostic", "validation", "test", "testing", "ci", "cd",
        "pipeline", "config", "configuration", "schema", "format", "spec",
        "specification", "graph", "bundle", "context", "agent", "mcp",
        "hook", "lint", "build", "deploy", "monitor", "metric", "log",
        "auth", "security", "init", "setup", "bootstrap", "onboard",
        "documentation", "docs", "guide", "review", "check", "verify",
        "prompt", "author", "drift", "gap", "coverage", "preflight",
        "implement", "orchestrat", "automat",
    ];
    let lower_title = title.to_lowercase();
    let lower_body_preview = if body.len() > 200 {
        body[..200].to_lowercase()
    } else {
        body.to_lowercase()
    };
    infra_indicators.iter().any(|ind| {
        lower_title.contains(ind) || lower_body_preview.contains(ind)
    })
}

/// Check for a top-level " and " conjunction in a responsibility statement.
/// Subordinate conjunctions ("no X and no Y", comma-separated lists) are exempt.
pub fn contains_top_level_conjunction(s: &str) -> bool {
    let norm: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let lower = norm.to_lowercase();
    let mut pos = 0;
    while let Some(idx) = lower[pos..].find(" and ") {
        let abs = pos + idx;
        let before = norm[..abs].trim();
        let after_lower = norm[abs + 5..].trim_start().to_lowercase();
        let skip = (after_lower.starts_with("no ") && before.to_lowercase().contains("no "))
            || before.ends_with(',')
            || before.ends_with('\u{2014}')
            || before.ends_with('-');
        if !skip { return true; }
        pos = abs + 5;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::path::PathBuf;

    fn make_feature(id: &str, title: &str) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.to_string(),
                title: title.to_string(),
                phase: 1,
                status: FeatureStatus::Planned,
                depends_on: vec![],
                adrs: vec![],
                tests: vec![],
                domains: vec![],
                domains_acknowledged: std::collections::HashMap::new(),
                bundle: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("{}.md", id)),
        }
    }

    #[test]
    fn w019_grocery_list_outside_cloud_scope() {
        let features = vec![
            make_feature("FT-099", "Grocery List Management"),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let mut result = CheckResult::new();
        check_responsibility(
            &graph,
            Some("A private cloud platform for Raspberry Pi"),
            &mut result,
        );
        assert!(
            result.warnings.iter().any(|w| w.code == "W019"),
            "should emit W019 for grocery list in a cloud platform"
        );
    }

    #[test]
    fn w019_not_emitted_for_in_scope_feature() {
        let features = vec![
            make_feature("FT-001", "Cluster Node Discovery"),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let mut result = CheckResult::new();
        check_responsibility(
            &graph,
            Some("A private cloud platform for Raspberry Pi clusters"),
            &mut result,
        );
        assert!(
            !result.warnings.iter().any(|w| w.code == "W019"),
            "should not emit W019 for cluster-related feature"
        );
    }

    #[test]
    fn w019_suppressed_when_responsibility_absent() {
        let features = vec![
            make_feature("FT-099", "Grocery List Management"),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let mut result = CheckResult::new();
        check_responsibility(&graph, None, &mut result);
        assert!(
            !result.warnings.iter().any(|w| w.code == "W019"),
            "should not emit W019 when responsibility is absent"
        );
    }

    #[test]
    fn w019_infra_feature_exempt() {
        let features = vec![
            make_feature("FT-010", "CLI Error Diagnostics"),
        ];
        let graph = KnowledgeGraph::build(features, vec![], vec![]);
        let mut result = CheckResult::new();
        check_responsibility(
            &graph,
            Some("A private cloud platform for Raspberry Pi"),
            &mut result,
        );
        assert!(
            !result.warnings.iter().any(|w| w.code == "W019"),
            "infrastructure features should be exempt from W019"
        );
    }
}
