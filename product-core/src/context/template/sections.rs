//! Section-content builders — produce one Markdown body per recognised
//! section name. Pure functions over the assembled `Collected` view; these
//! never re-query the graph and never mutate artifact bodies (ADR-049).

use super::collect::Collected;
use super::loader::Template;
use crate::types::AdrStatus;
use std::collections::HashSet;

pub fn section_task(c: &Collected) -> Option<String> {
    Some(format!(
        "Implement {} ({}). Read the feature, governing decisions, and test criteria below before scaffolding.",
        c.feature.front.id, c.feature.front.title,
    ))
}

pub fn section_feature(c: &Collected) -> Option<String> {
    Some(format!(
        "## {} — {}\n\nphase: {}\nstatus: {}\n\n{}",
        c.feature.front.id,
        c.feature.front.title,
        c.feature.front.phase,
        c.feature.front.status,
        c.feature.body.trim(),
    ))
}

pub fn section_deliverables(c: &Collected) -> Option<String> {
    let mut bullets: Vec<String> = Vec::new();
    bullets.push(format!(
        "Implement feature {} ({})",
        c.feature.front.id, c.feature.front.title,
    ));
    for tc in &c.tests {
        bullets.push(format!(
            "Make test criterion {} ({}) pass",
            tc.front.id, tc.front.title,
        ));
    }
    if bullets.is_empty() {
        return None;
    }
    Some(
        bullets
            .into_iter()
            .map(|b| format!("- {}", b))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

pub fn section_governing_adrs(c: &Collected) -> Option<String> {
    if c.adrs.is_empty() {
        return None;
    }
    let mut out = String::new();
    for adr in &c.adrs {
        let suffix = if adr.front.status == AdrStatus::Superseded {
            adr.front
                .superseded_by
                .first()
                .map(|by| format!(" [SUPERSEDED by {}]", by))
                .unwrap_or_else(|| " [SUPERSEDED]".to_string())
        } else {
            String::new()
        };
        out.push_str(&format!(
            "### {} — {}{}\n\n**Status:** {:?}\n\n{}\n\n",
            adr.front.id,
            adr.front.title,
            suffix,
            adr.front.status,
            adr.body.trim(),
        ));
    }
    Some(out.trim_end().to_string())
}

pub fn section_test_criteria(c: &Collected) -> Option<String> {
    if c.tests.is_empty() {
        return None;
    }
    let mut out = String::new();
    for tc in &c.tests {
        out.push_str(&format!(
            "### {} — {} ({})\n\n{}\n\n",
            tc.front.id,
            tc.front.title,
            tc.front.test_type,
            tc.body.trim(),
        ));
    }
    Some(out.trim_end().to_string())
}

pub fn section_dependencies(c: &Collected) -> Option<String> {
    if c.deps.is_empty() {
        return None;
    }
    let mut out = String::new();
    for dep in &c.deps {
        let version = dep.front.version.as_deref().unwrap_or("~");
        out.push_str(&format!(
            "### {} — {} [{}, {}]\n\n{}\n\n",
            dep.front.id,
            dep.front.title,
            dep.front.dep_type,
            version,
            dep.body.trim(),
        ));
    }
    Some(out.trim_end().to_string())
}

pub fn section_constraints(_c: &Collected) -> Option<String> {
    Some(
        "- All tests must pass under `cargo t`\n- `cargo clippy -- -D warnings -D clippy::unwrap_used` must succeed\n- File length cap: 400 lines per source file"
            .to_string(),
    )
}

pub fn section_bundle_metrics(c: &Collected) -> Option<String> {
    let bm = c.feature.front.bundle.as_ref()?;
    Some(format!(
        "tokens-approx: {}\ndepth-1-adrs: {}\ntcs: {}\npatterns: {}\nmeasured-at: {}",
        bm.tokens_approx, bm.depth_1_adrs, bm.tcs, bm.patterns, bm.measured_at,
    ))
}

/// FT-071: render the `## Patterns` section. Patterns appear in topological
/// order over `requires:`; deprecated patterns carry a status banner.
pub fn section_patterns(c: &Collected) -> Option<String> {
    if c.patterns.is_empty() {
        return None;
    }
    let mut out = String::new();
    for pat in &c.patterns {
        out.push_str(&format!(
            "### {} — {}\n\n",
            pat.front.id, pat.front.title,
        ));
        if pat.front.status == crate::types::PatternStatus::Deprecated {
            let by = pat
                .front
                .deprecated_by
                .as_deref()
                .map(|r| format!(" (replaced by {})", r))
                .unwrap_or_default();
            out.push_str(&format!("**Status:** Deprecated{}\n\n", by));
        }
        out.push_str(&format!("{}\n\n", pat.body.trim()));
    }
    Some(out.trim_end().to_string())
}

pub fn build_section(name: &str, c: &Collected) -> Option<String> {
    match name {
        "task" => section_task(c),
        "feature" => section_feature(c),
        "deliverables" => section_deliverables(c),
        "governing_adrs" => section_governing_adrs(c),
        "test_criteria" => section_test_criteria(c),
        "dependencies" => section_dependencies(c),
        "linked_documentation" => None,
        "constraints" => section_constraints(c),
        "bundle_metrics" => section_bundle_metrics(c),
        "patterns" => section_patterns(c),
        _ => None,
    }
}

/// Produce the final ordered list of section names, honouring
/// `deliverables_at_top` and `critical_first` flags.
pub fn ordered_sections(tpl: &Template) -> Vec<String> {
    let mut sections = tpl.ordering.sections.clone();
    if !tpl.ordering.deliverables_at_top {
        return sections;
    }
    let mut new_order: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    if tpl.ordering.critical_first && sections.contains(&"task".to_string()) {
        new_order.push("task".to_string());
        seen.insert("task".to_string());
    }
    if sections.contains(&"deliverables".to_string()) && seen.insert("deliverables".to_string()) {
        new_order.push("deliverables".to_string());
    }
    for s in sections.drain(..) {
        if seen.insert(s.clone()) {
            new_order.push(s);
        }
    }
    new_order
}

/// Title used as a heading prefix in Markdown / plain output.
pub fn section_title(name: &str) -> &'static str {
    match name {
        "task" => "Task",
        "feature" => "Feature",
        "deliverables" => "Deliverables",
        "governing_adrs" => "Governing ADRs",
        "test_criteria" => "Test Criteria",
        "dependencies" => "Dependencies",
        "linked_documentation" => "Linked Documentation",
        "constraints" => "Constraints",
        "bundle_metrics" => "Bundle Metrics",
        "patterns" => "Patterns",
        _ => "Section",
    }
}
