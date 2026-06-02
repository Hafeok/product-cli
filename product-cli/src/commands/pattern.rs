//! Pattern subcommand surface — thin adapters over `product_lib::pattern`.
//!
//! `new` / `show` / `list` / `status` / `link` all dispatch to the slice in
//! `src/pattern/` (FT-070, ADR-050). MCP parity for these tools lives in
//! `src/mcp/`.

use clap::Subcommand;
use product_lib::{error::ProductError, pattern as pat, types};

use super::{
    acquire_write_lock_typed, load_graph_typed, render, BoxResult, CmdResult, Output,
};

#[derive(Subcommand)]
pub enum PatternCommands {
    /// Link the pattern to an ADR, prerequisite PAT, or example feature.
    Link {
        /// Pattern ID
        id: String,
        /// ADR to operationalise
        #[arg(long)]
        adr: Option<String>,
        /// Prerequisite pattern (requires cycle check)
        #[arg(long)]
        requires: Option<String>,
        /// Feature that exemplifies this pattern
        #[arg(long)]
        example: Option<String>,
    },
    /// List patterns, optionally filtered by status.
    List {
        /// Filter: live | deprecated
        #[arg(long)]
        status: Option<String>,
    },
    /// Scaffold a new pattern file.
    New {
        /// Pattern title
        title: String,
    },
    /// Show a pattern's front-matter and body.
    Show { id: String },
    /// Transition a pattern between `live` and `deprecated`.
    Status {
        /// Pattern ID
        id: String,
        /// New status: live | deprecated
        new_status: String,
        /// Pattern that supersedes this one (when transitioning to deprecated)
        #[arg(long = "deprecated-by", value_name = "PAT")]
        deprecated_by: Option<String>,
    },
}

pub(crate) fn handle_pattern(cmd: PatternCommands, fmt: &str) -> BoxResult {
    match cmd {
        PatternCommands::New { title } => render(pattern_new(&title), fmt),
        PatternCommands::Show { id } => render(pattern_show(&id), fmt),
        PatternCommands::List { status } => render(pattern_list(status), fmt),
        PatternCommands::Status { id, new_status, deprecated_by } => render(
            pattern_status(&id, &new_status, deprecated_by.as_deref()),
            fmt,
        ),
        PatternCommands::Link { id, adr, requires, example } => render(
            pattern_link(&id, adr.as_deref(), requires.as_deref(), example.as_deref()),
            fmt,
        ),
    }
}

fn pattern_new(title: &str) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (config, root, graph) = load_graph_typed()?;
    let existing: Vec<String> = graph.patterns.keys().cloned().collect();
    let plan = pat::plan_create(title, &existing, &config.prefixes.pattern, &config.patterns)?;
    let target_dir = config.resolve_path(&root, &config.paths.patterns);
    let path = pat::apply_create(&plan, &target_dir)?;
    let json = serde_json::json!({ "id": plan.id, "path": path });
    let text = format!("Created: {} at {}", plan.id, path.display());
    Ok(Output::both(text, json))
}

fn pattern_show(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let p = graph
        .patterns
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("pattern {}", id)))?;
    let json = serde_json::json!({
        "id": p.front.id,
        "title": p.front.title,
        "status": p.front.status.to_string(),
        "domains": p.front.domains,
        "adrs": p.front.adrs,
        "requires": p.front.requires,
        "examples": p.front.examples,
        "deprecated-by": p.front.deprecated_by,
        "body": p.body,
    });
    let text = pat::render_show_text(p);
    Ok(Output::both(text, json))
}

fn pattern_list(status: Option<String>) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let want: Option<types::PatternStatus> = match status.as_deref() {
        Some(s) => Some(s.parse().map_err(ProductError::ConfigError)?),
        None => None,
    };
    let mut patterns: Vec<&types::Pattern> = graph.patterns.values().collect();
    if let Some(w) = want {
        patterns.retain(|p| p.front.status == w);
    }
    patterns.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    let json = serde_json::Value::Array(
        patterns
            .iter()
            .map(|p| {
                serde_json::json!({
                    "id": p.front.id,
                    "status": p.front.status.to_string(),
                    "title": p.front.title,
                    "domains": p.front.domains,
                })
            })
            .collect(),
    );
    let text = pat::render_list_text(&patterns);
    Ok(Output::both(text, json))
}

fn pattern_status(id: &str, new_status: &str, deprecated_by: Option<&str>) -> CmdResult {
    let parsed: types::PatternStatus = new_status.parse().map_err(ProductError::ConfigError)?;
    let _lock = acquire_write_lock_typed()?;
    let (_, _, graph) = load_graph_typed()?;
    let plan = pat::plan_status_change(&graph, &graph.patterns, id, parsed, deprecated_by)?;
    pat::apply_status_change(&plan)?;
    let json = serde_json::json!({
        "id": plan.pattern_id,
        "status": plan.new_status.to_string(),
        "previous-status": plan.previous_status.to_string(),
        "deprecated-by": plan.deprecated_by,
    });
    let text = format!(
        "{}: {} -> {}{}",
        plan.pattern_id,
        plan.previous_status,
        plan.new_status,
        match &plan.deprecated_by {
            Some(d) => format!(" (deprecated-by: {})", d),
            None => String::new(),
        },
    );
    Ok(Output::both(text, json))
}

fn pattern_link(
    id: &str,
    adr: Option<&str>,
    requires: Option<&str>,
    example: Option<&str>,
) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (_, _, graph) = load_graph_typed()?;
    let plan = pat::plan_link(&graph, &graph.patterns, id, adr, requires, example)?;
    pat::apply_link(&plan)?;
    let writes_json: Vec<serde_json::Value> = plan
        .writes
        .iter()
        .map(|w| {
            serde_json::json!({
                "path": w.path,
                "kind": w.kind.as_str(),
            })
        })
        .collect();
    let reciprocated_json: Vec<serde_json::Value> = plan
        .reciprocated
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "field": r.field,
            })
        })
        .collect();
    let json = serde_json::json!({
        "id": plan.pattern_id,
        "writes": writes_json,
        "reciprocated": reciprocated_json,
    });
    let text = if plan.writes.is_empty() {
        format!("{}: no changes (idempotent)", plan.pattern_id)
    } else {
        let mut s = format!("{}: linked\n", plan.pattern_id);
        for w in &plan.writes {
            s.push_str(&format!("  wrote {} ({})\n", w.path.display(), w.kind.as_str()));
        }
        for r in &plan.reciprocated {
            s.push_str(&format!("  reciprocated {}.{}\n", r.id, r.field));
        }
        s
    };
    Ok(Output::both(text, json))
}
