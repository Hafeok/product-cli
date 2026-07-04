//! Reify adapter — project the What graph into a verifiable C# frame.
//!
//! `product reify csharp` emits typed contracts (records, enums, Decider
//! frames), the generated xUnit scenario suite, and the §6.3 conformance
//! runner from the captured What graph + authored Deciders — behaviour is
//! *not* transpiled; the realiser implements the scaffolded stubs and is
//! held to the graph by the generated tests. `product reify check` is the
//! drift gate: it recomputes the input hash and fails when the emitted
//! code was generated from a graph the What has since moved past.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::model::DomainGraph;
use product_core::pf::reify::{input_hash, plan_csharp, recorded_hash, ReifyOptions, ReifyPlan};
use product_core::pf::session::DomainSession;
use product_core::pf::HowContract;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum ReifyCommands {
    /// Drift gate — fail when emitted code no longer matches the graph hash
    Check {
        /// The directory a previous `reify csharp` emitted into
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        product: Option<String>,
    },
    /// Emit a C# projection: typed contracts, Decider frames, scenario tests, conformance runner
    Csharp {
        /// Output directory (default: reified/<product>/csharp)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Root C# namespace (default: PascalCase of the product name)
        #[arg(long)]
        namespace: Option<String>,
        #[arg(long)]
        product: Option<String>,
        /// Also rewrite the editable <Aggregate>Decider.cs stubs
        #[arg(long)]
        force: bool,
    },
}

pub(crate) fn handle_reify(cmd: ReifyCommands) -> BoxResult {
    match cmd {
        ReifyCommands::Check { out, product } => check(out, product),
        ReifyCommands::Csharp { out, namespace, product, force } => {
            csharp(out, namespace, product, force)
        }
    }
}

fn csharp(out: Option<PathBuf>, namespace: Option<String>, product: Option<String>, force: bool) -> BoxResult {
    let (name, graph) = require_domain(product.as_deref())?;
    let deciders = load_deciders(Some(&name))?;
    let opts = ReifyOptions {
        namespace: namespace.unwrap_or_else(|| product_core::pf::reify_ident::pascal(&name)),
        what_version: what_version(Some(&name)),
        product: name.clone(),
    };
    let plan = plan_csharp(&graph, &deciders, &opts)?;
    let root = out.unwrap_or_else(|| default_out(&name));
    let (written, kept) = write_plan(&root, &plan, force)?;
    println!(
        "reified '{}' → {} — {} file(s) written{} across {} aggregate(s)",
        name,
        root.display(),
        written,
        if kept > 0 { format!(" ({kept} editable stub(s) kept)") } else { String::new() },
        plan.aggregates.len(),
    );
    println!("  graph hash sha256:{}", plan.graph_hash);
    println!("  verify: dotnet test — then `product decider conform <id> --runner \"dotnet run --project {}/{}.Conformance -- <id>\"`", root.display(), opts.namespace);
    Ok(())
}

fn write_plan(root: &std::path::Path, plan: &ReifyPlan, force: bool) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let (mut written, mut kept) = (0usize, 0usize);
    for f in &plan.files {
        let path = root.join(&f.path);
        if !f.overwrite && path.exists() && !force {
            kept += 1;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &f.content)?;
        written += 1;
    }
    Ok((written, kept))
}

fn check(out: Option<PathBuf>, product: Option<String>) -> BoxResult {
    let (name, graph) = require_domain(product.as_deref())?;
    let deciders = load_deciders(Some(&name))?;
    let current = input_hash(&graph, &name, &deciders)?;
    let root = out.unwrap_or_else(|| default_out(&name));
    let prov_path = root.join("provenance.g.json");
    let text = std::fs::read_to_string(&prov_path).map_err(|_| {
        format!(
            "no provenance.g.json at {} — emit first with `product reify csharp`",
            prov_path.display()
        )
    })?;
    let recorded = recorded_hash(&text)?;
    if recorded == current {
        println!("conformant — generated code at {} matches the What graph (sha256:{current})", root.display());
        return Ok(());
    }
    eprintln!("drift — the What graph has moved past the generated code:");
    eprintln!("  generated from sha256:{recorded}");
    eprintln!("  current graph  sha256:{current}");
    eprintln!("  regenerate with `product reify csharp --out {}`", root.display());
    Err("reified code is stale (graph drift)".into())
}

/// The configured product name plus its captured What graph.
fn require_domain(product: Option<&str>) -> Result<(String, DomainGraph), Box<dyn std::error::Error>> {
    let name = product
        .map(str::to_string)
        .or_else(super::shared::default_product_name)
        .ok_or("no product configured — run `product init` or pass --product")?;
    let session = DomainSession::load(&session_dir(&super::shared::domain_root(), &name))
        .map_err(|_| "no captured What graph for this product — author one with `product author domain`")?;
    Ok((name, session.graph))
}

/// Every authored Decider under the product's deciders dir, sorted by id.
fn load_deciders(product: Option<&str>) -> Result<Vec<product_core::pf::decider::Decider>, Box<dyn std::error::Error>> {
    let dir = super::decider::deciders_dir(product);
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(out) };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    paths.sort();
    for p in paths {
        let text = std::fs::read_to_string(&p)?;
        out.push(product_core::pf::decider::Decider::from_yaml(&text)?);
    }
    Ok(out)
}

/// The What version this projection realises: the How's `realises_version`
/// when a contract is present (§7.3), else "unversioned".
fn what_version(product: Option<&str>) -> String {
    let path = super::shared::artifact_dir(product, "").join("how-contract.yaml");
    HowContract::load_opt(&path)
        .ok()
        .flatten()
        .and_then(|c| c.realises_version)
        .unwrap_or_else(|| "unversioned".to_string())
}

fn default_out(product: &str) -> PathBuf {
    super::shared::domain_root().join("reified").join(product).join("csharp")
}
