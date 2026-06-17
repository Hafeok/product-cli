//! Delivery-feature pointers — one slice plus acceptance (§7.1).
//!
//! `product deliverable {new,list,show}` manages the framework's delivery
//! features (named `deliverable` because `product feature` owns the legacy
//! FT-XXX graph). A deliverable points at one slice and restates no behaviour.

use clap::Subcommand;
use product_core::pf::deliverable::{validate_deliverable, AcceptanceCriterion, Deliverable};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::BoxResult;

#[derive(Subcommand)]
pub enum DeliverableCommands {
    /// List the deliverables under .product/deliverables/
    List {},
    /// Create a deliverable pointing at one slice
    New {
        /// The deliverable id (e.g. place-order)
        id: String,
        /// The slice this deliverable ships
        #[arg(long)]
        slice: String,
        /// An acceptance criterion as "id:statement"; repeatable
        #[arg(long = "accept")]
        accept: Vec<String>,
        #[arg(long)]
        force: bool,
    },
    /// Show a deliverable
    Show {
        /// The deliverable id (filename stem)
        name: String,
    },
}

pub(crate) fn handle_deliverable(cmd: DeliverableCommands) -> BoxResult {
    match cmd {
        DeliverableCommands::List {} => list(),
        DeliverableCommands::New { id, slice, accept, force } => new(&id, &slice, accept, force),
        DeliverableCommands::Show { name } => show(&name),
    }
}

fn dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("deliverables")
}

fn slices_dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("slices")
}

/// The set of artifact ids (filename stems) under a directory.
pub(super) fn ids_in(dir: &Path) -> BTreeSet<String> {
    match std::fs::read_dir(dir) {
        Ok(it) => it
            .flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
            .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
            .collect(),
        Err(_) => BTreeSet::new(),
    }
}

fn parse_acceptance(specs: Vec<String>) -> Vec<AcceptanceCriterion> {
    specs
        .into_iter()
        .map(|s| match s.split_once(':') {
            Some((id, statement)) => AcceptanceCriterion { id: id.trim().to_string(), statement: statement.trim().to_string() },
            None => AcceptanceCriterion { id: s.trim().to_string(), statement: String::new() },
        })
        .collect()
}

fn load(name: &str) -> Result<Deliverable, Box<dyn std::error::Error>> {
    let path = dir().join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| format!("no deliverable '{name}' at {} — create one with `product deliverable new`", path.display()))?;
    Ok(Deliverable::from_yaml(&text)?)
}

fn new(id: &str, slice: &str, accept: Vec<String>, force: bool) -> BoxResult {
    let deliverable = Deliverable { id: id.to_string(), slice: slice.to_string(), acceptance: parse_acceptance(accept) };
    let problems = validate_deliverable(&deliverable, &ids_in(&slices_dir()));
    if !problems.is_empty() {
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} deliverable problem(s)", problems.len()).into());
    }
    let d = dir();
    std::fs::create_dir_all(&d)?;
    let path = d.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(&path, deliverable.to_yaml()?)?;
    println!("Created deliverable '{id}' → slice '{slice}' ({} acceptance criteria)", deliverable.acceptance.len());
    Ok(())
}

fn show(name: &str) -> BoxResult {
    let d = load(name)?;
    println!("deliverable: {}", d.id);
    println!("slice: {}", d.slice);
    if d.acceptance.is_empty() {
        println!("acceptance: (none)");
    } else {
        println!("acceptance:");
        for a in &d.acceptance {
            println!("  - {}: {}", a.id, a.statement);
        }
    }
    Ok(())
}

fn list() -> BoxResult {
    let ids = ids_in(&dir());
    if ids.is_empty() {
        println!("(no deliverables — create one with `product deliverable new <id> --slice <slice>`)");
    }
    for id in ids {
        println!("{id}");
    }
    Ok(())
}
