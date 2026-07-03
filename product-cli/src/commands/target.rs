//! Target versions — a declared future partition of features (§7.3).
//!
//! `product target {new,list,show,direction}` names a What-version goal as a set
//! of deliverables, some not yet realised. `direction` is the computed gap: the
//! unrealised members, queried from the graph against the declared target — never
//! roadmap prose.

use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::Subcommand;
use product_core::pf::target::{direction, validate_target, Target};

use super::BoxResult;

#[derive(Subcommand)]
pub enum TargetCommands {
    /// Compute the gap to the target — the unrealised features (§7.3)
    Direction {
        /// The target id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// List the targets under .product/targets/
    List { #[arg(long)] product: Option<String> },
    /// Declare a target version as a set of features (deliverables)
    New {
        /// The target id (e.g. v2)
        id: String,
        /// The What-version this target constitutes (e.g. 2.0)
        #[arg(long)]
        version: Option<String>,
        /// A feature (deliverable id) in the target's partition; repeatable
        #[arg(long = "feature", alias = "slice", required = true)]
        features: Vec<String>,
        #[arg(long)]
        product: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Show a target
    Show {
        /// The target id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_target(cmd: TargetCommands) -> BoxResult {
    match cmd {
        TargetCommands::Direction { name, product } => direction_cmd(&name, product),
        TargetCommands::List { product } => list(product),
        TargetCommands::New { id, version, features, product, force } => new(&id, version, features, product, force),
        TargetCommands::Show { name, product } => show(&name, product),
    }
}

fn direction_cmd(name: &str, product: Option<String>) -> BoxResult {
    let pr = product.as_deref();
    let target = load(name, pr)?;
    let deciders = super::deliverable::load_deciders(pr);
    let projectors = super::deliverable::load_projectors(pr);
    let conformed = super::decider::conformed_set(pr);
    let graph = super::deliverable::load_graph(product.clone())?;
    // Compute feature_done for each member that resolves to a deliverable.
    let mut done = BTreeMap::new();
    for m in &target.in_target {
        if let (Ok(d), Some(s)) = (super::deliverable::load(m, pr), member_feature(m, pr)) {
            let fd = product_core::pf::done::feature_done(&d, &s, &graph, &deciders, &conformed, &projectors);
            done.insert(m.clone(), fd.done);
        }
    }
    let dir = direction(&target, &done);
    println!(
        "target '{}'{}: {:.0}% realised ({}/{} features)",
        target.id,
        dir.version.as_deref().map(|v| format!(" → What {v}")).unwrap_or_default(),
        dir.progress() * 100.0,
        dir.total - dir.unrealised.len(),
        dir.total,
    );
    if dir.unrealised.is_empty() {
        println!("  direction: reached — every feature in the partition is done.");
        Ok(())
    } else {
        println!("  distance: {} unrealised feature(s):", dir.unrealised.len());
        for m in &dir.unrealised {
            println!("    - {m}");
        }
        Err(format!("target '{name}' not yet reached — {} feature(s) unrealised", dir.unrealised.len()).into())
    }
}

/// Load a member's feature via its deliverable, returning None if either is absent.
fn member_feature(member: &str, product: Option<&str>) -> Option<product_core::pf::feature::Feature> {
    let d = super::deliverable::load(member, product).ok()?;
    super::deliverable::load_feature(&d.feature, product).ok()
}

fn dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "targets")
}

fn deliverables_dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "deliverables")
}

fn load(name: &str, product: Option<&str>) -> Result<Target, Box<dyn std::error::Error>> {
    let path = dir(product).join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| format!("no target '{name}' at {} — create one with `product target new`", path.display()))?;
    Ok(Target::from_yaml(&text)?)
}

fn new(id: &str, version: Option<String>, features: Vec<String>, product: Option<String>, force: bool) -> BoxResult {
    let target = Target { id: id.to_string(), version, in_target: features };
    let known = super::deliverable::ids_in(&deliverables_dir(product.as_deref()));
    let problems = validate_target(&target, &known);
    if !problems.is_empty() {
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} target problem(s)", problems.len()).into());
    }
    let d = dir(product.as_deref());
    std::fs::create_dir_all(&d)?;
    let path = d.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(&path, target.to_yaml()?)?;
    println!("Created target '{id}' → {} feature(s): {}", target.in_target.len(), target.in_target.join(", "));
    Ok(())
}

fn show(name: &str, product: Option<String>) -> BoxResult {
    let t = load(name, product.as_deref())?;
    println!("target: {}", t.id);
    if let Some(v) = &t.version {
        println!("version: What {v}");
    }
    println!("features: {}", t.in_target.join(", "));
    Ok(())
}

fn list(product: Option<String>) -> BoxResult {
    let ids = super::deliverable::ids_in(&dir(product.as_deref()));
    if ids.is_empty() {
        println!("(no targets — create one with `product target new <id> --feature <deliverable>`)");
    }
    for id in ids {
        println!("{id}");
    }
    Ok(())
}
