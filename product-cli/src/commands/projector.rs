//! Projector derivation plus validation + simulation against the event model (§3.4).
//!
//! `product projector {derive,list,show,simulate,validate}` derives a Projector's
//! fold signature from the captured What graph and validates/simulates an authored
//! Projector against it — no foreign events, event coverage, sound + complete.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::how_validate::has_blocking;
use product_core::pf::model::DomainGraph;
use product_core::pf::projector::{derive_projector, validate_projector, Projector};
use product_core::pf::projector_sim;
use product_core::pf::session::DomainSession;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum ProjectorCommands {
    /// Derive a Projector's fold signature for a read model from the What graph
    Derive {
        /// The read-model id to project for
        read_model: String,
        /// Product whose What graph to derive from (defaults to config)
        #[arg(long)]
        product: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// List the projectors under .product/projectors/
    List { #[arg(long)] product: Option<String> },
    /// Show a Projector's derived signature
    Show {
        /// The projector id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// Simulate the Projector's scenarios — sound + complete before realisation
    Simulate {
        /// The projector id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// Validate a Projector's signature against the event model
    Validate {
        /// The projector id (filename stem)
        name: String,
        /// Product whose What graph to cross-check against (defaults to config)
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_projector(cmd: ProjectorCommands) -> BoxResult {
    match cmd {
        ProjectorCommands::Derive { read_model, product, force } => derive(&read_model, product, force),
        ProjectorCommands::List { product } => list(product),
        ProjectorCommands::Show { name, product } => show(&name, product),
        ProjectorCommands::Simulate { name, product } => simulate(&name, product),
        ProjectorCommands::Validate { name, product } => validate(&name, product),
    }
}

pub(super) fn projectors_dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "projectors")
}

fn load_domain(product: Option<String>) -> Option<DomainGraph> {
    let p = product.or_else(super::shared::default_product_name)?;
    DomainSession::load(&session_dir(&super::shared::domain_root(), &p)).ok().map(|s| s.graph)
}

fn require_domain(product: Option<String>) -> Result<DomainGraph, Box<dyn std::error::Error>> {
    Ok(load_domain(product)
        .ok_or("no captured What graph for this product — author one with `product author domain`")?)
}

fn load(name: &str, product: Option<&str>) -> Result<Projector, Box<dyn std::error::Error>> {
    let p = projectors_dir(product).join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&p)
        .map_err(|_| format!("no projector '{name}' at {} — derive one with `product projector derive <read-model>`", p.display()))?;
    Ok(Projector::from_yaml(&text)?)
}

fn derive(read_model: &str, product: Option<String>, force: bool) -> BoxResult {
    let dir = projectors_dir(product.as_deref());
    let graph = require_domain(product)?;
    let projector = derive_projector(&graph, read_model)?;
    std::fs::create_dir_all(&dir)?;
    let p = dir.join(format!("{}.yaml", projector.id));
    if p.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", p.display()).into());
    }
    std::fs::write(&p, projector.to_yaml()?)?;
    println!("Derived projector '{}' for read model '{read_model}' at {}", projector.id, p.display());
    println!("  folds {} event(s) over {} entity(ies)", projector.folds.len(), projector.over.len());
    Ok(())
}

fn validate(name: &str, product: Option<String>) -> BoxResult {
    let projector = load(name, product.as_deref())?;
    let graph = require_domain(product)?;
    let results = validate_projector(&projector, &graph);
    for w in results.iter().filter(|r| r.severity == "warning") {
        eprintln!("warning: [{}] {}: {}", w.focus, w.path, w.message);
    }
    if has_blocking(&results) {
        let violations: Vec<_> = results.iter().filter(|r| r.severity == "violation").collect();
        eprintln!("non-conformant — {} violation(s):", violations.len());
        for v in &violations {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} projector conformance violation(s)", violations.len()).into());
    }
    println!(
        "conformant — projector '{}' projects for '{}' ({} event(s) over {} entity(ies))",
        projector.id, projector.projects_for, projector.folds.len(), projector.over.len(),
    );
    Ok(())
}

fn simulate(name: &str, product: Option<String>) -> BoxResult {
    let projector = load(name, product.as_deref())?;
    let results = projector_sim::simulate(&projector);
    if results.is_empty() {
        println!(
            "sound + complete — projector '{}': {} scenario(s) over {} event(s)",
            projector.id, projector.scenarios.len(), projector.folds.len(),
        );
        return Ok(());
    }
    eprintln!("not sound/complete — {} finding(s):", results.len());
    for r in &results {
        eprintln!("  - [{}] {}: {}", r.focus, r.path, r.message);
    }
    Err(format!("{} simulation finding(s)", results.len()).into())
}

fn show(name: &str, product: Option<String>) -> BoxResult {
    let p = load(name, product.as_deref())?;
    println!("projector: {}", p.id);
    println!("projects-for: {}", p.projects_for);
    println!("folds: {}", join(&p.folds));
    println!("over: {}", join(&p.over));
    Ok(())
}

fn join(v: &[String]) -> String {
    if v.is_empty() { "(none)".to_string() } else { v.join(", ") }
}

fn list(product: Option<String>) -> BoxResult {
    let dir = projectors_dir(product.as_deref());
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(_) => {
            println!("(no projectors — derive one with `product projector derive <read-model>`)");
            return Ok(());
        }
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no projectors)");
    }
    for n in names {
        println!("{n}");
    }
    Ok(())
}
