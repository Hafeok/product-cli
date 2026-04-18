//! Feature navigation, creation, linking, status management.

use clap::Subcommand;
use product_lib::{error::ProductError, graph, types};

use super::{load_graph, BoxResult};
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
        FeatureCommands::List { phase, status } => feature_list(phase, status, fmt),
        FeatureCommands::Show { id } => feature_show(&id, fmt),
        FeatureCommands::Adrs { id } => feature_adrs(&id),
        FeatureCommands::Tests { id } => feature_tests(&id),
        FeatureCommands::Deps { id } => feature_deps(&id),
        FeatureCommands::Next { ignore_phase_gate } => feature_next(ignore_phase_gate),
        FeatureCommands::New { title, phase } => feature_write_ops::feature_new(&title, phase),
        FeatureCommands::Link { id, adr, test, dep, yes } => {
            feature_write_ops::feature_link(&id, adr, test, dep, yes)
        }
        FeatureCommands::Status { id, new_status } => {
            feature_write_ops::feature_status(&id, &new_status)
        }
        FeatureCommands::Acknowledge { id, domain, adr, reason, remove } => {
            feature_write_ops::feature_acknowledge(&id, domain, adr, reason, remove)
        }
        FeatureCommands::Domain { id, add, remove } => {
            feature_write_ops::feature_domain(&id, add, remove)
        }
    }
}

fn feature_list(phase: Option<u32>, status: Option<String>, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let mut features: Vec<&types::Feature> = graph.features.values().collect();
    features.sort_by_key(|f| &f.front.id);

    if let Some(p) = phase {
        features.retain(|f| f.front.phase == p);
    }
    if let Some(ref s) = status {
        let target: types::FeatureStatus = s.parse().map_err(|e: String| ProductError::ConfigError(e))?;
        features.retain(|f| f.front.status == target);
    }

    if fmt == "json" {
        let arr: Vec<serde_json::Value> = features
            .iter()
            .map(|f| {
                serde_json::json!({
                    "id": f.front.id,
                    "phase": f.front.phase,
                    "status": f.front.status.to_string(),
                    "title": f.front.title,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else {
        println!("{:<10} {:<8} {:<15} TITLE", "ID", "PHASE", "STATUS");
        println!("{}", "-".repeat(60));
        for f in &features {
            println!(
                "{:<10} {:<8} {:<15} {}",
                f.front.id, f.front.phase, f.front.status, f.front.title
            );
        }
    }
    Ok(())
}

fn feature_show(id: &str, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
    if fmt == "json" {
        let obj = serde_json::json!({
            "id": f.front.id,
            "title": f.front.title,
            "phase": f.front.phase,
            "status": f.front.status.to_string(),
            "depends_on": f.front.depends_on,
            "adrs": f.front.adrs,
            "tests": f.front.tests,
            "body": f.body,
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else {
        println!("# {} — {}\n", f.front.id, f.front.title);
        println!("Phase:      {}", f.front.phase);
        println!("Status:     {}", f.front.status);
        println!("Depends-on: {}", if f.front.depends_on.is_empty() { "(none)".to_string() } else { f.front.depends_on.join(", ") });
        println!("ADRs:       {}", if f.front.adrs.is_empty() { "(none)".to_string() } else { f.front.adrs.join(", ") });
        println!("Tests:      {}", if f.front.tests.is_empty() { "(none)".to_string() } else { f.front.tests.join(", ") });
        println!("\n{}", f.body);
    }
    Ok(())
}

fn feature_adrs(id: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
    println!("ADRs linked to {}:", id);
    for adr_id in &f.front.adrs {
        if let Some(adr) = graph.adrs.get(adr_id.as_str()) {
            println!("  {} — {} ({})", adr.front.id, adr.front.title, adr.front.status);
        } else {
            println!("  {} (broken link)", adr_id);
        }
    }
    Ok(())
}

fn feature_tests(id: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
    println!("Tests linked to {}:", id);
    for test_id in &f.front.tests {
        if let Some(tc) = graph.tests.get(test_id.as_str()) {
            println!(
                "  {} — {} ({}, {})",
                tc.front.id, tc.front.title, tc.front.test_type, tc.front.status
            );
        } else {
            println!("  {} (broken link)", test_id);
        }
    }
    Ok(())
}

fn feature_deps(id: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let _f = graph
        .features
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
    println!("Dependency tree for {}:", id);
    print_dep_tree(&graph, id, 0, &mut std::collections::HashSet::new());
    Ok(())
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

fn print_dep_tree(
    graph: &graph::KnowledgeGraph,
    id: &str,
    indent: usize,
    visited: &mut std::collections::HashSet<String>,
) {
    if visited.contains(id) {
        println!("{}  {} (circular)", "  ".repeat(indent), id);
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
        println!("{}{} {} — {}", "  ".repeat(indent), marker, f.front.id, f.front.title);
        for dep in &f.front.depends_on {
            print_dep_tree(graph, dep, indent + 1, visited);
        }
    }
}
