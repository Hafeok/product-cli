//! Task-type (cell) inspection plus cross-validation commands.
//!
//! `product cell {validate,show,list,init}` operates on a §5 task-type
//! definition (a YAML file). `validate` cross-checks it against the captured
//! What graph (a cell's `domain:…` inputs must resolve to real entities) and
//! the archetype's How contract (a cell's `applies` must name real patterns) —
//! so cells are built from the domain model, not free-floating.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::cell::TaskType;
use product_core::pf::cell_validate::validate_cell;
use product_core::pf::how::HowContract;
use product_core::pf::how_validate::has_blocking;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum CellCommands {
    /// Scaffold a starter task-type definition
    Init {
        /// The task-type id (e.g. add-crud-resource)
        id: String,
        /// The home archetype (defaults to the repo product name)
        #[arg(long)]
        archetype: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// List items of a kind: slots, cells, audits
    List {
        /// One of: slots, cells, audits
        kind: String,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Show a summary of the task-type definition
    Show {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Validate the task type against the What graph + How contract
    Validate {
        #[arg(long)]
        file: Option<PathBuf>,
        /// Product whose What graph to cross-check against (defaults to config)
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_cell(cmd: CellCommands) -> BoxResult {
    match cmd {
        CellCommands::Validate { file, product } => validate(file, product),
        CellCommands::Show { file } => show(file),
        CellCommands::List { kind, file } => list(kind, file),
        CellCommands::Init { id, archetype, file, force } => init(id, archetype, file, force),
    }
}

fn path(file: Option<PathBuf>) -> PathBuf {
    file.unwrap_or_else(|| super::shared::domain_root().join(".product").join("cell.yaml"))
}

fn load(file: Option<PathBuf>) -> Result<TaskType, Box<dyn std::error::Error>> {
    let p = path(file);
    let text = std::fs::read_to_string(&p)
        .map_err(|_| format!("no task type at {} — scaffold one with `product cell init <id>`", p.display()))?;
    Ok(TaskType::from_yaml(&text)?)
}

/// Best-effort load of the captured What graph for cross-validation.
fn load_domain(product: Option<String>) -> Option<DomainGraph> {
    let p = product.or_else(super::shared::default_product_name)?;
    let dir = session_dir(&super::shared::domain_root(), &p);
    DomainSession::load(&dir).ok().map(|s| s.graph)
}

/// Best-effort load of the archetype's How contract for cross-validation.
fn load_how() -> Option<HowContract> {
    let p = super::shared::domain_root().join(".product").join("how-contract.yaml");
    std::fs::read_to_string(p).ok().and_then(|t| HowContract::from_yaml(&t).ok())
}

fn validate(file: Option<PathBuf>, product: Option<String>) -> BoxResult {
    let task = load(file)?;
    let domain = load_domain(product);
    let how = load_how();
    let results = validate_cell(&task, domain.as_ref(), how.as_ref());

    for w in results.iter().filter(|r| r.severity == "warning") {
        eprintln!("warning: [{}] {}: {}", w.focus, w.path, w.message);
    }
    if has_blocking(&results) {
        let violations: Vec<_> = results.iter().filter(|r| r.severity == "violation").collect();
        eprintln!("non-conformant — {} violation(s):", violations.len());
        for v in &violations {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} cell conformance violation(s)", violations.len()).into());
    }
    println!(
        "conformant — {} slot(s), {} cell(s), {} audit(s) [domain: {}, how: {}]",
        task.slots.len(),
        task.cells.len(),
        task.audits.len(),
        if domain.is_some() { "cross-checked" } else { "not loaded" },
        if how.is_some() { "cross-checked" } else { "not loaded" },
    );
    Ok(())
}

fn show(file: Option<PathBuf>) -> BoxResult {
    let t = load(file)?;
    println!("task-type: {} — {}", t.id, t.name);
    if let Some(a) = &t.archetype {
        println!("archetype: {a}");
    }
    if let Some(c) = &t.classification {
        println!("classification: {c}");
    }
    println!("applies-when: {}", t.applies_when);
    println!("slots:  {}", t.slots.len());
    println!("cells:  {}", t.cells.len());
    println!("audits: {}", t.audits.len());
    Ok(())
}

fn list(kind: String, file: Option<PathBuf>) -> BoxResult {
    let t = load(file)?;
    let rows: Vec<(String, String)> = match kind.as_str() {
        "slots" | "slot" => t.slots.iter().map(|s| (s.name.clone(), s.capture.clone())).collect(),
        "cells" | "cell" => t.cells.iter().map(|c| (c.id.clone(), format!("{} <- [{}]", c.artifact, c.derived_from.join(", ")))).collect(),
        "audits" | "audit" => t.audits.iter().map(|a| (a.id.clone(), format!("{} (protects {})", a.checks, a.protects))).collect(),
        other => return Err(format!("unknown kind {other:?} — use slots, cells, or audits").into()),
    };
    if rows.is_empty() {
        println!("(none)");
        return Ok(());
    }
    let w = rows.iter().map(|r| r.0.len()).max().unwrap_or(2);
    for (id, desc) in rows {
        println!("{id:<w$}  {desc}");
    }
    Ok(())
}

fn init(id: String, archetype: Option<String>, file: Option<PathBuf>, force: bool) -> BoxResult {
    let p = path(file);
    if p.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", p.display()).into());
    }
    let arch = archetype.or_else(super::shared::default_product_name).unwrap_or_else(|| "archetype".to_string());
    let task = TaskType::scaffold(&id, &arch);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, task.to_yaml()?)?;
    println!("Scaffolded task type '{id}' (archetype '{arch}') at {}", p.display());
    Ok(())
}
