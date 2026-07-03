//! Releases — a coherent set of delivery features (§7.1).
//!
//! `product release {new,list,show}` groups deliverables that ship together. A
//! release is a partition of the What via its deliverables' features, validated so
//! every member resolves to a real deliverable.

use clap::Subcommand;
use product_core::pf::done::release_done;
use product_core::pf::release::{validate_release, Release};
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum ReleaseCommands {
    /// Compute whether the release is done — members done + cut closed (§7.2)
    Done {
        /// The release id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// List the releases under .product/releases/
    List { #[arg(long)] product: Option<String> },
    /// Create a release grouping delivery features
    New {
        /// The release id (e.g. R1)
        id: String,
        /// A delivery feature (deliverable id) in this release; repeatable
        #[arg(long = "feature", required = true)]
        features: Vec<String>,
        #[arg(long)]
        product: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Show a release
    Show {
        /// The release id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_release(cmd: ReleaseCommands) -> BoxResult {
    match cmd {
        ReleaseCommands::Done { name, product } => done(&name, product),
        ReleaseCommands::List { product } => list(product),
        ReleaseCommands::New { id, features, product, force } => new(&id, features, product, force),
        ReleaseCommands::Show { name, product } => show(&name, product),
    }
}

fn done(name: &str, product: Option<String>) -> BoxResult {
    let pr = product.as_deref();
    let release = load(name, pr)?;
    let deciders = super::deliverable::load_deciders(pr);
    let conformed = super::decider::conformed_set(pr);
    let projectors = super::deliverable::load_projectors(pr);
    let mut members = Vec::new();
    for f in &release.features {
        let d = super::deliverable::load(f, pr)?;
        let s = super::deliverable::load_feature(&d.feature, pr)?;
        members.push((d, s));
    }
    let graph = super::deliverable::load_graph(product.clone())?;
    let rd = release_done(&release.id, &members, &graph, &deciders, &conformed, &projectors);
    println!(
        "release '{}': {} — cut {}",
        rd.id,
        if rd.done { "DONE" } else { "not done" },
        if rd.closed() { "closed" } else { "OPEN" },
    );
    for f in &rd.members {
        super::deliverable::print_feature_done(f);
    }
    for (node, dep) in &rd.open_edges {
        println!("  open edge: {node} depends on excluded {dep}");
    }
    if rd.done {
        Ok(())
    } else {
        Err(format!("release '{name}' is not done").into())
    }
}

fn dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "releases")
}

fn deliverables_dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "deliverables")
}

fn load(name: &str, product: Option<&str>) -> Result<Release, Box<dyn std::error::Error>> {
    let path = dir(product).join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| format!("no release '{name}' at {} — create one with `product release new`", path.display()))?;
    Ok(Release::from_yaml(&text)?)
}

fn new(id: &str, features: Vec<String>, product: Option<String>, force: bool) -> BoxResult {
    let release = Release { id: id.to_string(), features };
    let known = super::deliverable::ids_in(&deliverables_dir(product.as_deref()));
    let problems = validate_release(&release, &known);
    if !problems.is_empty() {
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} release problem(s)", problems.len()).into());
    }
    let d = dir(product.as_deref());
    std::fs::create_dir_all(&d)?;
    let path = d.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(&path, release.to_yaml()?)?;
    println!("Created release '{id}' → {} feature(s): {}", release.features.len(), release.features.join(", "));
    Ok(())
}

fn show(name: &str, product: Option<String>) -> BoxResult {
    let r = load(name, product.as_deref())?;
    println!("release: {}", r.id);
    println!("features: {}", r.features.join(", "));
    Ok(())
}

fn list(product: Option<String>) -> BoxResult {
    let ids = super::deliverable::ids_in(&dir(product.as_deref()));
    if ids.is_empty() {
        println!("(no releases — create one with `product release new <id> --feature <deliverable>`)");
    }
    for id in ids {
        println!("{id}");
    }
    Ok(())
}
