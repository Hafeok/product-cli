//! ADR navigation, creation, status, review, amendment, sealing.

use clap::Subcommand;
use product_lib::{adr as adr_slice, author, error::ProductError, types};

use super::{acquire_write_lock_typed, load_graph, load_graph_typed, BoxResult, CmdResult, Output};
mod adr_write_ops {
    pub(crate) use super::super::adr_write::*;
}

#[derive(Subcommand)]
pub enum AdrCommands {
    /// List all ADRs
    List {
        #[arg(long)]
        status: Option<String>,
    },
    /// Show an ADR's details
    Show { id: String },
    /// List features that reference this ADR
    Features { id: String },
    /// List tests that validate this ADR
    Tests { id: String },
    /// Create a new ADR file
    New {
        /// ADR title
        title: String,
    },
    /// Set ADR status
    Status {
        /// ADR ID
        id: String,
        /// New status: proposed, accepted, superseded, abandoned
        new_status: String,
        /// When setting to superseded, specify the replacement ADR
        #[arg(long)]
        by: Option<String>,
    },
    /// Review staged ADR files
    Review {
        /// Only review staged files (for pre-commit hook)
        #[arg(long)]
        staged: bool,
    },
    /// Record a legitimate amendment to an accepted ADR (ADR-032)
    Amend {
        /// ADR ID
        id: String,
        /// Reason for the amendment (mandatory)
        #[arg(long)]
        reason: Option<String>,
    },
    /// Seal an accepted ADR that predates content-hash (ADR-032)
    Rehash {
        /// ADR ID (omit with --all to seal all)
        id: Option<String>,
        /// Seal all accepted ADRs without content-hash
        #[arg(long)]
        all: bool,
    },
    /// Add or remove concern domains on an ADR
    Domain {
        /// ADR ID
        id: String,
        /// Domain to add (repeatable)
        #[arg(long)]
        add: Vec<String>,
        /// Domain to remove (repeatable)
        #[arg(long)]
        remove: Vec<String>,
    },
    /// Set ADR scope
    Scope {
        /// ADR ID
        id: String,
        /// Scope value: cross-cutting, domain, feature-specific
        scope: String,
    },
    /// Manage ADR supersession (bidirectional write)
    Supersede {
        /// ADR ID (the newer ADR)
        id: String,
        /// ADR that this ADR supersedes
        #[arg(long)]
        supersedes: Option<String>,
        /// Remove supersession link to this ADR
        #[arg(long)]
        remove: Option<String>,
    },
    /// Add or remove governed source files on an ADR
    #[command(name = "source-files")]
    SourceFiles {
        /// ADR ID
        id: String,
        /// Source file/directory to add (repeatable)
        #[arg(long)]
        add: Vec<String>,
        /// Source file/directory to remove (repeatable)
        #[arg(long)]
        remove: Vec<String>,
    },
    /// Structural conflict check — cycles, symmetry, domain overlap (ADR-040)
    #[command(name = "check-conflicts")]
    CheckConflicts {
        /// ADR ID to check (omit to check every ADR)
        id: Option<String>,
    },
    /// Emit an LLM-ready conflict-check bundle on stdout (ADR-040)
    #[command(name = "conflict-bundle")]
    ConflictBundle {
        /// ADR ID
        id: String,
        /// Output format: markdown (default) or json
        #[arg(long, default_value = "markdown")]
        format: String,
    },
}

pub(crate) fn handle_adr(cmd: AdrCommands, fmt: &str) -> BoxResult {
    match cmd {
        AdrCommands::List { status } => super::render(adr_list(status), fmt),
        AdrCommands::Show { id } => super::render(adr_show(&id), fmt),
        AdrCommands::Features { id } => super::render(adr_features(&id), fmt),
        AdrCommands::Tests { id } => super::render(adr_tests(&id), fmt),
        AdrCommands::New { title } => super::render(adr_new(&title), fmt),
        AdrCommands::Status { id, new_status, by } => {
            super::render(adr_status(&id, &new_status, by), fmt)
        }
        AdrCommands::Review { staged } => adr_review(staged),
        AdrCommands::Amend { id, reason } => super::render(super::adr_seal::adr_amend(&id, reason), fmt),
        AdrCommands::Rehash { id, all } => super::render(super::adr_seal::adr_rehash(id, all), fmt),
        AdrCommands::Domain { id, add, remove } => {
            super::render(adr_write_ops::adr_domain(&id, add, remove), fmt)
        }
        AdrCommands::Scope { id, scope } => super::render(adr_write_ops::adr_scope(&id, &scope), fmt),
        AdrCommands::Supersede { id, supersedes, remove } => {
            super::render(adr_write_ops::adr_supersede(&id, supersedes, remove), fmt)
        }
        AdrCommands::SourceFiles { id, add, remove } => {
            super::render(adr_write_ops::adr_source_files(&id, add, remove), fmt)
        }
        AdrCommands::CheckConflicts { id } => {
            super::render(super::adr_conflicts::adr_check_conflicts(id, fmt), fmt)
        }
        AdrCommands::ConflictBundle { id, format } => super::adr_conflicts::adr_conflict_bundle(&id, &format),
    }
}

fn adr_list(status: Option<String>) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let mut adrs: Vec<&types::Adr> = graph.adrs.values().collect();
    adrs.sort_by_key(|a| &a.front.id);
    if let Some(ref s) = status {
        let target: types::AdrStatus = s.parse().map_err(ProductError::ConfigError)?;
        adrs.retain(|a| a.front.status == target);
    }
    let json = serde_json::Value::Array(
        adrs.iter()
            .map(|a| {
                serde_json::json!({
                    "id": a.front.id,
                    "status": a.front.status.to_string(),
                    "title": a.front.title,
                })
            })
            .collect(),
    );
    let mut text = format!("{:<10} {:<15} TITLE\n", "ID", "STATUS");
    text.push_str(&"-".repeat(60));
    text.push('\n');
    for a in &adrs {
        text.push_str(&format!(
            "{:<10} {:<15} {}\n",
            a.front.id, a.front.status, a.front.title
        ));
    }
    Ok(Output::both(text, json))
}

fn adr_show(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;
    let json = serde_json::json!({
        "id": a.front.id,
        "title": a.front.title,
        "status": a.front.status.to_string(),
        "features": a.front.features,
        "supersedes": a.front.supersedes,
        "superseded_by": a.front.superseded_by,
        "body": a.body,
    });
    let text = render_adr_show_text(a);
    Ok(Output::both(text, json))
}

fn render_adr_show_text(a: &types::Adr) -> String {
    let mut out = format!("# {} — {}\n\n", a.front.id, a.front.title);
    out.push_str(&format!("Status:        {}\n", a.front.status));
    out.push_str(&format!(
        "Features:      {}\n",
        if a.front.features.is_empty() { "(none)".to_string() } else { a.front.features.join(", ") }
    ));
    out.push_str(&format!(
        "Supersedes:    {}\n",
        if a.front.supersedes.is_empty() { "(none)".to_string() } else { a.front.supersedes.join(", ") }
    ));
    out.push_str(&format!(
        "Superseded-by: {}\n",
        if a.front.superseded_by.is_empty() { "(none)".to_string() } else { a.front.superseded_by.join(", ") }
    ));
    out.push_str(&format!("\n{}", a.body));
    out
}

fn adr_features(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let id_string = id.to_string();
    let mut text = format!("Features referencing {}:\n", id);
    let mut arr: Vec<serde_json::Value> = Vec::new();
    for f in graph.features.values() {
        if f.front.adrs.contains(&id_string) {
            text.push_str(&format!(
                "  {} — {} ({})\n",
                f.front.id, f.front.title, f.front.status
            ));
            arr.push(serde_json::json!({
                "id": f.front.id,
                "title": f.front.title,
                "status": f.front.status.to_string(),
            }));
        }
    }
    Ok(Output::both(text, serde_json::Value::Array(arr)))
}

fn adr_tests(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let id_string = id.to_string();
    let mut text = format!("Tests validating {}:\n", id);
    let mut arr: Vec<serde_json::Value> = Vec::new();
    for t in graph.tests.values() {
        if t.front.validates.adrs.contains(&id_string) {
            text.push_str(&format!(
                "  {} — {} ({}, {})\n",
                t.front.id, t.front.title, t.front.test_type, t.front.status
            ));
            arr.push(serde_json::json!({
                "id": t.front.id,
                "title": t.front.title,
                "type": t.front.test_type.to_string(),
                "status": t.front.status.to_string(),
            }));
        }
    }
    Ok(Output::both(text, serde_json::Value::Array(arr)))
}

fn adr_new(title: &str) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (config, root, graph) = load_graph_typed()?;
    let existing: Vec<String> = graph.adrs.keys().cloned().collect();
    let plan = adr_slice::plan_create(title, &existing, &config.prefixes.adr)?;
    let target_dir = config.resolve_path(&root, &config.paths.adrs);
    let path = adr_slice::apply_create(&plan, &target_dir)?;
    Ok(Output::text(format!("Created: {} at {}", plan.id, path.display())))
}

fn adr_status(id: &str, new_status: &str, by: Option<String>) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (_, _, graph) = load_graph_typed()?;
    let status: types::AdrStatus = new_status
        .parse()
        .map_err(ProductError::ConfigError)?;

    // Preserve the legacy behaviour of printing impact before a supersession
    // status change. This prints to stdout directly — a future refactor
    // could move impact rendering into a pure formatter and return it as
    // part of the Output value.
    if status == types::AdrStatus::Superseded {
        let impact = graph.impact(id);
        impact.print(&graph);
        println!();
    }

    let plan = adr_slice::plan_status_change(&graph, id, status, by.as_deref())?;
    adr_slice::apply_status_change(&plan)?;
    Ok(Output::text(format!("{} status -> {}", id, plan.new_status)))
}

fn adr_review(staged: bool) -> BoxResult {
    if staged {
        let (_, root, _) = load_graph()?;
        let warnings = author::review_staged(&root)?;
        for w in &warnings {
            eprintln!("{}", w);
        }
        if !warnings.is_empty() {
            eprintln!("{} ADR review warning(s)", warnings.len());
        }
    } else {
        eprintln!("Use --staged to review staged ADR files.");
    }
    Ok(())
}


