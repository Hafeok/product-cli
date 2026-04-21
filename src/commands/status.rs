//! Status summary, impact analysis — thin adapters over the `status` slice.
//!
//! Handlers here load the graph, call a pure builder in `product_lib::status`,
//! then return `Output::Both { text, json }` so the dispatcher renders per
//! the `--format` flag. No println!, no format branching in the handler.

use product_lib::cycle_times;
use product_lib::{error::ProductError, status};

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
    let text = status::render_project_summary_text(&summary, phase.is_some());
    let json = serde_json::to_value(&summary).unwrap_or(serde_json::Value::Null);
    Ok(Output::both(text, json))
}

fn serde_json_format_matches(needle: &str, haystack: &str) -> bool {
    haystack.eq_ignore_ascii_case(needle)
}
