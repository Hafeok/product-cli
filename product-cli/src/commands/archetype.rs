//! Archetype assembly inspection plus whole-archetype validation.
//!
//! `product archetype {validate,show,list,init}` assembles an archetype from
//! `.product/archetypes/<name>/` (its How contract, layout model, and the
//! task-type cells under `cells/`) and validates the whole thing — each part
//! against its shapes plus the cross-part coherence, with cells' `domain:`
//! inputs cross-checked against the captured What graph.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::archetype::Archetype;
use product_core::pf::cell::TaskType;
use product_core::pf::how::HowContract;
use product_core::pf::how_validate::has_blocking;
use product_core::pf::layout::LayoutModel;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum ArchetypeCommands {
    /// Check the archetype's layout against the actual repository tree (§4.3)
    Check {
        /// The archetype name
        name: String,
    },
    /// Scaffold a new archetype (How + layout + an example cell)
    Init {
        /// The archetype name (e.g. rest-api)
        name: String,
        #[arg(long)]
        force: bool,
    },
    /// List the archetypes under .product/archetypes/
    List {},
    /// Show a summary of an assembled archetype
    Show {
        /// The archetype name
        name: String,
    },
    /// Validate the whole archetype (How + layout + cells + coherence)
    Validate {
        /// The archetype name
        name: String,
        /// Product whose What graph to cross-check cells against
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_archetype(cmd: ArchetypeCommands) -> BoxResult {
    match cmd {
        ArchetypeCommands::Validate { name, product } => validate(&name, product),
        ArchetypeCommands::Check { name } => check(&name),
        ArchetypeCommands::Show { name } => show(&name),
        ArchetypeCommands::List {} => list(),
        ArchetypeCommands::Init { name, force } => init(&name, force),
    }
}

/// Apply the archetype's layout model to the actual repository tree (§4.3
/// layout-conformance — the cheapest gate). `validate` checks the model is
/// well-formed; `check` checks the repo conforms to it.
fn check(name: &str) -> BoxResult {
    let root = super::shared::domain_root();
    let arch = Archetype::load_from_dir(&archetypes_dir().join(name), name)?;
    let Some(layout) = &arch.layout else {
        println!("archetype '{name}': no layout model to check");
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

fn archetypes_dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("archetypes")
}

fn load_domain(product: Option<String>) -> Option<DomainGraph> {
    let p = product.or_else(super::shared::default_product_name)?;
    DomainSession::load(&session_dir(&super::shared::domain_root(), &p)).ok().map(|s| s.graph)
}

fn validate(name: &str, product: Option<String>) -> BoxResult {
    let arch = Archetype::load_from_dir(&archetypes_dir().join(name), name)?;
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
        return Err(format!("{} archetype conformance violation(s)", violations.len()).into());
    }
    println!(
        "conformant — archetype '{name}': how {}, layout {}, {} cell(s) [domain: {}]",
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

fn show(name: &str) -> BoxResult {
    let arch = Archetype::load_from_dir(&archetypes_dir().join(name), name)?;
    println!("archetype: {name}");
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

fn list() -> BoxResult {
    let dir = archetypes_dir();
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(_) => {
            println!("(no archetypes — scaffold one with `product archetype init <name>`)");
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
        println!("(no archetypes)");
    }
    for n in names {
        println!("{n}");
    }
    Ok(())
}

fn init(name: &str, force: bool) -> BoxResult {
    let dir = archetypes_dir().join(name);
    if dir.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", dir.display()).into());
    }
    std::fs::create_dir_all(dir.join("cells"))?;
    std::fs::write(dir.join("how-contract.yaml"), HowContract::scaffold(name).to_yaml()?)?;
    std::fs::write(dir.join("layout.yaml"), LayoutModel::scaffold(name).to_yaml()?)?;
    std::fs::write(
        dir.join("cells").join("example-task.yaml"),
        TaskType::scaffold("example-task", name).to_yaml()?,
    )?;
    println!("Scaffolded archetype '{name}' at {}", dir.display());
    println!("  how-contract.yaml, layout.yaml, cells/example-task.yaml");
    Ok(())
}
