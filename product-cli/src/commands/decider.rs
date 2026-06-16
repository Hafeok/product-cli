//! Decider derivation plus signature validation against the event model (§3.3).
//!
//! `product decider {derive,list,show,validate}` derives a Decider's signature
//! from the captured What graph and validates an authored Decider against it —
//! no foreign commands, command coverage, output-alphabet containment.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::decider::{derive_decider, validate_decider, Decider};
use product_core::pf::how_validate::has_blocking;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum DeciderCommands {
    /// Derive a Decider's signature for an aggregate from the What graph
    Derive {
        /// The aggregate entity id to decide for
        aggregate: String,
        /// Product whose What graph to derive from (defaults to config)
        #[arg(long)]
        product: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// List the deciders under .product/deciders/
    List {},
    /// Show a Decider's derived signature
    Show {
        /// The decider id (filename stem)
        name: String,
    },
    /// Validate a Decider's signature against the event model
    Validate {
        /// The decider id (filename stem)
        name: String,
        /// Product whose What graph to cross-check against (defaults to config)
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_decider(cmd: DeciderCommands) -> BoxResult {
    match cmd {
        DeciderCommands::Derive { aggregate, product, force } => derive(&aggregate, product, force),
        DeciderCommands::List {} => list(),
        DeciderCommands::Show { name } => show(&name),
        DeciderCommands::Validate { name, product } => validate(&name, product),
    }
}

fn deciders_dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("deciders")
}

fn load_domain(product: Option<String>) -> Option<DomainGraph> {
    let p = product.or_else(super::shared::default_product_name)?;
    DomainSession::load(&session_dir(&super::shared::domain_root(), &p)).ok().map(|s| s.graph)
}

fn require_domain(product: Option<String>) -> Result<DomainGraph, Box<dyn std::error::Error>> {
    Ok(load_domain(product)
        .ok_or("no captured What graph for this product — author one with `product author domain`")?)
}

fn load(name: &str) -> Result<Decider, Box<dyn std::error::Error>> {
    let p = deciders_dir().join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&p)
        .map_err(|_| format!("no decider '{name}' at {} — derive one with `product decider derive <aggregate>`", p.display()))?;
    Ok(Decider::from_yaml(&text)?)
}

fn derive(aggregate: &str, product: Option<String>, force: bool) -> BoxResult {
    let graph = require_domain(product)?;
    let decider = derive_decider(&graph, aggregate)?;
    let dir = deciders_dir();
    std::fs::create_dir_all(&dir)?;
    let p = dir.join(format!("{}.yaml", decider.id));
    if p.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", p.display()).into());
    }
    std::fs::write(&p, decider.to_yaml()?)?;
    println!("Derived decider '{}' for aggregate '{aggregate}' at {}", decider.id, p.display());
    println!(
        "  handles {} command(s), emits {} event(s), evolves from {} event(s), rejects {} invariant(s)",
        decider.handles.len(), decider.emits.len(), decider.evolves_from.len(), decider.rejects.len(),
    );
    Ok(())
}

fn validate(name: &str, product: Option<String>) -> BoxResult {
    let decider = load(name)?;
    let graph = require_domain(product)?;
    let results = validate_decider(&decider, &graph);
    for w in results.iter().filter(|r| r.severity == "warning") {
        eprintln!("warning: [{}] {}: {}", w.focus, w.path, w.message);
    }
    if has_blocking(&results) {
        let violations: Vec<_> = results.iter().filter(|r| r.severity == "violation").collect();
        eprintln!("non-conformant — {} violation(s):", violations.len());
        for v in &violations {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} decider conformance violation(s)", violations.len()).into());
    }
    println!(
        "conformant — decider '{}' decides for '{}' ({} command(s), {} event(s))",
        decider.id, decider.decides_for, decider.handles.len(), decider.emits.len(),
    );
    Ok(())
}

fn show(name: &str) -> BoxResult {
    let d = load(name)?;
    println!("decider: {}", d.id);
    println!("decides-for: {}", d.decides_for);
    println!("handles: {}", join(&d.handles));
    println!("emits: {}", join(&d.emits));
    println!("evolves-from: {}", join(&d.evolves_from));
    println!("rejects: {}", join(&d.rejects));
    Ok(())
}

fn join(v: &[String]) -> String {
    if v.is_empty() { "(none)".to_string() } else { v.join(", ") }
}

fn list() -> BoxResult {
    let dir = deciders_dir();
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(_) => {
            println!("(no deciders — derive one with `product decider derive <aggregate>`)");
            return Ok(());
        }
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no deciders)");
    }
    for n in names {
        println!("{n}");
    }
    Ok(())
}
