//! Template-aware bundle renderer — orchestrates collect + format passes.
//!
//! Format-specific rendering lives in sibling modules (`render_xml`,
//! `render_text`, `render_structured`). This module owns the public entry
//! point and the token-budget bookkeeping.

use super::collect::collect;
use super::loader::Template;
use super::resolve::ResolvedTemplate;
use super::{render_structured, render_text, render_xml};
use crate::graph::KnowledgeGraph;

/// Optional product info for the bundle header (FT-039).
pub struct ProductInfo<'a> {
    pub name: &'a str,
    pub responsibility: &'a str,
}

/// A rendered bundle plus its computed token-budget signals.
pub struct RenderedBundle {
    pub format: String,
    pub target: String,
    pub content: String,
    pub token_count_approx: usize,
    pub exceeded_target_max: bool,
    pub exceeded_hard_max: bool,
}

/// Render a feature bundle through a resolved template. Returns `None` when
/// the feature does not exist in the graph.
pub fn render_feature(
    graph: &KnowledgeGraph,
    feature_id: &str,
    depth: usize,
    template: &ResolvedTemplate,
    product_info: Option<ProductInfo<'_>>,
) -> Option<RenderedBundle> {
    let feature = graph.features.get(feature_id)?;
    let collected = collect(graph, feature, depth, &template.template);
    let pi = product_info.as_ref();
    let content = match template.template.format.structure.as_str() {
        "xml" => render_xml::render(&collected, &template.template, pi),
        "markdown" => render_text::render(&collected, &template.template, pi, true),
        "plain" => render_text::render(&collected, &template.template, pi, false),
        "yaml" => render_structured::render_yaml(&collected, &template.template, pi),
        "json" => render_structured::render_json(&collected, &template.template, pi),
        _ => render_text::render(&collected, &template.template, pi, false),
    };
    Some(make_bundle(content, template, &template.template))
}

fn make_bundle(content: String, template: &ResolvedTemplate, tpl: &Template) -> RenderedBundle {
    let token_count_approx = content.len() / 4;
    let target_max = tpl.token_budget.target_max as usize;
    let hard_max = tpl.token_budget.hard_max as usize;
    RenderedBundle {
        format: tpl.format.structure.clone(),
        target: template.name.clone(),
        content,
        token_count_approx,
        exceeded_target_max: target_max > 0 && token_count_approx > target_max,
        exceeded_hard_max: hard_max > 0 && token_count_approx > hard_max,
    }
}
