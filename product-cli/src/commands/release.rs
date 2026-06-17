//! Releases — a coherent set of delivery features (§7.1).
//!
//! `product release {new,list,show}` groups deliverables that ship together. A
//! release is a partition of the What via its deliverables' slices, validated so
//! every member resolves to a real deliverable.

use clap::Subcommand;
use product_core::pf::release::{validate_release, Release};
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum ReleaseCommands {
    /// List the releases under .product/releases/
    List {},
    /// Create a release grouping delivery features
    New {
        /// The release id (e.g. R1)
        id: String,
        /// A delivery feature (deliverable id) in this release; repeatable
        #[arg(long = "feature", required = true)]
        features: Vec<String>,
        #[arg(long)]
        force: bool,
    },
    /// Show a release
    Show {
        /// The release id (filename stem)
        name: String,
    },
}

pub(crate) fn handle_release(cmd: ReleaseCommands) -> BoxResult {
    match cmd {
        ReleaseCommands::List {} => list(),
        ReleaseCommands::New { id, features, force } => new(&id, features, force),
        ReleaseCommands::Show { name } => show(&name),
    }
}

fn dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("releases")
}

fn deliverables_dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("deliverables")
}

fn load(name: &str) -> Result<Release, Box<dyn std::error::Error>> {
    let path = dir().join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| format!("no release '{name}' at {} — create one with `product release new`", path.display()))?;
    Ok(Release::from_yaml(&text)?)
}

fn new(id: &str, features: Vec<String>, force: bool) -> BoxResult {
    let release = Release { id: id.to_string(), features };
    let known = super::deliverable::ids_in(&deliverables_dir());
    let problems = validate_release(&release, &known);
    if !problems.is_empty() {
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} release problem(s)", problems.len()).into());
    }
    let d = dir();
    std::fs::create_dir_all(&d)?;
    let path = d.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(&path, release.to_yaml()?)?;
    println!("Created release '{id}' → {} feature(s): {}", release.features.len(), release.features.join(", "));
    Ok(())
}

fn show(name: &str) -> BoxResult {
    let r = load(name)?;
    println!("release: {}", r.id);
    println!("features: {}", r.features.join(", "));
    Ok(())
}

fn list() -> BoxResult {
    let ids = super::deliverable::ids_in(&dir());
    if ids.is_empty() {
        println!("(no releases — create one with `product release new <id> --feature <deliverable>`)");
    }
    for id in ids {
        println!("{id}");
    }
    Ok(())
}
