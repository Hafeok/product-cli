//! Feature navigation, creation, linking, status management.

use clap::Subcommand;
use product_lib::{error::ProductError, graph, types};

use super::{load_graph, load_graph_typed, BoxResult, CmdResult, Output};
mod feature_write_ops {
    pub(crate) use super::super::feature_write::*;
}

#[derive(Subcommand)]
pub enum FeatureCommands {
    /// List all features
    List {
        #[arg(long)]
        phase: Option<u32>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Show a feature's details
    Show { id: String },
    /// List ADRs linked to a feature
    Adrs { id: String },
    /// List test criteria for a feature
    Tests { id: String },
    /// Show the full dependency tree for a feature
    Deps { id: String },
    /// Show the next feature to implement (topological order)
    Next {
        /// Skip phase gate checks (allow phase-2+ features even if prior gates fail)
        #[arg(long)]
        ignore_phase_gate: bool,
    },
    /// Create a new feature file
    New {
        /// Feature title
        title: String,
        /// Phase number
        #[arg(long, default_value = "1")]
        phase: u32,
    },
    /// Link a feature to an ADR, test, or dependency
    Link {
        /// Feature ID
        id: String,
        /// ADR ID to link
        #[arg(long)]
        adr: Option<String>,
        /// Test ID to link
        #[arg(long)]
        test: Option<String>,
        /// Feature ID this feature depends on
        #[arg(long)]
        dep: Option<String>,
        /// Accept inferred transitive TC links without prompting (required in non-TTY use)
        #[arg(long)]
        yes: bool,
    },
    /// Set feature status
    Status {
        /// Feature ID
        id: String,
        /// New status: planned, in-progress, complete, abandoned
        new_status: String,
    },
    /// Acknowledge a domain or ADR gap with reasoning
    Acknowledge {
        /// Feature ID
        id: String,
        /// Domain to acknowledge
        #[arg(long)]
        domain: Option<String>,
        /// ADR to acknowledge
        #[arg(long)]
        adr: Option<String>,
        /// Reasoning (required unless --remove)
        #[arg(long)]
        reason: Option<String>,
        /// Remove the acknowledgement instead of adding
        #[arg(long)]
        remove: bool,
    },
    /// Add or remove concern domains on a feature
    Domain {
        /// Feature ID
        id: String,
        /// Domain to add (repeatable)
        #[arg(long)]
        add: Vec<String>,
        /// Domain to remove (repeatable)
        #[arg(long)]
        remove: Vec<String>,
    },
}

pub(crate) fn handle_feature(cmd: FeatureCommands, fmt: &str) -> BoxResult {
    match cmd {
        FeatureCommands::List { phase, status } => {
            super::render(feature_list(phase, status), fmt)
        }
        FeatureCommands::Show { id } => super::render(feature_show(&id), fmt),
        FeatureCommands::Adrs { id } => super::render(feature_adrs(&id), fmt),
        FeatureCommands::Tests { id } => super::render(feature_tests(&id), fmt),
        FeatureCommands::Deps { id } => super::render(feature_deps(&id), fmt),
        FeatureCommands::Next { ignore_phase_gate } => feature_next(ignore_phase_gate),
        FeatureCommands::New { title, phase } => {
            super::render(feature_write_ops::feature_new(&title, phase), fmt)
        }
        FeatureCommands::Link { id, adr, test, dep, yes } => {
            feature_write_ops::feature_link(&id, adr, test, dep, yes)
        }
        FeatureCommands::Status { id, new_status } => {
            super::render(feature_write_ops::feature_status(&id, &new_status), fmt)
        }
        FeatureCommands::Acknowledge { id, domain, adr, reason, remove } => {
            feature_write_ops::feature_acknowledge(&id, domain, adr, reason, remove)
        }
        FeatureCommands::Domain { id, add, remove } => {
            super::render(feature_write_ops::feature_domain(&id, add, remove), fmt)
        }
    }
}

fn feature_list(phase: Option<u32>, status: Option<String>) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let mut features: Vec<&types::Feature> = graph.features.values().collect();
    features.sort_by_key(|f| &f.front.id);

    if let Some(p) = phase {
        features.retain(|f| f.front.phase == p);
    }
    if let Some(ref s) = status {
        let target: types::FeatureStatus = s.parse().map_err(ProductError::ConfigError)?;
        features.retain(|f| f.front.status == target);
    }

    let json = serde_json::Value::Array(
        features
            .iter()
            .map(|f| {
                serde_json::json!({
                    "id": f.front.id,
                    "phase": f.front.phase,
                    "status": f.front.status.to_string(),
                    "title": f.front.title,
                })
            })
            .collect(),
    );
    let text = render_feature_list_text(&features);
    Ok(Output::both(text, json))
}

fn render_feature_list_text(features: &[&types::Feature]) -> String {
    let mut out = format!("{:<10} {:<8} {:<15} TITLE\n", "ID", "PHASE", "STATUS");
    out.push_str(&"-".repeat(60));
    out.push('\n');
    for f in features {
        out.push_str(&format!(
            "{:<10} {:<8} {:<15} {}\n",
            f.front.id, f.front.phase, f.front.status, f.front.title
        ));
    }
    out
}

fn feature_show(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
    let json = serde_json::json!({
        "id": f.front.id,
        "title": f.front.title,
        "phase": f.front.phase,
        "status": f.front.status.to_string(),
        "depends_on": f.front.depends_on,
        "adrs": f.front.adrs,
        "tests": f.front.tests,
        "body": f.body,
    });
    let text = render_feature_show_text(f);
    Ok(Output::both(text, json))
}

fn render_feature_show_text(f: &types::Feature) -> String {
    let mut out = format!("# {} — {}\n\n", f.front.id, f.front.title);
    out.push_str(&format!("Phase:      {}\n", f.front.phase));
    out.push_str(&format!("Status:     {}\n", f.front.status));
    out.push_str(&format!(
        "Depends-on: {}\n",
        if f.front.depends_on.is_empty() {
            "(none)".to_string()
        } else {
            f.front.depends_on.join(", ")
        }
    ));
    out.push_str(&format!(
        "ADRs:       {}\n",
        if f.front.adrs.is_empty() {
            "(none)".to_string()
        } else {
            f.front.adrs.join(", ")
        }
    ));
    out.push_str(&format!(
        "Tests:      {}\n",
        if f.front.tests.is_empty() {
            "(none)".to_string()
        } else {
            f.front.tests.join(", ")
        }
    ));
    out.push_str(&format!("\n{}", f.body));
    out
}

fn feature_adrs(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
    let mut text = format!("ADRs linked to {}:\n", id);
    let mut json_arr: Vec<serde_json::Value> = Vec::new();
    for adr_id in &f.front.adrs {
        if let Some(adr) = graph.adrs.get(adr_id.as_str()) {
            text.push_str(&format!(
                "  {} — {} ({})\n",
                adr.front.id, adr.front.title, adr.front.status
            ));
            json_arr.push(serde_json::json!({
                "id": adr.front.id,
                "title": adr.front.title,
                "status": adr.front.status.to_string(),
            }));
        } else {
            text.push_str(&format!("  {} (broken link)\n", adr_id));
            json_arr.push(serde_json::json!({ "id": adr_id, "broken_link": true }));
        }
    }
    Ok(Output::both(text, serde_json::Value::Array(json_arr)))
}

fn feature_tests(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
    let mut text = format!("Tests linked to {}:\n", id);
    let mut json_arr: Vec<serde_json::Value> = Vec::new();
    for test_id in &f.front.tests {
        if let Some(tc) = graph.tests.get(test_id.as_str()) {
            text.push_str(&format!(
                "  {} — {} ({}, {})\n",
                tc.front.id, tc.front.title, tc.front.test_type, tc.front.status
            ));
            json_arr.push(serde_json::json!({
                "id": tc.front.id,
                "title": tc.front.title,
                "type": tc.front.test_type.to_string(),
                "status": tc.front.status.to_string(),
            }));
        } else {
            text.push_str(&format!("  {} (broken link)\n", test_id));
            json_arr.push(serde_json::json!({ "id": test_id, "broken_link": true }));
        }
    }
    Ok(Output::both(text, serde_json::Value::Array(json_arr)))
}

fn feature_deps(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let _f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
    let mut text = format!("Dependency tree for {}:\n", id);
    append_dep_tree(&mut text, &graph, id, 0, &mut std::collections::HashSet::new());
    // JSON rendering for a tree is a deeper restructure — text only for now.
    Ok(Output::text(text))
}

fn feature_next(ignore_phase_gate: bool) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    match graph.feature_next_with_gate(ignore_phase_gate)? {
        graph::FeatureNextResult::Ready(id) => {
            if ignore_phase_gate {
                eprintln!("warning: --ignore-phase-gate: phase gate checks skipped");
            }
            let f = &graph.features[&id];
            println!("{} — {} (phase {}, {})", f.front.id, f.front.title, f.front.phase, f.front.status);
        }
        graph::FeatureNextResult::Blocked { candidate, blocked_phase, exit_criteria } => {
            print_blocked_next(&graph, &candidate, blocked_phase, &exit_criteria);
        }
        graph::FeatureNextResult::AllDone => {
            println!("All features are complete or have incomplete dependencies.");
        }
    }
    Ok(())
}

fn print_blocked_next(
    graph: &graph::KnowledgeGraph,
    candidate: &str,
    blocked_phase: u32,
    exit_criteria: &[graph::PhaseGateTC],
) {
    let f = &graph.features[candidate];
    println!(
        "  Next candidate: {} — {}  [phase {}, {}]",
        f.front.id, f.front.title, f.front.phase, f.front.status
    );
    let failing: Vec<&graph::PhaseGateTC> = exit_criteria.iter().filter(|tc| !tc.passing).collect();
    eprintln!(
        "  \u{2717} Phase {} locked — Phase {} exit criteria not all passing:",
        f.front.phase, blocked_phase
    );
    eprintln!();
    for tc in exit_criteria {
        let mark = if tc.passing { "passing  \u{2713}" } else { "failing  \u{2717}" };
        eprintln!("    {}  {}  [{}]", tc.id, tc.title, mark);
    }
    eprintln!();
    let failing_ids: Vec<&str> = failing.iter().map(|tc| tc.id.as_str()).collect();
    eprintln!("  Fix {} to unlock Phase {}.", failing_ids.join(" and "), f.front.phase);
    eprintln!("  To skip the gate:  product feature next --ignore-phase-gate");
}

fn append_dep_tree(
    out: &mut String,
    graph: &graph::KnowledgeGraph,
    id: &str,
    indent: usize,
    visited: &mut std::collections::HashSet<String>,
) {
    if visited.contains(id) {
        out.push_str(&format!("{}  {} (circular)\n", "  ".repeat(indent), id));
        return;
    }
    visited.insert(id.to_string());

    if let Some(f) = graph.features.get(id) {
        let marker = match f.front.status {
            types::FeatureStatus::Complete => "[x]",
            types::FeatureStatus::InProgress => "[~]",
            types::FeatureStatus::Planned => "[ ]",
            types::FeatureStatus::Abandoned => "[-]",
        };
        out.push_str(&format!(
            "{}{} {} — {}\n",
            "  ".repeat(indent),
            marker,
            f.front.id,
            f.front.title
        ));
        for dep in &f.front.depends_on {
            append_dep_tree(out, graph, dep, indent + 1, visited);
        }
    }
}
