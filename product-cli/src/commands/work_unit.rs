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
/// under `.product/blueprints/<name>/work-units/`, so prefer its blueprint's
/// `how-contract.yaml`; otherwise fall back to `.product/how-contract.yaml`.
fn load_how(file: &Option<PathBuf>, product: Option<&str>) -> Option<HowContract> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(f) = file {
        if f.parent().and_then(|p| p.file_name()) == Some(std::ffi::OsStr::new("work-units")) {
            if let Some(arch) = f.parent().and_then(|p| p.parent()) {
                candidates.push(arch.join("how-contract.yaml"));
            }
        }
    }
    candidates.push(super::shared::artifact_dir(product, "").join("how-contract.yaml"));
    candidates.into_iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| HowContract::from_yaml(&t).ok())
}

/// The files `validate` covers: an explicit `--file`, or else the singleton
/// `.product/work-unit.yaml` (if present) plus the dispatched fleet under
/// `.product/work-units/*.yaml` — the units `product build` actually runs.
fn validate_targets(file: Option<PathBuf>) -> Vec<PathBuf> {
    if let Some(f) = file {
        return vec![f];
    }
    let root = super::shared::domain_root().join(".product");
    let mut out = Vec::new();
    let singleton = root.join("work-unit.yaml");
    if singleton.exists() {
        out.push(singleton);
    }
    if let Ok(entries) = std::fs::read_dir(root.join("work-units")) {
        let mut fleet: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|x| x == "yaml" || x == "yml").unwrap_or(false))
            .collect();
        fleet.sort();
        out.extend(fleet);
    }
    out
}

/// Validate one work-unit file; print its warnings/violations and return the
/// violation count. Beyond the core checks, a `produces.path` that resolves to
/// an existing directory under `repo_root` is a violation — a worker echoes
/// this path as a write target, and a directory fails at dispatch.
fn validate_one(target: &PathBuf, domain: Option<&DomainGraph>, product: Option<&str>, repo_root: &PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(target)
        .map_err(|e| format!("cannot read {}: {e}", target.display()))?;
    let wu = WorkUnit::from_yaml(&text)?;
    let how = load_how(&Some(target.clone()), product);
    let mut results = validate_work_unit(&wu, domain, how.as_ref());
    let produced = wu.produces.path.trim();
    if !produced.is_empty() && repo_root.join(produced).is_dir() {
        results.push(product_core::pf::validate::Violation {
            focus: wu.id.clone(),
            path: "produces.path".to_string(),
            message: format!("§5 produces.path '{produced}' is an existing directory — the artifact needs an exact file path."),
            severity: "violation".to_string(),
        });
    }
    for w in results.iter().filter(|r| r.severity == "warning") {
        eprintln!("warning: [{}] {}: {}", w.focus, w.path, w.message);
    }
    if !has_blocking(&results) {
        return Ok(0);
    }
    let violations: Vec<_> = results.iter().filter(|r| r.severity == "violation").collect();
    eprintln!("non-conformant — '{}' ({}): {} violation(s):", wu.id, target.display(), violations.len());
    for v in &violations {
        eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
    }
    Ok(violations.len())
}

fn validate(file: Option<PathBuf>, product: Option<String>) -> BoxResult {
    let targets = validate_targets(file.clone());
    if targets.is_empty() {
        return Err(format!(
            "no work unit at {} and no fleet under .product/work-units/ — scaffold one with `product work-unit init <id>`",
            path(None).display()
        ).into());
    }
    let domain = load_domain(product.clone());
    let repo_root = super::shared::domain_root();
    let mut total_violations = 0usize;
    for target in &targets {
        total_violations += validate_one(target, domain.as_ref(), product.as_deref(), &repo_root)?;
    }
    if total_violations > 0 {
        return Err(format!("{total_violations} work-unit conformance violation(s) across {} unit(s)", targets.len()).into());
    }
    println!(
        "conformant — {} work unit(s) [domain: {}]",
        targets.len(),
        if domain.is_some() { "cross-checked" } else { "not loaded" },
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
