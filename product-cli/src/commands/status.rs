//! Status summary, impact analysis — thin adapters over the `status` slice.
//!
//! Handlers here load the graph, call a pure builder in `product_core::status`,
//! then return `Output::Both { text, json }` so the dispatcher renders per
//! the `--format` flag. No println!, no format branching in the handler.

use product_core::author::domain::session_dir;
use product_core::cycle_times;
use product_core::pf::how::HowContract;
use product_core::pf::layout::LayoutModel;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use product_core::{error::ProductError, status};

use super::{load_graph_typed, CmdResult, Output};

pub(crate) fn handle_impact(id: &str, _fmt: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    if !graph.all_ids().contains(id) {
        return Err(ProductError::NotFound(format!("artifact {}", id)));
    }
    let impact = graph.impact(id);
    let json = serde_json::json!({
        "seed": impact.seed,
        "direct_features": impact.direct_features,
        "direct_tests": impact.direct_tests,
        "direct_adrs": impact.direct_adrs,
        "direct_deps": impact.direct_deps,
        "direct_patterns": impact.direct_patterns,
        "transitive_features": impact.transitive_features,
        "transitive_tests": impact.transitive_tests,
    });

    // Text rendering still delegates to the existing ImpactResult::print,
    // which writes to stdout directly. Capturing that into Output::Text
    // requires a bigger refactor in graph/types.rs — out of scope for this
    // pilot. For now, text branch prints, json branch goes through Output.
    if serde_json_format_matches("json", _fmt) {
        Ok(Output::json(json))
    } else {
        impact.print(&graph);
        Ok(Output::Empty)
    }
}

pub(crate) fn handle_status(
    phase: Option<u32>,
    untested: bool,
    failing: bool,
    _fmt: &str,
) -> CmdResult {
    let (config, root, graph) = load_graph_typed()?;

    if untested {
        let list = status::build_untested_list(&graph);
        let text = status::render_feature_list_text("Features with no linked test criteria:", &list);
        let json = serde_json::to_value(&list.items).unwrap_or(serde_json::Value::Null);
        return Ok(Output::both(text, json));
    }
    if failing {
        let list = status::build_failing_list(&graph);
        let text = status::render_feature_list_text("Features with failing tests:", &list);
        let json = serde_json::to_value(&list.items).unwrap_or(serde_json::Value::Null);
        return Ok(Output::both(text, json));
    }

    // Build cycle-time context so the status view can render the column
    // (FT-054, ADR-046 §12).
    let tag_ts = super::cycle_times::read_tag_timestamps(&root, &graph);
    let ct_report = cycle_times::build_report(
        &graph,
        &tag_ts,
        config.cycle_times.recent_window,
        config.cycle_times.trend_threshold,
        None,
    );
    let recent_median = ct_report.summary.recent_5.as_ref().map(|s| s.median);
    let complete_count = ct_report.summary.count;

    let summary = status::build_project_summary_with_cycle_times(
        &config,
        &graph,
        phase,
        Some(&tag_ts),
        recent_median,
        complete_count,
    );
    let mut text = status::render_project_summary_text(&summary, phase.is_some());
    let mut json = serde_json::to_value(&summary).unwrap_or(serde_json::Value::Null);

    // Append the framework What/How/delivery graph summary, when present.
    if let Some((fw_text, fw_json)) = framework_section() {
        text.push('\n');
        text.push_str(&fw_text);
        if let serde_json::Value::Object(map) = &mut json {
            map.insert("framework".to_string(), fw_json);
        }
    }
    Ok(Output::both(text, json))
}

/// Count `*.yaml` files directly under `dir` (0 if it does not exist).
fn count_yaml(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .map(|it| {
            it.flatten()
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
                .count()
        })
        .unwrap_or(0)
}

/// Summarise the framework graph (What / How / delivery) for the default
/// product. `None` when no framework artifacts exist (so legacy repos are
/// unaffected).
fn framework_section() -> Option<(String, serde_json::Value)> {
    let product = super::shared::default_product_name()?;
    let pdir = super::shared::domain_root().join(".product");
    let graph = DomainSession::load(&session_dir(&super::shared::domain_root(), &product))
        .ok()
        .map(|s| s.graph);
    let how = std::fs::read_to_string(pdir.join("how-contract.yaml"))
        .ok()
        .and_then(|t| HowContract::from_yaml(&t).ok());
    let layout_rules = std::fs::read_to_string(pdir.join("layout.yaml"))
        .ok()
        .and_then(|t| LayoutModel::from_yaml(&t).ok())
        .map(|l| l.layout.len())
        .unwrap_or(0);
    let deciders = count_yaml(&pdir.join("deciders"));
    let delivery = Delivery {
        slices: count_yaml(&pdir.join("slices")),
        deliverables: count_yaml(&pdir.join("deliverables")),
        releases: count_yaml(&pdir.join("releases")),
    };

    if graph.is_none() && how.is_none() && deciders == 0 && delivery.slices == 0 && delivery.deliverables == 0 && delivery.releases == 0 {
        return None;
    }
    Some(render_framework(graph.as_ref(), how.as_ref(), layout_rules, deciders, delivery))
}

/// Delivery-layer counts (§7.1).
struct Delivery {
    slices: usize,
    deliverables: usize,
    releases: usize,
}

/// Render the framework summary as a text block plus a JSON object.
fn render_framework(
    graph: Option<&DomainGraph>,
    how: Option<&HowContract>,
    layout_rules: usize,
    deciders: usize,
    delivery: Delivery,
) -> (String, serde_json::Value) {
    let counts = graph.map(|g| g.counts()).unwrap_or_default();
    let n = |k: &str| counts.iter().find(|(name, _)| *name == k).map(|(_, c)| *c).unwrap_or(0);
    let (decisions, principles, patterns) = how
        .map(|h| (h.top_decisions.len(), h.principles.len(), h.patterns.len()))
        .unwrap_or((0, 0, 0));
    let contracts = how.map(|h| 1 + usize::from(h.infrastructure_contract.is_some())).unwrap_or(0);

    let what = if graph.is_some() {
        format!("{} contexts, {} entities, {} events, {} commands, {deciders} deciders",
            n("BoundedContext"), n("Entity"), n("Event"), n("Command"))
    } else {
        "(none captured)".to_string()
    };
    let how_line = if how.is_some() {
        format!("{decisions} decisions, {principles} principles, {patterns} patterns, {contracts} contracts, {layout_rules} layout rules")
    } else {
        "(none)".to_string()
    };
    let text = format!(
        "── Framework graph ──\nWhat: {what}\nHow: {how_line}\nDelivery: {} slices, {} deliverables, {} releases\n",
        delivery.slices, delivery.deliverables, delivery.releases,
    );
    let json = serde_json::json!({
        "what": { "contexts": n("BoundedContext"), "entities": n("Entity"), "events": n("Event"), "commands": n("Command"), "deciders": deciders },
        "how": { "decisions": decisions, "principles": principles, "patterns": patterns, "contracts": contracts, "layout_rules": layout_rules },
        "delivery": { "slices": delivery.slices, "deliverables": delivery.deliverables, "releases": delivery.releases },
    });
    (text, json)
}

fn serde_json_format_matches(needle: &str, haystack: &str) -> bool {
    haystack.eq_ignore_ascii_case(needle)
}
