//! CLI adapters for the interactive request builder (FT-052, ADR-044).
//!
//! Each adapter loads the graph, opens (or creates) the draft, calls into
//! the pure `request::builder` slice, and renders the result. The `add`
//! subcommand dispatch lives in `request_builder_add.rs` to keep this file
//! under the 400-line cap.

use clap::Subcommand;
use product_lib::config::ProductConfig;
use product_lib::graph::KnowledgeGraph;
use product_lib::parser;
use product_lib::request::builder::{
    self, draft::Draft, draft::DraftKind, render, submit,
};

use super::request_builder_add::{self, AddCommands};
use super::BoxResult;

#[derive(Subcommand)]
pub enum BuilderCommands {
    /// Open a new interactive draft (kind = create or change)
    New {
        /// Draft kind: "create" or "change"
        kind: String,
    },
    /// Resume editing the existing draft
    Continue,
    /// Remove the active draft
    Discard {
        /// Skip the confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Show a summary of the draft with per-artifact indicators
    Status,
    /// Print the raw draft YAML
    Show,
    /// Apply the draft atomically and archive it on success
    Submit {
        /// Submit through warnings without prompting
        #[arg(long)]
        force: bool,
    },
    /// Open the draft in `$EDITOR`
    Edit,
    /// Append one artifact or mutation to the draft
    Add {
        #[command(subcommand)]
        command: AddCommands,
    },
}

pub(crate) fn handle_builder(cmd: BuilderCommands) -> BoxResult {
    match cmd {
        BuilderCommands::New { kind } => cmd_new(&kind),
        BuilderCommands::Continue => cmd_continue(),
        BuilderCommands::Discard { force } => cmd_discard(force),
        BuilderCommands::Status => cmd_status(),
        BuilderCommands::Show => cmd_show(),
        BuilderCommands::Submit { force } => cmd_submit(force),
        BuilderCommands::Edit => cmd_edit(),
        BuilderCommands::Add { command } => request_builder_add::handle_add(command),
    }
}

fn cmd_new(kind: &str) -> BoxResult {
    let k = DraftKind::parse(kind).ok_or_else(|| {
        format!("unknown draft kind '{kind}' — expected 'create' or 'change'")
    })?;
    let (_config, root) = ProductConfig::discover()?;
    if Draft::exists(&root) {
        println!(
            "Active draft already exists at {}",
            Draft::draft_path_str(&root)
        );
        println!("Options:");
        println!("  product request status     — show the draft");
        println!("  product request submit     — apply the draft");
        println!("  product request discard    — delete the draft");
        println!("  product request continue   — resume (open in $EDITOR)");
        return Ok(());
    }
    let draft = Draft::new(&root, k);
    draft.save()?;
    println!(
        "Draft started at {} (type: {}).",
        draft.path.display(),
        k.as_str()
    );
    println!();
    println!("Next-step commands:");
    if k == DraftKind::Create {
        println!("  product request add feature|adr|tc|dep|doc …");
    } else {
        println!("  product request add target ID   — add a change block");
        println!("  product request add acknowledgement ID DOMAIN REASON");
    }
    println!("  product request status           — review findings");
    println!("  product request validate         — run full validation");
    println!("  product request submit           — apply & archive");
    println!("  product request discard          — delete the draft");
    Ok(())
}

fn cmd_continue() -> BoxResult {
    let (_config, root) = ProductConfig::discover()?;
    if !Draft::exists(&root) {
        println!("no active draft — start one with `product request new create`");
        return Ok(());
    }
    println!("Active draft: {}", Draft::draft_path_str(&root));
    Ok(())
}

fn cmd_discard(force: bool) -> BoxResult {
    let (_config, root) = ProductConfig::discover()?;
    if !Draft::exists(&root) {
        println!("no active draft to discard");
        return Ok(());
    }
    if !force && std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        use std::io::Write;
        print!("Discard draft at {} ? [y/N] ", Draft::draft_path_str(&root));
        std::io::stdout().flush()?;
        let mut reply = String::new();
        std::io::stdin().read_line(&mut reply)?;
        if !reply.trim().eq_ignore_ascii_case("y") {
            println!("aborted — draft left in place");
            return Ok(());
        }
    }
    Draft::delete(&root)?;
    println!("Draft discarded.");
    Ok(())
}

fn cmd_status() -> BoxResult {
    let (config, root) = ProductConfig::discover()?;
    match Draft::load(&root) {
        None => {
            println!("no active draft");
            Ok(())
        }
        Some(Ok(draft)) => {
            let graph = build_graph(&config, &root)?;
            let report = builder::status_report(&draft, &config, &graph);
            print!("{}", render::render_status(&report));
            Ok(())
        }
        Some(Err(e)) => Err(format!("failed to load draft: {e}").into()),
    }
}

fn cmd_show() -> BoxResult {
    let (_config, root) = ProductConfig::discover()?;
    match Draft::load(&root) {
        None => {
            println!("no active draft");
            Ok(())
        }
        Some(Ok(draft)) => {
            print!("{}", draft.to_yaml());
            Ok(())
        }
        Some(Err(e)) => Err(format!("failed to load draft: {e}").into()),
    }
}

fn cmd_submit(force: bool) -> BoxResult {
    let (config, root) = ProductConfig::discover()?;
    let draft = require_draft(&root)?;
    let graph = build_graph(&config, &root)?;
    let policy = submit::WarnPolicy::parse(&config.request_builder.warn_on_warnings)
        .unwrap_or(submit::WarnPolicy::Warn);
    let _lock = product_lib::fileops::RepoLock::acquire(&root)?;
    let outcome = submit::submit(
        &draft,
        &config,
        &graph,
        &root,
        submit::SubmitOptions { force },
        policy,
    );
    if !outcome.result.applied {
        for f in &outcome.result.findings {
            eprintln!("{f}\n");
        }
        if let Some(reason) = &outcome.blocked_reason {
            eprintln!("submit blocked — {reason}");
        }
        std::process::exit(1);
    }
    println!("Applied:");
    for c in &outcome.result.created {
        println!("  {}  (new) -> {}", c.id, c.file);
    }
    for c in &outcome.result.changed {
        println!("  {}  ({} mutation(s)) -> {}", c.id, c.mutations, c.file);
    }
    if let Some(p) = &outcome.archived_to {
        println!("Archived draft -> {}", p.display());
    }
    for f in outcome.result.findings.iter().filter(|f| !f.is_error()) {
        eprintln!("{f}\n");
    }
    Ok(())
}

fn cmd_edit() -> BoxResult {
    let (config, root) = ProductConfig::discover()?;
    let draft = require_draft(&root)?;
    let editor = config
        .request_builder
        .editor
        .clone()
        .or_else(|| std::env::var("EDITOR").ok())
        .ok_or("no editor configured — set $EDITOR or [request-builder].editor")?;
    let status = std::process::Command::new(editor).arg(&draft.path).status()?;
    if !status.success() {
        return Err("editor exited with non-zero status".into());
    }
    Ok(())
}

fn require_draft(root: &std::path::Path) -> Result<Draft, Box<dyn std::error::Error>> {
    match Draft::load(root) {
        None => Err("no active draft — run `product request new create|change`".into()),
        Some(Ok(d)) => Ok(d),
        Some(Err(e)) => Err(format!("failed to load draft: {e}").into()),
    }
}

fn build_graph(
    config: &ProductConfig,
    root: &std::path::Path,
) -> Result<KnowledgeGraph, Box<dyn std::error::Error>> {
    let features_dir = config.resolve_path(root, &config.paths.features);
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let tests_dir = config.resolve_path(root, &config.paths.tests);
    let deps_dir = config.resolve_path(root, &config.paths.dependencies);
    let loaded =
        parser::load_all_with_deps(&features_dir, &adrs_dir, &tests_dir, Some(&deps_dir))?;
    Ok(KnowledgeGraph::build_with_deps(
        loaded.features, loaded.adrs, loaded.tests, loaded.dependencies,
    ))
}
