//! Pattern suggestion ranking (FT-073, ADR-050).
//!
//! `suggest_patterns(graph, feature_domains)` ranks every live pattern by
//! domain-overlap with the supplied feature domains, breaking ties by the
//! pattern-aware betweenness centrality from FT-071. Pure function with no
//! I/O — callers in `author-feature` and the MCP layer compose it with their
//! own rendering.

use crate::graph::KnowledgeGraph;
use crate::types::{Pattern, PatternStatus};
use std::collections::HashSet;

/// One ranked suggestion for the caller to render.
#[derive(Debug, Clone)]
pub struct Suggestion<'a> {
    pub pattern: &'a Pattern,
    /// Number of domains that intersect between the pattern and the feature.
    pub overlap: usize,
    /// Pattern-aware centrality. Higher is more architecturally pivotal.
    pub centrality: f64,
}

/// Rank live patterns by domain overlap with `feature_domains`, then by
/// pattern-aware centrality. Deprecated patterns are excluded — the author
/// should not be steered toward a deprecated pattern. The result is sorted
/// descending: most relevant first.
///
/// When `feature_domains` is empty, the result is empty: no overlap can
/// score above zero.
pub fn suggest_patterns<'g>(
    graph: &'g KnowledgeGraph,
    feature_domains: &[String],
) -> Vec<Suggestion<'g>> {
    if feature_domains.is_empty() {
        return Vec::new();
    }
    let feature_set: HashSet<&str> = feature_domains.iter().map(String::as_str).collect();
    let centrality = graph.betweenness_centrality_with(true);

    let mut suggestions: Vec<Suggestion<'g>> = graph
        .patterns
        .values()
        .filter(|p| p.front.status == PatternStatus::Live)
        .filter_map(|p| {
            let overlap = p
                .front
                .domains
                .iter()
                .filter(|d| feature_set.contains(d.as_str()))
                .count();
            if overlap == 0 {
                return None;
            }
            let c = centrality
                .get(&p.front.id)
                .copied()
                .unwrap_or(0.0);
            Some(Suggestion {
                pattern: p,
                overlap,
                centrality: c,
            })
        })
        .collect();

    suggestions.sort_by(|a, b| {
        b.overlap
            .cmp(&a.overlap)
            .then_with(|| {
                b.centrality
                    .partial_cmp(&a.centrality)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.pattern.front.id.cmp(&b.pattern.front.id))
    });
    suggestions
}

/// Render the suggestions as a markdown block for an authoring prompt.
/// Returns `None` when there are no suggestions (silent — no block).
pub fn render_suggestions_block(suggestions: &[Suggestion]) -> Option<String> {
    if suggestions.is_empty() {
        return None;
    }
    let mut out = String::from("## Matching patterns\n\n");
    out.push_str(
        "Patterns whose declared `domains:` overlap this feature's domains. \
Cite the ones that apply via `product feature link FT-XXX --pattern PAT-YYY` \
(or include `patterns:` on the create request).\n\n",
    );
    for (i, s) in suggestions.iter().enumerate() {
        out.push_str(&format!(
            "{}. **{}** — {} [{}]\n",
            i + 1,
            s.pattern.front.id,
            s.pattern.front.title,
            s.pattern.front.status,
        ));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Feature, FeatureFrontMatter, FeatureStatus, PatternFrontMatter};
    use std::path::PathBuf;

    fn mk_pat(id: &str, status: PatternStatus, domains: Vec<&str>) -> Pattern {
        Pattern {
            front: PatternFrontMatter {
                id: id.into(),
                title: format!("Pattern {}", id),
                status,
                domains: domains.into_iter().map(String::from).collect(),
                adrs: vec![],
                requires: vec![],
                examples: vec![],
                deprecated_by: None,
            },
            body: String::new(),
            path: PathBuf::from(format!("docs/patterns/{}.md", id)),
        }
    }

    fn mk_feat(id: &str) -> Feature {
        Feature {
            front: FeatureFrontMatter {
                id: id.into(),
                title: id.into(),
                phase: 1,
                status: FeatureStatus::Planned,
                depends_on: vec![],
                adrs: vec![],
                tests: vec![],
                domains: vec![],
                domains_acknowledged: Default::default(),
                patterns: vec![],
                due_date: None,
                bundle: None,
                adrs_rejected: vec![],
            },
            body: String::new(),
            path: PathBuf::from(format!("docs/features/{}.md", id)),
        }
    }

    #[test]
    fn empty_feature_domains_returns_empty() {
        let g = KnowledgeGraph::build_full(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![mk_pat("PAT-001", PatternStatus::Live, vec!["api"])],
        );
        assert!(suggest_patterns(&g, &[]).is_empty());
    }

    #[test]
    fn overlapping_pattern_is_returned() {
        let g = KnowledgeGraph::build_full(
            vec![mk_feat("FT-001")],
            vec![],
            vec![],
            vec![],
            vec![
                mk_pat("PAT-A", PatternStatus::Live, vec!["api"]),
                mk_pat("PAT-B", PatternStatus::Live, vec!["observability"]),
                mk_pat("PAT-C", PatternStatus::Live, vec!["unrelated"]),
            ],
        );
        let result = suggest_patterns(&g, &["api".into(), "observability".into()]);
        let ids: Vec<&str> = result.iter().map(|s| s.pattern.front.id.as_str()).collect();
        assert!(ids.contains(&"PAT-A"));
        assert!(ids.contains(&"PAT-B"));
        assert!(!ids.contains(&"PAT-C"));
    }

    #[test]
    fn deprecated_patterns_are_excluded() {
        let g = KnowledgeGraph::build_full(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![mk_pat("PAT-001", PatternStatus::Deprecated, vec!["api"])],
        );
        let result = suggest_patterns(&g, &["api".into()]);
        assert!(result.is_empty());
    }

    #[test]
    fn higher_overlap_outranks_lower() {
        let g = KnowledgeGraph::build_full(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![
                mk_pat("PAT-LOW", PatternStatus::Live, vec!["api"]),
                mk_pat("PAT-HIGH", PatternStatus::Live, vec!["api", "observability"]),
            ],
        );
        let result = suggest_patterns(&g, &["api".into(), "observability".into()]);
        assert_eq!(result.first().map(|s| s.pattern.front.id.as_str()), Some("PAT-HIGH"));
    }

    #[test]
    fn render_returns_none_on_empty() {
        assert!(render_suggestions_block(&[]).is_none());
    }

    #[test]
    fn render_block_lists_patterns() {
        let pat = mk_pat("PAT-001", PatternStatus::Live, vec!["api"]);
        let s = Suggestion {
            pattern: &pat,
            overlap: 1,
            centrality: 0.5,
        };
        let block = render_suggestions_block(&[s]).expect("block");
        assert!(block.contains("## Matching patterns"));
        assert!(block.contains("PAT-001"));
        assert!(block.contains("live"));
    }
}
