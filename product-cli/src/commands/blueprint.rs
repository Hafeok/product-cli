//! Blueprint assembly inspection plus whole-blueprint validation.
//!
//! `product blueprint {validate,show,list,init}` assembles a blueprint from
//! `.product/blueprints/<name>/` (its How contract, layout model, and the
//! task-type cells under `cells/`) and validates the whole thing — each part
//! against its shapes plus the cross-part coherence, with cells' `domain:`
//! inputs cross-checked against the captured What graph.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::blueprint::Blueprint;
use product_core::pf::how_validate::has_blocking;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum BlueprintCommands {
    /// Check the blueprint's layout against the actual repository tree (§4.3)
    Check {
        /// The blueprint name
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// Scaffold a new blueprint (How + layout + an example cell)
    Init {
        /// The blueprint name (e.g. rest-api)
        name: String,
        #[arg(long)]
        product: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// List the blueprints under .product/blueprints/
    List { #[arg(long)] product: Option<String> },
    /// Show a summary of an assembled blueprint
    Show {
        /// The blueprint name
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// Validate the whole blueprint (How + layout + cells + coherence)
    Validate {
        /// The blueprint name
        name: String,
        /// Product whose What graph to cross-check cells against
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_blueprint(cmd: BlueprintCommands) -> BoxResult {
    match cmd {
        BlueprintCommands::Validate { name, product } => validate(&name, product),
        BlueprintCommands::Check { name, product } => check(&name, product),
        BlueprintCommands::Show { name, product } => show(&name, product),
        BlueprintCommands::List { product } => list(product),
        BlueprintCommands::Init { name, product, force } => init(&name, product, force),
    }
}

/// Apply the blueprint's layout model to the actual repository tree (§4.3
/// layout-conformance — the cheapest gate). `validate` checks the model is
/// well-formed; `check` checks the repo conforms to it.
fn check(name: &str, product: Option<String>) -> BoxResult {
    let root = super::shared::domain_root();
    let arch = Blueprint::load_from_dir(&blueprints_dir(product.as_deref()).join(name), name)?;
    let Some(layout) = &arch.layout else {
        println!("blueprint '{name}': no layout model to check");
        return Ok(());
    };
    let violations = product_core::pf::layout_check::check_layout(layout, &root);
    if violations.is_empty() {
        println!("layout-conformant — {} rule(s) over {} hold against the tree", layout.layout.len(), root.display());
        return Ok(());
    }
    eprintln!("layout violations — {}:", violations.len());
    for v in &violations {
        eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
    }
    Err(format!("{} layout violation(s)", violations.len()).into())
}

fn blueprints_dir(product: Option<&str>) -> PathBuf {
    let pdir = super::shared::artifact_dir(product, "");
    let canonical = pdir.join("blueprints");
    if canonical.exists() {
        return canonical;
    }
    // Back-compat: fall back to the legacy `.product/archetypes/` if present.
    let legacy = pdir.join("archetypes");
    if legacy.exists() {
        return legacy;
    }
    canonical
}

fn load_domain(product: Option<String>) -> Option<DomainGraph> {
    let p = product.or_else(super::shared::default_product_name)?;
    DomainSession::load(&session_dir(&super::shared::domain_root(), &p)).ok().map(|s| s.graph)
}

fn validate(name: &str, product: Option<String>) -> BoxResult {
    let arch = Blueprint::load_from_dir(&blueprints_dir(product.as_deref()).join(name), name)?;
    let domain = load_domain(product);
    let results = arch.validate(domain.as_ref());

    for w in results.iter().filter(|r| r.severity == "warning") {
        eprintln!("warning: [{}] {}: {}", w.focus, w.path, w.message);
    }
    if has_blocking(&results) {
        let violations: Vec<_> = results.iter().filter(|r| r.severity == "violation").collect();
        eprintln!("non-conformant — {} violation(s):", violations.len());
        for v in &violations {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} blueprint conformance violation(s)", violations.len()).into());
    }
    println!(
        "conformant — blueprint '{name}': how {}, layout {}, {} cell(s) [domain: {}]",
        yes_no(arch.how.is_some()),
        yes_no(arch.layout.is_some()),
        arch.cells.len(),
        if domain.is_some() { "cross-checked" } else { "not loaded" },
    );
    Ok(())
}

fn yes_no(b: bool) -> &'static str {
    if b { "present" } else { "missing" }
}

fn show(name: &str, product: Option<String>) -> BoxResult {
    let arch = Blueprint::load_from_dir(&blueprints_dir(product.as_deref()).join(name), name)?;
    println!("blueprint: {name}");
    if let Some(how) = &arch.how {
        println!("how-contract: {} ({} decision(s), {} principle(s), {} pattern(s))",
            how.application_contract.id, how.top_decisions.len(), how.principles.len(), how.patterns.len());
    } else {
        println!("how-contract: (missing)");
    }
    match &arch.layout {
        Some(l) => println!("layout: {} rule(s)", l.layout.len()),
        None => println!("layout: (none)"),
    }
    println!("cells:");
    for (src, cell) in &arch.cells {
        println!("  - {src}: {} ({} slot(s), {} cell(s))", cell.name, cell.slots.len(), cell.cells.len());
    }
    Ok(())
}

fn list(product: Option<String>) -> BoxResult {
    let dir = blueprints_dir(product.as_deref());
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(_) => {
            println!("(no blueprints — scaffold one with `product blueprint init <name>`)");
            return Ok(());
        }
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no blueprints)");
    }
    for n in names {
        println!("{n}");
    }
    Ok(())
}

fn init(name: &str, product: Option<String>, force: bool) -> BoxResult {
    let dir = blueprints_dir(product.as_deref()).join(name);
    if dir.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", dir.display()).into());
    }
    let written = Blueprint::scaffold(&dir, name)?;
    println!("Scaffolded blueprint '{name}' at {}", dir.display());
    for w in &written {
        println!("  {w}");
    }
    Ok(())
}
