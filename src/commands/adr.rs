//! ADR navigation, creation, status, review, amendment, sealing.

use clap::Subcommand;
use product_lib::{author, error::ProductError, fileops, hash, parser, types};

use super::{acquire_write_lock, load_graph, BoxResult};
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
}

pub(crate) fn handle_adr(cmd: AdrCommands, fmt: &str) -> BoxResult {
    match cmd {
        AdrCommands::List { status } => adr_list(status, fmt),
        AdrCommands::Show { id } => adr_show(&id, fmt),
        AdrCommands::Features { id } => adr_features(&id),
        AdrCommands::Tests { id } => adr_tests(&id),
        AdrCommands::New { title } => adr_new(&title),
        AdrCommands::Status { id, new_status, by } => adr_status(&id, &new_status, by),
        AdrCommands::Review { staged } => adr_review(staged),
        AdrCommands::Amend { id, reason } => adr_amend(&id, reason),
        AdrCommands::Rehash { id, all } => adr_rehash(id, all),
        AdrCommands::Domain { id, add, remove } => adr_write_ops::adr_domain(&id, add, remove),
        AdrCommands::Scope { id, scope } => adr_write_ops::adr_scope(&id, &scope),
        AdrCommands::Supersede { id, supersedes, remove } => adr_write_ops::adr_supersede(&id, supersedes, remove),
        AdrCommands::SourceFiles { id, add, remove } => adr_write_ops::adr_source_files(&id, add, remove),
    }
}

fn adr_list(status: Option<String>, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let mut adrs: Vec<&types::Adr> = graph.adrs.values().collect();
    adrs.sort_by_key(|a| &a.front.id);

    if let Some(ref s) = status {
        let target: types::AdrStatus = s.parse().map_err(|e: String| ProductError::ConfigError(e))?;
        adrs.retain(|a| a.front.status == target);
    }

    if fmt == "json" {
        let arr: Vec<serde_json::Value> = adrs
            .iter()
            .map(|a| {
                serde_json::json!({
                    "id": a.front.id,
                    "status": a.front.status.to_string(),
                    "title": a.front.title,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else {
        println!("{:<10} {:<15} TITLE", "ID", "STATUS");
        println!("{}", "-".repeat(60));
        for a in &adrs {
            println!("{:<10} {:<15} {}", a.front.id, a.front.status, a.front.title);
        }
    }
    Ok(())
}

fn adr_show(id: &str, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;
    if fmt == "json" {
        let obj = serde_json::json!({
            "id": a.front.id,
            "title": a.front.title,
            "status": a.front.status.to_string(),
            "features": a.front.features,
            "supersedes": a.front.supersedes,
            "superseded_by": a.front.superseded_by,
            "body": a.body,
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else {
        println!("# {} — {}\n", a.front.id, a.front.title);
        println!("Status:        {}", a.front.status);
        println!("Features:      {}", if a.front.features.is_empty() { "(none)".to_string() } else { a.front.features.join(", ") });
        println!("Supersedes:    {}", if a.front.supersedes.is_empty() { "(none)".to_string() } else { a.front.supersedes.join(", ") });
        println!("Superseded-by: {}", if a.front.superseded_by.is_empty() { "(none)".to_string() } else { a.front.superseded_by.join(", ") });
        println!("\n{}", a.body);
    }
    Ok(())
}

fn adr_features(id: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    println!("Features referencing {}:", id);
    let id_string = id.to_string();
    for f in graph.features.values() {
        if f.front.adrs.contains(&id_string) {
            println!("  {} — {} ({})", f.front.id, f.front.title, f.front.status);
        }
    }
    Ok(())
}

fn adr_tests(id: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    println!("Tests validating {}:", id);
    let id_string = id.to_string();
    for t in graph.tests.values() {
        if t.front.validates.adrs.contains(&id_string) {
            println!(
                "  {} — {} ({}, {})",
                t.front.id, t.front.title, t.front.test_type, t.front.status
            );
        }
    }
    Ok(())
}

fn adr_new(title: &str) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, root, graph) = load_graph()?;
    let existing: Vec<String> = graph.adrs.keys().cloned().collect();
    let id = parser::next_id(&config.prefixes.adr, &existing);
    let filename = parser::id_to_filename(&id, title);
    let dir = config.resolve_path(&root, &config.paths.adrs);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(&filename);

    let front = types::AdrFrontMatter {
        id: id.clone(),
        title: title.to_string(),
        status: types::AdrStatus::Proposed,
        features: vec![],
        supersedes: vec![],
        superseded_by: vec![],
        domains: vec![],
        scope: types::AdrScope::Domain,
        content_hash: None,
        amendments: vec![],
        source_files: vec![],
    };
    let body = "**Status:** Proposed\n\n**Context:**\n\n[Describe the context here.]\n\n**Decision:**\n\n[Describe the decision.]\n\n**Rationale:**\n\n[Explain why.]\n\n**Rejected alternatives:**\n\n- [Alternative 1]\n".to_string();
    let content = parser::render_adr(&front, &body);
    fileops::write_file_atomic(&path, &content)?;
    println!("Created: {} at {}", id, path.display());
    Ok(())
}

fn adr_status(id: &str, new_status: &str, by: Option<String>) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;

    let status: types::AdrStatus = new_status
        .parse()
        .map_err(|e: String| ProductError::ConfigError(e))?;

    // If superseding, show impact first
    if status == types::AdrStatus::Superseded {
        let impact = graph.impact(id);
        impact.print(&graph);
        println!();
    }

    let mut front = a.front.clone();
    front.status = status;

    // Compute content-hash on acceptance (ADR-032)
    if status == types::AdrStatus::Accepted {
        let h = hash::compute_adr_hash(&front.title, &a.body);
        front.content_hash = Some(h);
    }

    if let Some(by_id) = by {
        update_supersession(&mut front, id, &by_id, &graph)?;
    }

    let content = parser::render_adr(&front, &a.body);
    fileops::write_file_atomic(&a.path, &content)?;
    println!("{} status -> {}", id, status);
    Ok(())
}

fn update_supersession(
    front: &mut types::AdrFrontMatter,
    id: &str,
    by_id: &str,
    graph: &product_lib::graph::KnowledgeGraph,
) -> BoxResult {
    if !front.superseded_by.contains(&by_id.to_string()) {
        front.superseded_by.push(by_id.to_string());
    }
    // Also update the successor to list this in supersedes
    if let Some(successor) = graph.adrs.get(by_id) {
        let mut succ_front = successor.front.clone();
        if !succ_front.supersedes.contains(&id.to_string()) {
            succ_front.supersedes.push(id.to_string());
        }
        let succ_content = parser::render_adr(&succ_front, &successor.body);
        fileops::write_file_atomic(&successor.path, &succ_content)?;
    }
    Ok(())
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

fn adr_amend(id: &str, reason: Option<String>) -> BoxResult {
    let reason = reason.ok_or_else(|| {
        ProductError::ConfigError("--reason is required for amendments".to_string())
    })?;
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;

    let (new_hash, amendment) = hash::amend_adr(a, &reason)?;

    let mut front = a.front.clone();
    front.content_hash = Some(new_hash.clone());
    front.amendments.push(amendment);

    let content = parser::render_adr(&front, &a.body);
    fileops::write_file_atomic(&a.path, &content)?;
    println!("{} amended: content-hash updated to {}", id, new_hash);
    Ok(())
}

fn adr_rehash(id: Option<String>, all: bool) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;

    if all {
        rehash_all(&graph)?;
    } else {
        rehash_single(id, &graph)?;
    }
    Ok(())
}

fn rehash_all(graph: &product_lib::graph::KnowledgeGraph) -> BoxResult {
    let mut sealed = 0;
    let mut skipped = 0;
    let mut adrs: Vec<&types::Adr> = graph.adrs.values().collect();
    adrs.sort_by_key(|a| &a.front.id);
    for a in adrs {
        if a.front.status != types::AdrStatus::Accepted {
            continue;
        }
        if a.front.content_hash.is_some() {
            skipped += 1;
            continue;
        }
        let h = hash::seal_adr(a)?;
        let mut front = a.front.clone();
        front.content_hash = Some(h.clone());
        let content = parser::render_adr(&front, &a.body);
        fileops::write_file_atomic(&a.path, &content)?;
        println!("  sealed {} -> {}", a.front.id, h);
        sealed += 1;
    }
    println!("{} ADR(s) sealed, {} already sealed", sealed, skipped);
    Ok(())
}

fn rehash_single(id: Option<String>, graph: &product_lib::graph::KnowledgeGraph) -> BoxResult {
    let adr_id = id.ok_or_else(|| {
        ProductError::ConfigError(
            "specify an ADR ID or use --all".to_string(),
        )
    })?;
    let a = graph
        .adrs
        .get(&adr_id)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;
    if a.front.content_hash.is_some() {
        println!("{} is already sealed", adr_id);
        return Ok(());
    }
    let h = hash::seal_adr(a)?;
    let mut front = a.front.clone();
    front.content_hash = Some(h.clone());
    let content = parser::render_adr(&front, &a.body);
    fileops::write_file_atomic(&a.path, &content)?;
    println!("{} sealed: content-hash = {}", adr_id, h);
    Ok(())
}

