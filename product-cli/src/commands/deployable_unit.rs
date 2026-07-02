//! DeployableUnit — the concrete artifact a blueprint produces (§4, §4.2).
//!
//! `product deployable-unit {new,list,show,validate}` manages the concrete
//! instances a blueprint is instantiated as: each names the blueprint it is
//! `built_from`, the system(s) it `deploys`, its environment, and the §4.2
//! deployment identity it carries. `validate` resolves the blueprint against
//! `.product/blueprints/` and each system against the captured What graph.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::deployable_unit::{
    validate_deployable_unit, DeployableUnit, DeploymentIdentity,
};
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum DeployableUnitCommands {
    /// List the deployable units under .product/deployable-units/
    List {},
    /// Instantiate a blueprint as a concrete deployable unit
    New {
        /// The deployable-unit id (e.g. shop-ios)
        id: String,
        /// The blueprint (reusable How) this unit is built from
        #[arg(long = "built-from", required = true)]
        built_from: String,
        /// A system (§3.2.5) this unit deploys; repeatable (monolith fan-out)
        #[arg(long = "system", required = true)]
        systems: Vec<String>,
        /// The deployment environment (e.g. production, staging)
        #[arg(long)]
        environment: Option<String>,
        /// §4.2 deployment identity — production domain name
        #[arg(long = "domain-name")]
        domain_name: Option<String>,
        /// §4.2 deployment identity — App Store / Play bundle id
        #[arg(long = "bundle-id")]
        bundle_id: Option<String>,
        /// §4.2 deployment identity — the chosen runtime
        #[arg(long)]
        runtime: Option<String>,
        /// Product whose What graph to resolve systems against
        #[arg(long)]
        product: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Show a deployable unit's declaration
    Show {
        /// The deployable-unit id (filename stem)
        name: String,
    },
    /// Validate a deployable unit (§4/§4.2 — blueprint, systems, identity)
    Validate {
        /// The deployable-unit id (filename stem)
        name: String,
        /// Product whose What graph to resolve systems against
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_deployable_unit(cmd: DeployableUnitCommands) -> BoxResult {
    match cmd {
        DeployableUnitCommands::List {} => list(),
        DeployableUnitCommands::New {
            id, built_from, systems, environment, domain_name, bundle_id, runtime, product, force,
        } => new(&id, built_from, systems, environment, domain_name, bundle_id, runtime, product, force),
        DeployableUnitCommands::Show { name } => show(&name),
        DeployableUnitCommands::Validate { name, product } => validate(&name, product),
    }
}

fn units_dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("deployable-units")
}

/// Prefer `.product/blueprints/`, fall back to the legacy `.product/archetypes/`.
fn blueprints_dir() -> PathBuf {
    let base = super::shared::domain_root().join(".product");
    let blueprints = base.join("blueprints");
    if blueprints.is_dir() {
        return blueprints;
    }
    let legacy = base.join("archetypes");
    if legacy.is_dir() {
        return legacy;
    }
    blueprints
}

/// The blueprint names available on disk (directory names under blueprints_dir).
fn known_blueprints() -> Vec<String> {
    product_core::pf::deployable_unit::blueprint_names(&blueprints_dir())
}

fn load_domain(product: Option<String>) -> Option<DomainGraph> {
    let p = product.or_else(super::shared::default_product_name)?;
    DomainSession::load(&session_dir(&super::shared::domain_root(), &p)).ok().map(|s| s.graph)
}

fn load(name: &str) -> Result<DeployableUnit, Box<dyn std::error::Error>> {
    let path = units_dir().join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path).map_err(|_| {
        format!("no deployable unit '{name}' at {} — create one with `product deployable-unit new`", path.display())
    })?;
    Ok(DeployableUnit::from_yaml(&text)?)
}

#[allow(clippy::too_many_arguments)]
fn new(
    id: &str,
    built_from: String,
    systems: Vec<String>,
    environment: Option<String>,
    domain_name: Option<String>,
    bundle_id: Option<String>,
    runtime: Option<String>,
    product: Option<String>,
    force: bool,
) -> BoxResult {
    let du = DeployableUnit {
        id: id.to_string(),
        built_from,
        deploys_system: systems,
        environment,
        identity: DeploymentIdentity { domain_name, bundle_id, runtime },
    };
    let graph = load_domain(product);
    let problems = validate_deployable_unit(&du, graph.as_ref(), &known_blueprints());
    if !problems.is_empty() {
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} deployable-unit problem(s)", problems.len()).into());
    }
    let dir = units_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(&path, du.to_yaml()?)?;
    println!(
        "Created deployable unit '{id}' — built_from '{}', deploys {}",
        du.built_from,
        du.deploys_system.join(", "),
    );
    Ok(())
}

fn show(name: &str) -> BoxResult {
    let du = load(name)?;
    println!("deployable-unit: {}", du.id);
    println!("built_from: {}", du.built_from);
    println!("deploys_system: {}", du.deploys_system.join(", "));
    if let Some(env) = &du.environment {
        println!("environment: {env}");
    }
    let id = &du.identity;
    println!("identity:");
    if let Some(d) = &id.domain_name {
        println!("  domain_name: {d}");
    }
    if let Some(b) = &id.bundle_id {
        println!("  bundle_id: {b}");
    }
    if let Some(r) = &id.runtime {
        println!("  runtime: {r}");
    }
    Ok(())
}

fn validate(name: &str, product: Option<String>) -> BoxResult {
    let du = load(name)?;
    let graph = load_domain(product);
    let problems = validate_deployable_unit(&du, graph.as_ref(), &known_blueprints());
    if !problems.is_empty() {
        eprintln!("non-conformant — {} violation(s):", problems.len());
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} deployable-unit conformance violation(s)", problems.len()).into());
    }
    println!(
        "conformant — deployable-unit '{name}': built_from '{}', deploys {} [domain: {}]",
        du.built_from,
        du.deploys_system.join(", "),
        if graph.is_some() { "cross-checked" } else { "not loaded" },
    );
    Ok(())
}

fn list() -> BoxResult {
    let dir = units_dir();
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(_) => {
            println!("(no deployable units — create one with `product deployable-unit new <id> --built-from <blueprint> --system <id>`)");
            return Ok(());
        }
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no deployable units)");
    }
    for n in names {
        println!("{n}");
    }
    Ok(())
}
