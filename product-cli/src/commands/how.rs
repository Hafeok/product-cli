//! How-contract (§4 architecture) inspection plus validation commands.
//!
//! `product how {validate,show,list,export,init}` operates on an archetype's
//! How contract — a YAML file (default `.product/how-contract.yaml`). Unlike
//! the What graph (an RDF session), the How is authored as a file and
//! projected into the graph; these commands validate it (incl. the crown
//! trace-truth rule), render it, and project it to Turtle.

use clap::Subcommand;
use product_core::pf::how::HowContract;
use product_core::pf::how_turtle::how_to_turtle;
use product_core::pf::how_validate::{has_blocking, validate_how};
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum HowCommands {
    /// Add a Why-cascade element or contract part:
    /// decision | principle | pattern | interface | app-statement | resource
    Add {
        /// The element kind to add
        element: String,
        /// The new element id
        id: String,
        #[command(flatten)]
        fields: super::how_fields::HowFields,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Project the How contract into the graph as Turtle
    Export {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Scaffold a starter how-contract.yaml
    Init {
        /// The archetype this How belongs to (e.g. rest-api)
        #[arg(long)]
        archetype: Option<String>,
        #[arg(long)]
        file: Option<PathBuf>,
        /// Overwrite an existing file
        #[arg(long)]
        force: bool,
    },
    /// List items of a kind: decisions, principles, patterns, interfaces
    List {
        /// One of: decisions, principles, patterns, interfaces
        kind: String,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Set a singleton contract: app-contract | infra-contract (with --id)
    Set {
        /// One of: app-contract, infra-contract
        target: String,
        /// The contract id
        #[arg(long)]
        id: String,
        #[command(flatten)]
        fields: super::how_fields::HowFields,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Show a summary of the How contract
    Show {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Validate the contract (structure + conformance + trace-truth)
    Validate {
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

pub(crate) fn handle_how(cmd: HowCommands) -> BoxResult {
    match cmd {
        HowCommands::Add { element, id, fields, file } => add(element, id, fields, file),
        HowCommands::Set { target, id, fields, file } => set(target, id, fields, file),
        HowCommands::Validate { file } => validate(file),
        HowCommands::Show { file } => show(file),
        HowCommands::List { kind, file } => list(kind, file),
        HowCommands::Export { file } => export(file),
        HowCommands::Init { archetype, file, force } => init(archetype, file, force),
    }
}

/// Resolve the contract path: `--file` or `<root>/.product/how-contract.yaml`.
fn path(file: Option<PathBuf>) -> PathBuf {
    file.unwrap_or_else(|| super::shared::domain_root().join(".product").join("how-contract.yaml"))
}

fn load(file: Option<PathBuf>) -> Result<HowContract, Box<dyn std::error::Error>> {
    let p = path(file);
    let text = std::fs::read_to_string(&p).map_err(|_| {
        format!(
            "no how-contract at {} — scaffold one with `product how init`",
            p.display()
        )
    })?;
    Ok(HowContract::from_yaml(&text)?)
}

/// Load the contract, or start a fresh one keyed to the repo product so the
/// How can be built up element by element.
fn load_or_init(file: &Option<PathBuf>) -> Result<HowContract, Box<dyn std::error::Error>> {
    let p = path(file.clone());
    match std::fs::read_to_string(&p) {
        Ok(text) => Ok(HowContract::from_yaml(&text)?),
        Err(_) => Ok(HowContract {
            archetype: super::shared::default_product_name().unwrap_or_else(|| "archetype".to_string()),
            ..Default::default()
        }),
    }
}

fn save(contract: &HowContract, file: &Option<PathBuf>) -> BoxResult {
    let p = path(file.clone());
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, contract.to_yaml()?)?;
    Ok(())
}

fn add(element: String, id: String, fields: super::how_fields::HowFields, file: Option<PathBuf>) -> BoxResult {
    use product_core::pf::how_edit as edit;
    let mut c = load_or_init(&file)?;
    match element.as_str() {
        "decision" => edit::add_decision(&mut c, fields.decision(&id))?,
        "principle" => edit::add_principle(&mut c, fields.principle(&id))?,
        "pattern" => edit::add_pattern(&mut c, fields.pattern(&id))?,
        "interface" => edit::add_interface(&mut c, fields.interface(&id))?,
        "app-statement" => edit::add_app_statement(&mut c, fields.app_statement(&id))?,
        "resource" => edit::add_resource(&mut c, fields.resource(&id))?,
        other => return Err(format!(
            "unknown element {other:?} — use decision, principle, pattern, interface, app-statement, or resource"
        ).into()),
    }
    save(&c, &file)?;
    println!("Added {element} '{id}'");
    Ok(())
}

fn set(target: String, id: String, fields: super::how_fields::HowFields, file: Option<PathBuf>) -> BoxResult {
    use product_core::pf::how_edit as edit;
    let mut c = load_or_init(&file)?;
    match target.as_str() {
        "app-contract" => edit::set_app_contract(&mut c, fields.app_contract(&id)),
        "infra-contract" => edit::set_infra_contract(&mut c, fields.infra_contract(&id)),
        other => return Err(format!("unknown target {other:?} — use app-contract or infra-contract").into()),
    }
    save(&c, &file)?;
    println!("Set {target} '{id}'");
    Ok(())
}

fn validate(file: Option<PathBuf>) -> BoxResult {
    let contract = load(file)?;
    let results = validate_how(&contract);
    let warnings: Vec<_> = results.iter().filter(|r| r.severity == "warning").collect();
    let violations: Vec<_> = results.iter().filter(|r| r.severity == "violation").collect();
    for w in &warnings {
        eprintln!("warning: [{}] {}: {}", w.focus, w.path, w.message);
    }
    if has_blocking(&results) {
        eprintln!("non-conformant — {} violation(s):", violations.len());
        for v in &violations {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} How conformance violation(s)", violations.len()).into());
    }
    println!(
        "conformant — {} decision(s), {} principle(s), {} pattern(s); {} warning(s)",
        contract.top_decisions.len(),
        contract.principles.len(),
        contract.patterns.len(),
        warnings.len()
    );
    Ok(())
}

fn show(file: Option<PathBuf>) -> BoxResult {
    let c = load(file)?;
    println!("archetype: {}", c.archetype);
    if let Some(v) = &c.version {
        println!("version:   {v}");
    }
    println!("application-contract: {} ({})", c.application_contract.id, c.application_contract.language);
    println!("  statements: {}", c.application_contract.statements.len());
    if let Some(infra) = &c.infrastructure_contract {
        println!("infrastructure-contract: {} satisfies {} ({} resource(s))", infra.id, infra.satisfies, infra.resources.len());
    }
    println!("top-decisions: {}", c.top_decisions.len());
    println!("principles:    {}", c.principles.len());
    println!("patterns:      {}", c.patterns.len());
    println!("interfaces:    {}", c.interface_contracts.len());
    Ok(())
}

fn list(kind: String, file: Option<PathBuf>) -> BoxResult {
    let c = load(file)?;
    let rows: Vec<(String, String)> = match kind.as_str() {
        "decisions" | "decision" => c.top_decisions.iter().map(|d| (d.id.clone(), d.decision.clone())).collect(),
        "principles" | "principle" => c.principles.iter().map(|p| (p.id.clone(), p.statement.clone())).collect(),
        "patterns" | "pattern" => c.patterns.iter().map(|p| (p.id.clone(), p.shape.clone())).collect(),
        "interfaces" | "interface" => c.interface_contracts.iter().map(|i| (i.id.clone(), format!("{} ({})", i.surface, i.standard))).collect(),
        other => return Err(format!("unknown kind {other:?} — use decisions, principles, patterns, or interfaces").into()),
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

fn export(file: Option<PathBuf>) -> BoxResult {
    print!("{}", how_to_turtle(&load(file)?));
    Ok(())
}

fn init(archetype: Option<String>, file: Option<PathBuf>, force: bool) -> BoxResult {
    let p = path(file);
    if p.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", p.display()).into());
    }
    let name = archetype
        .or_else(super::shared::default_product_name)
        .unwrap_or_else(|| "archetype".to_string());
    let contract = HowContract::scaffold(&name);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, contract.to_yaml()?)?;
    println!("Scaffolded How contract for '{name}' at {}", p.display());
    Ok(())
}
