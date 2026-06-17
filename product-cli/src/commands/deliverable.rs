//! Delivery-feature pointers — one slice plus acceptance (§7.1).
//!
//! `product deliverable {new,list,show}` manages the framework's delivery
//! features (named `deliverable` because `product feature` owns the legacy
//! FT-XXX graph). A deliverable points at one slice and restates no behaviour.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::decider::Decider;
use product_core::pf::deliverable::{validate_deliverable, AcceptanceCriterion, Deliverable};
use product_core::pf::done::{feature_done, FeatureDone};
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use product_core::pf::slice::Slice;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::BoxResult;

#[derive(Subcommand)]
pub enum DeliverableCommands {
    /// Record an acceptance criterion's verdict (--pass / --fail)
    Accept {
        /// The deliverable id
        id: String,
        /// The acceptance criterion id
        criterion: String,
        #[arg(long)]
        pass: bool,
        #[arg(long)]
        fail: bool,
    },
    /// Compute whether the deliverable is done (§7.2)
    Done {
        /// The deliverable id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
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
        DeliverableCommands::Accept { id, criterion, pass, fail } => accept(&id, &criterion, pass, fail),
        DeliverableCommands::Done { name, product } => done(&name, product),
        DeliverableCommands::List {} => list(),
        DeliverableCommands::New { id, slice, accept, force } => new(&id, &slice, accept, force),
        DeliverableCommands::Show { name } => show(&name),
    }
}

/// Load the captured What graph for the resolved product.
pub(super) fn load_graph(product: Option<String>) -> Result<DomainGraph, Box<dyn std::error::Error>> {
    let p = product
        .or_else(super::shared::default_product_name)
        .ok_or("no product configured — pass --product")?;
    Ok(DomainSession::load(&session_dir(&super::shared::domain_root(), &p))
        .map_err(|_| format!("no captured What graph for '{p}' — author one with `product author domain`"))?
        .graph)
}

/// Load a slice pointer by id.
pub(super) fn load_slice(id: &str) -> Result<Slice, Box<dyn std::error::Error>> {
    let path = slices_dir().join(format!("{id}.yaml"));
    Ok(Slice::from_yaml(&std::fs::read_to_string(&path).map_err(|_| format!("slice '{id}' not found at {}", path.display()))?)?)
}

/// Load every Decider under .product/deliverables' sibling deciders/ dir.
pub(super) fn load_deciders() -> Vec<Decider> {
    let dir = super::shared::domain_root().join(".product").join("deciders");
    match std::fs::read_dir(&dir) {
        Ok(it) => it
            .flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
            .filter_map(|e| std::fs::read_to_string(e.path()).ok())
            .filter_map(|t| Decider::from_yaml(&t).ok())
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn accept(id: &str, criterion: &str, pass: bool, fail: bool) -> BoxResult {
    if pass == fail {
        return Err("record a verdict with exactly one of --pass / --fail".into());
    }
    let mut d = load(id)?;
    let Some(c) = d.acceptance.iter_mut().find(|c| c.id == criterion) else {
        return Err(format!("no acceptance criterion '{criterion}' on deliverable '{id}'").into());
    };
    c.status = if pass { "passing" } else { "failing" }.to_string();
    let status = c.status.clone();
    std::fs::write(dir().join(format!("{id}.yaml")), d.to_yaml()?)?;
    println!("deliverable '{id}': acceptance '{criterion}' → {status}");
    Ok(())
}

fn done(name: &str, product: Option<String>) -> BoxResult {
    let d = load(name)?;
    let slice = load_slice(&d.slice)?;
    let graph = load_graph(product)?;
    let fd = feature_done(&d, &slice, &graph, &load_deciders());
    print_feature_done(&fd);
    if fd.done {
        Ok(())
    } else {
        Err(format!("deliverable '{name}' is not done").into())
    }
}

/// Print a feature-done verdict + its per-check breakdown.
pub(super) fn print_feature_done(fd: &FeatureDone) {
    let passing = fd.checks.iter().filter(|c| c.passing).count();
    println!(
        "deliverable '{}': {} ({:.0}% — {}/{} checks)",
        fd.id, if fd.done { "DONE" } else { "not done" }, fd.progress() * 100.0, passing, fd.checks.len(),
    );
    for c in &fd.checks {
        println!("  [{}] {} {}: {}", if c.passing { "x" } else { " " }, c.kind, c.subject, c.detail);
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
            Some((id, statement)) => AcceptanceCriterion { id: id.trim().to_string(), statement: statement.trim().to_string(), status: "pending".to_string() },
            None => AcceptanceCriterion { id: s.trim().to_string(), statement: String::new(), status: "pending".to_string() },
        })
        .collect()
}

pub(super) fn load(name: &str) -> Result<Deliverable, Box<dyn std::error::Error>> {
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
