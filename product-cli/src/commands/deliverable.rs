//! Deliverables — one feature plus acceptance (§7.1).
//!
//! `product deliverable {new,list,show}` manages the shippable units of
//! delivery. A deliverable points at one feature (the §7.1 subgraph, authored
//! with `product feature`) and adds acceptance criteria — it restates no
//! behaviour; "done" is computed over the feature's footprint (§7.2).

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::decider::Decider;
use product_core::pf::deliverable::{validate_deliverable, AcceptanceCriterion, Deliverable};
use product_core::pf::done::{feature_done, FeatureDone};
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use product_core::pf::feature::Feature;
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
        #[arg(long)]
        product: Option<String>,
    },
    /// Compute whether the deliverable is done (§7.2)
    Done {
        /// The deliverable id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// List the deliverables under .product/deliverables/
    List { #[arg(long)] product: Option<String> },
    /// Create a deliverable pointing at one feature
    New {
        /// The deliverable id (e.g. place-order)
        id: String,
        /// The feature (§7.1) this deliverable ships
        #[arg(long = "feature", alias = "slice")]
        feature: String,
        /// An acceptance criterion as "id:statement"; repeatable
        #[arg(long = "accept")]
        accept: Vec<String>,
        #[arg(long)]
        product: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Bind a verification runner to an acceptance criterion (§6)
    Runner {
        /// The deliverable id
        id: String,
        /// The acceptance criterion id
        criterion: String,
        /// The runner: cargo-test | shell
        #[arg(long)]
        runner: String,
        /// Runner args — a test filter for cargo-test, a command line for shell
        #[arg(long = "args")]
        args: Option<String>,
        #[arg(long)]
        product: Option<String>,
    },
    /// Show a deliverable
    Show {
        /// The deliverable id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_deliverable(cmd: DeliverableCommands) -> BoxResult {
    match cmd {
        DeliverableCommands::Accept { id, criterion, pass, fail, product } => accept(&id, &criterion, pass, fail, product),
        DeliverableCommands::Runner { id, criterion, runner, args, product } => set_runner(&id, &criterion, &runner, args, product),
        DeliverableCommands::Done { name, product } => done(&name, product),
        DeliverableCommands::List { product } => list(product),
        DeliverableCommands::New { id, feature, accept, product, force } => new(&id, &feature, accept, product, force),
        DeliverableCommands::Show { name, product } => show(&name, product),
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

/// Load a feature pointer by id.
pub(super) fn load_feature(id: &str, product: Option<&str>) -> Result<Feature, Box<dyn std::error::Error>> {
    let path = features_dir(product).join(format!("{id}.yaml"));
    Ok(Feature::from_yaml(&std::fs::read_to_string(&path).map_err(|_| format!("feature '{id}' not found at {}", path.display()))?)?)
}

/// Load every Decider under .product/deliverables' sibling deciders/ dir.
pub(super) fn load_deciders(product: Option<&str>) -> Vec<Decider> {
    let dir = super::decider::deciders_dir(product);
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

/// Load every Projector under the sibling projectors/ dir (§3.4) — their
/// soundness gates `done` the way deciders' does.
pub(super) fn load_projectors(product: Option<&str>) -> Vec<product_core::pf::projector::Projector> {
    let dir = super::projector::projectors_dir(product);
    match std::fs::read_dir(&dir) {
        Ok(it) => it
            .flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
            .filter_map(|e| std::fs::read_to_string(e.path()).ok())
            .filter_map(|t| product_core::pf::projector::Projector::from_yaml(&t).ok())
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn accept(id: &str, criterion: &str, pass: bool, fail: bool, product: Option<String>) -> BoxResult {
    if pass == fail {
        return Err("record a verdict with exactly one of --pass / --fail".into());
    }
    let mut d = load(id, product.as_deref())?;
    let Some(c) = d.acceptance.iter_mut().find(|c| c.id == criterion) else {
        return Err(format!("no acceptance criterion '{criterion}' on deliverable '{id}'").into());
    };
    c.status = if pass { "passing" } else { "failing" }.to_string();
    let status = c.status.clone();
    std::fs::write(dir(product.as_deref()).join(format!("{id}.yaml")), d.to_yaml()?)?;
    println!("deliverable '{id}': acceptance '{criterion}' → {status}");
    Ok(())
}

/// Known verification runners (mirrors `pf::verify`). Binding an unknown runner
/// is pointless — the §6 verifier would skip it — so reject it here.
fn known_runner(runner: &str) -> bool {
    runner == "cargo-test" || runner == "shell"
}

fn set_runner(id: &str, criterion: &str, runner: &str, args: Option<String>, product: Option<String>) -> BoxResult {
    if !known_runner(runner) {
        return Err(format!("unknown runner '{runner}' — use cargo-test or shell").into());
    }
    let mut d = load(id, product.as_deref())?;
    {
        let Some(c) = d.acceptance.iter_mut().find(|c| c.id == criterion) else {
            return Err(format!("no acceptance criterion '{criterion}' on deliverable '{id}'").into());
        };
        c.runner = Some(runner.to_string());
        c.runner_args = args.filter(|s| !s.trim().is_empty());
    }
    save(&d, product.as_deref())?;
    println!("deliverable '{id}': acceptance '{criterion}' → runner '{runner}'");
    Ok(())
}

fn done(name: &str, product: Option<String>) -> BoxResult {
    let pr = product.as_deref();
    let d = load(name, pr)?;
    let feature = load_feature(&d.feature, pr)?;
    let deciders = load_deciders(pr);
    let projectors = load_projectors(pr);
    let conformed = super::decider::conformed_set(pr);
    let graph = load_graph(product.clone())?;
    let fd = feature_done(&d, &feature, &graph, &deciders, &conformed, &projectors);
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

fn dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "deliverables")
}

fn features_dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "features")
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
            Some((id, statement)) => AcceptanceCriterion { id: id.trim().to_string(), statement: statement.trim().to_string(), status: "pending".to_string(), runner: None, runner_args: None },
            None => AcceptanceCriterion { id: s.trim().to_string(), statement: String::new(), status: "pending".to_string(), runner: None, runner_args: None },
        })
        .collect()
}

pub(super) fn save(d: &Deliverable, product: Option<&str>) -> BoxResult {
    std::fs::write(dir(product).join(format!("{}.yaml", d.id)), d.to_yaml()?)?;
    Ok(())
}

pub(super) fn load(name: &str, product: Option<&str>) -> Result<Deliverable, Box<dyn std::error::Error>> {
    let path = dir(product).join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| format!("no deliverable '{name}' at {} — create one with `product deliverable new`", path.display()))?;
    Ok(Deliverable::from_yaml(&text)?)
}

fn new(id: &str, feature: &str, accept: Vec<String>, product: Option<String>, force: bool) -> BoxResult {
    let deliverable = Deliverable { id: id.to_string(), feature: feature.to_string(), acceptance: parse_acceptance(accept) };
    let problems = validate_deliverable(&deliverable, &ids_in(&features_dir(product.as_deref())));
    if !problems.is_empty() {
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} deliverable problem(s)", problems.len()).into());
    }
    let d = dir(product.as_deref());
    std::fs::create_dir_all(&d)?;
    let path = d.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(&path, deliverable.to_yaml()?)?;
    println!("Created deliverable '{id}' → feature '{feature}' ({} acceptance criteria)", deliverable.acceptance.len());
    Ok(())
}

fn show(name: &str, product: Option<String>) -> BoxResult {
    let d = load(name, product.as_deref())?;
    println!("deliverable: {}", d.id);
    println!("feature: {}", d.feature);
    if d.acceptance.is_empty() {
        println!("acceptance: (none)");
    } else {
        println!("acceptance:");
        for a in &d.acceptance {
            let runner = a.runner.as_ref().map(|r| {
                let args = a.runner_args.as_ref().map(|x| format!(" {x}")).unwrap_or_default();
                format!("  [{r}{args}]")
            }).unwrap_or_default();
            println!("  - {} [{}]: {}{}", a.id, a.status, a.statement, runner);
        }
    }
    Ok(())
}

fn list(product: Option<String>) -> BoxResult {
    let ids = ids_in(&dir(product.as_deref()));
    if ids.is_empty() {
        println!("(no deliverables — create one with `product deliverable new <id> --feature <feature>`)");
    }
    for id in ids {
        println!("{id}");
    }
    Ok(())
}
