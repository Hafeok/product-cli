//! Task-type (cell) inspection plus cross-validation commands.
//!
//! `product cell {validate,show,list,init}` operates on a §5 task-type
//! definition (a YAML file). `validate` cross-checks it against the captured
//! What graph (a cell's `domain:…` inputs must resolve to real entities) and
//! the blueprint's How contract (a cell's `applies` must name real patterns) —
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
    /// Dispatch a task type into concrete frozen work units (bind slots)
    Dispatch {
        #[arg(long)]
        file: Option<PathBuf>,
        /// Bind a slot to a value, repeatable: --bind entity=Order
        #[arg(long = "bind", value_name = "SLOT=VALUE")]
        binds: Vec<String>,
        /// Product whose What graph to bind/verify against (defaults to config)
        #[arg(long)]
        product: Option<String>,
        /// Directory to write the work units (default .product/work-units/)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Print the work units to stdout instead of writing files
        #[arg(long)]
        print: bool,
    },
    /// Scaffold a starter task-type definition
    Init {
        /// The task-type id (e.g. add-crud-resource)
        id: String,
        /// The home blueprint (defaults to the repo product name)
        #[arg(long)]
        blueprint: Option<String>,
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
        CellCommands::Dispatch { file, binds, product, out, print } => dispatch(file, binds, product, out, print),
        CellCommands::Init { id, blueprint, file, force } => init(id, blueprint, file, force),
    }
}

fn dispatch(file: Option<PathBuf>, binds: Vec<String>, product: Option<String>, out: Option<PathBuf>, print: bool) -> BoxResult {
    use product_core::pf::dispatch as disp;
    let task = load(file)?;
    let domain = load_domain(product.clone());
    let mut bindings = Vec::new();
    for b in &binds {
        let (k, val) = b.split_once('=').ok_or_else(|| format!("--bind expects SLOT=VALUE, got {b:?}"))?;
        bindings.push((k.trim().to_string(), val.trim().to_string()));
    }
    let result = disp::dispatch(&task, &bindings, domain.as_ref());

    for w in result.violations.iter().filter(|r| r.severity == "warning") {
        eprintln!("warning: [{}] {}: {}", w.focus, w.path, w.message);
    }
    if result.violations.iter().any(|r| r.severity == "violation") {
        let violations: Vec<_> = result.violations.iter().filter(|r| r.severity == "violation").collect();
        eprintln!("cannot dispatch — {} binding violation(s):", violations.len());
        for v in &violations {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} dispatch binding violation(s)", violations.len()).into());
    }
    write_work_units(&result.work_units, out, product, print)
}

fn write_work_units(work_units: &[product_core::pf::work_unit::WorkUnit], out: Option<PathBuf>, product: Option<String>, print: bool) -> BoxResult {
    if print {
        for wu in work_units {
            println!("# {}\n{}", wu.id, wu.to_yaml()?);
        }
        return Ok(());
    }
    let dir = out.unwrap_or_else(|| super::shared::artifact_dir(product.as_deref(), "work-units"));
    std::fs::create_dir_all(&dir)?;
    for wu in work_units {
        let path = dir.join(format!("{}.yaml", wu.id));
        std::fs::write(&path, wu.to_yaml()?)?;
        println!("Dispatched {} -> {}", wu.id, path.display());
    }
    println!("{} work unit(s) written to {}", work_units.len(), dir.display());
    Ok(())
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

/// Best-effort load of the blueprint's How contract for cross-validation.
fn load_how(product: Option<&str>) -> Option<HowContract> {
    let p = super::shared::artifact_dir(product, "").join("how-contract.yaml");
    std::fs::read_to_string(p).ok().and_then(|t| HowContract::from_yaml(&t).ok())
}

fn validate(file: Option<PathBuf>, product: Option<String>) -> BoxResult {
    let task = load(file)?;
    let how = load_how(product.as_deref());
    let domain = load_domain(product);
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
    if let Some(a) = &t.blueprint {
        println!("blueprint: {a}");
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

fn init(id: String, blueprint: Option<String>, file: Option<PathBuf>, force: bool) -> BoxResult {
    let p = path(file);
    if p.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", p.display()).into());
    }
    let arch = blueprint.or_else(super::shared::default_product_name).unwrap_or_else(|| "blueprint".to_string());
    let task = TaskType::scaffold(&id, &arch);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, task.to_yaml()?)?;
    println!("Scaffolded task type '{id}' (blueprint '{arch}') at {}", p.display());
    Ok(())
}
