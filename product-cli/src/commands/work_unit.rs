//! Work-unit (SPMC) inspection plus cross-validation commands.
//!
//! `product work-unit {validate,show,init}` operates on a §5 SPMC work unit (a
//! YAML file). `validate` cross-checks it against the captured What graph (a
//! `domain:X` input must resolve to a real entity) and the How contract
//! (`applies` must name real patterns; an applied principle should be
//! enforced).

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::how::HowContract;
use product_core::pf::how_validate::has_blocking;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use product_core::pf::work_unit::WorkUnit;
use product_core::pf::work_unit_validate::validate_work_unit;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum WorkUnitCommands {
    /// Scaffold a starter work-unit.yaml
    Init {
        /// The work-unit id (e.g. complete-task-handler)
        id: String,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// Show a summary of the work unit
    Show {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Validate the work unit against the What graph + How contract
    Validate {
        #[arg(long)]
        file: Option<PathBuf>,
        /// Product whose What graph to cross-check against (defaults to config)
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_work_unit(cmd: WorkUnitCommands) -> BoxResult {
    match cmd {
        WorkUnitCommands::Validate { file, product } => validate(file, product),
        WorkUnitCommands::Show { file } => show(file),
        WorkUnitCommands::Init { id, file, force } => init(id, file, force),
    }
}

fn path(file: Option<PathBuf>) -> PathBuf {
    file.unwrap_or_else(|| super::shared::domain_root().join(".product").join("work-unit.yaml"))
}

fn load(file: Option<PathBuf>) -> Result<WorkUnit, Box<dyn std::error::Error>> {
    let p = path(file);
    let text = std::fs::read_to_string(&p)
        .map_err(|_| format!("no work unit at {} — scaffold one with `product work-unit init <id>`", p.display()))?;
    Ok(WorkUnit::from_yaml(&text)?)
}

fn load_domain(product: Option<String>) -> Option<DomainGraph> {
    let p = product.or_else(super::shared::default_product_name)?;
    DomainSession::load(&session_dir(&super::shared::domain_root(), &p)).ok().map(|s| s.graph)
}

/// Discover the How contract to cross-check against. A dispatched unit lives
/// under `.product/archetypes/<name>/work-units/`, so prefer its archetype's
/// `how-contract.yaml`; otherwise fall back to `.product/how-contract.yaml`.
fn load_how(file: &Option<PathBuf>) -> Option<HowContract> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(f) = file {
        if f.parent().and_then(|p| p.file_name()) == Some(std::ffi::OsStr::new("work-units")) {
            if let Some(arch) = f.parent().and_then(|p| p.parent()) {
                candidates.push(arch.join("how-contract.yaml"));
            }
        }
    }
    candidates.push(super::shared::domain_root().join(".product").join("how-contract.yaml"));
    candidates.into_iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| HowContract::from_yaml(&t).ok())
}

fn validate(file: Option<PathBuf>, product: Option<String>) -> BoxResult {
    let wu = load(file.clone())?;
    let domain = load_domain(product);
    let how = load_how(&file);
    let results = validate_work_unit(&wu, domain.as_ref(), how.as_ref());

    for w in results.iter().filter(|r| r.severity == "warning") {
        eprintln!("warning: [{}] {}: {}", w.focus, w.path, w.message);
    }
    if has_blocking(&results) {
        let violations: Vec<_> = results.iter().filter(|r| r.severity == "violation").collect();
        eprintln!("non-conformant — {} violation(s):", violations.len());
        for v in &violations {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} work-unit conformance violation(s)", violations.len()).into());
    }
    println!(
        "conformant — work unit '{}' produces {} [domain: {}, how: {}]",
        wu.id,
        wu.produces.artifact,
        if domain.is_some() { "cross-checked" } else { "not loaded" },
        if how.is_some() { "cross-checked" } else { "not loaded" },
    );
    Ok(())
}

fn show(file: Option<PathBuf>) -> BoxResult {
    let w = load(file)?;
    println!("work-unit: {}", w.id);
    println!("produces:  {}", w.produces.artifact);
    if let Some(m) = &w.model {
        println!("model:     {m}");
    }
    println!("context (frozen={}):", w.context.frozen);
    for d in &w.context.derived_from {
        println!("  - {d}");
    }
    if !w.applies.is_empty() {
        println!("applies: {}", w.applies.join(", "));
    }
    Ok(())
}

fn init(id: String, file: Option<PathBuf>, force: bool) -> BoxResult {
    let p = path(file);
    if p.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", p.display()).into());
    }
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, WorkUnit::scaffold(&id).to_yaml()?)?;
    println!("Scaffolded work unit '{id}' at {}", p.display());
    Ok(())
}
