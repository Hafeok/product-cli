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
    /// List the built-in language backends (external ones run via `plugin`)
    Backends,
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
        /// Emit only the verification shell (IConformanceAdapter seam, wire-level
        /// scenario tests, runner, provenance) — the realiser owns all domain types
        #[arg(long = "oracle-only")]
        oracle_only: bool,
        #[arg(long)]
        product: Option<String>,
        /// Also rewrite the editable scaffolds (Decider stubs / ConformanceAdapter)
        #[arg(long)]
        force: bool,
    },
    /// Run the How contract's declared realisations (§4.2 `realisations:` block)
    Emit {
        /// Run only this realisation id (default: all declared)
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        product: Option<String>,
    },
    /// Emit a Kotlin verification shell (oracle-only): wire seam, kotlin.test facts, runner
    Kotlin {
        /// Output directory (default: reified/<product>/kotlin)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Kotlin package (default: lowercased product name)
        #[arg(long)]
        namespace: Option<String>,
        #[arg(long)]
        product: Option<String>,
        /// Also rewrite the scaffolded adapters + Gradle files
        #[arg(long)]
        force: bool,
    },
    /// Emit the language-neutral reify manifest — the whole oracle, by value, as JSON
    Manifest {
        /// Write to this file (default: stdout)
        #[arg(long)]
        out: Option<PathBuf>,
        /// Slice to one work unit's neighbourhood: a decider or projector id
        /// (frozen SPMC context for a single realisation hole; same graph hash)
        #[arg(long)]
        unit: Option<String>,
        #[arg(long)]
        namespace: Option<String>,
        #[arg(long)]
        product: Option<String>,
    },
    /// Run an external backend: pipe the manifest to CMD, write the file plan it answers
    Plugin {
        /// Shell command implementing the backend — reads the manifest JSON on
        /// stdin, answers {"files": [{"path", "content", "overwrite"?}]} on stdout
        #[arg(long)]
        cmd: String,
        /// Output directory (default: reified/<product>/plugin)
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        namespace: Option<String>,
        #[arg(long)]
        product: Option<String>,
        /// Also rewrite files the plugin marks overwrite: false
        #[arg(long)]
        force: bool,
    },
    /// Emit web pages: one HTML page per UI step composed from the How-bound design system (§4.5/§11)
    Web {
        /// Output directory (default: reified/<product>/web)
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        product: Option<String>,
        /// Also rewrite any editable scaffolds
        #[arg(long)]
        force: bool,
    },
}

pub(crate) fn handle_reify(cmd: ReifyCommands) -> BoxResult {
    match cmd {
        ReifyCommands::Backends => backends_list(),
        ReifyCommands::Check { out, product } => check(out, product),
        ReifyCommands::Csharp { out, namespace, oracle_only, product, force } => {
            emit(Lang::Csharp { oracle_only }, out, namespace, product, force)
        }
        ReifyCommands::Emit { id, product } => super::reify_how::emit_from_how(id, product),
        ReifyCommands::Kotlin { out, namespace, product, force } => {
            emit(Lang::Kotlin, out, namespace, product, force)
        }
        ReifyCommands::Manifest { out, unit, namespace, product } => {
            manifest(out, unit, namespace, product)
        }
        ReifyCommands::Plugin { cmd, out, namespace, product, force } => {
            plugin(&cmd, out, namespace, product, force)
        }
        ReifyCommands::Web { out, product, force } => emit(Lang::Web, out, None, product, force),
    }
}

/// The target backend for an emit run.
enum Lang {
    Csharp { oracle_only: bool },
    Kotlin,
    Web,
}

fn emit(lang: Lang, out: Option<PathBuf>, namespace: Option<String>, product: Option<String>, force: bool) -> BoxResult {
    let (name, graph) = require_domain(product.as_deref())?;
    let deciders = load_deciders(Some(&name))?;
    let projectors = load_projectors(Some(&name))?;
    let oracle_only = matches!(lang, Lang::Csharp { oracle_only: true } | Lang::Kotlin | Lang::Web);
    let opts = ReifyOptions {
        namespace: namespace.unwrap_or_else(|| product_core::pf::reify_ident::pascal(&name)),
        what_version: what_version(Some(&name)),
        product: name.clone(),
        oracle_only,
        design_system: super::design_system::load_bound_ds(Some(&name))?,
    };
    let (plan, subdir, mode, verify_hint) = match &lang {
        Lang::Csharp { oracle_only } => (
            plan_csharp(&graph, &deciders, &projectors, &opts)?,
            "csharp",
            if *oracle_only { "oracle-only" } else { "full" },
            format!("dotnet test — then `product decider conform <id> --runner \"dotnet {ns}.Conformance/bin/Debug/net8.0/{ns}.Conformance.dll <id>\"`", ns = opts.namespace),
        ),
        Lang::Kotlin => {
            let pkg = product_core::pf::reify_kotlin::package_of(&opts.namespace);
            (
                product_core::pf::reify_kotlin::plan_kotlin(&graph, &deciders, &projectors, &opts)?,
                "kotlin",
                "oracle-only",
                format!("gradle test — then `product decider conform <id> --runner \"build/install/{pkg}/bin/{pkg} <id>\"` after `gradle installDist`"),
            )
        }
        Lang::Web => (
            product_core::pf::reify_web::plan_web(&graph, &deciders, &projectors, &opts)?,
            "web",
            "design-system",
            "open index.g.html — on-system composition is on the data-cio attributes; drift gate: `product reify check --out <dir>`".to_string(),
        ),
    };
    let root = out.unwrap_or_else(|| default_out(&name, subdir));
    let stale = remove_stale(&root, &plan);
    let (written, kept) = write_plan(&root, &plan, force)?;
    report(&name, &root, subdir, mode, &plan, (written, kept, stale), &verify_hint);
    Ok(())
}

/// Print the one-run summary for an emit.
pub(super) fn report(name: &str, root: &std::path::Path, subdir: &str, mode: &str, plan: &ReifyPlan, counts: (usize, usize, usize), verify_hint: &str) {
    let (written, kept, stale) = counts;
    println!(
        "reified '{name}' → {} ({subdir}, {mode}) — {written} file(s) written{}{} across {} aggregate(s)",
        root.display(),
        if kept > 0 { format!(" ({kept} scaffold(s) kept)") } else { String::new() },
        if stale > 0 { format!(", {stale} stale generated file(s) removed") } else { String::new() },
        plan.aggregates.len(),
    );
    println!("  graph hash sha256:{}", plan.graph_hash);
    println!("  verify: {verify_hint} (from {})", root.display());
}

fn backends_list() -> BoxResult {
    for b in product_core::pf::reify_backend::backends() {
        println!("{:<10} {}{}", b.id(), b.description(),
            if b.oracle_only_forced() { " [oracle-only]" } else { "" });
    }
    println!("(external backends: `product reify plugin --cmd <command>` — manifest JSON in, file plan out)");
    Ok(())
}

/// Everything an emit/manifest run needs, resolved once.
pub(super) type ReifyInputs = (
    String,
    DomainGraph,
    Vec<product_core::pf::decider::Decider>,
    Vec<product_core::pf::projector::Projector>,
    ReifyOptions,
);

/// Resolve everything the manifest needs: product, graph, artifacts, options.
pub(super) fn resolve_inputs(namespace: Option<String>, product: Option<String>) -> Result<ReifyInputs, Box<dyn std::error::Error>> {
    let (name, graph) = require_domain(product.as_deref())?;
    let deciders = load_deciders(Some(&name))?;
    let projectors = load_projectors(Some(&name))?;
    let opts = ReifyOptions {
        namespace: namespace.unwrap_or_else(|| product_core::pf::reify_ident::pascal(&name)),
        what_version: what_version(Some(&name)),
        product: name.clone(),
        oracle_only: true,
        design_system: super::design_system::load_bound_ds(Some(&name))?,
    };
    Ok((name, graph, deciders, projectors, opts))
}

fn manifest(out: Option<PathBuf>, unit: Option<String>, namespace: Option<String>, product: Option<String>) -> BoxResult {
    use product_core::pf::reify_manifest as rm;
    let (_, graph, deciders, projectors, opts) = resolve_inputs(namespace, product)?;
    let json = match unit {
        Some(u) => {
            let m = rm::manifest_unit(&graph, &deciders, &projectors, &opts, &u)?;
            let mut s = serde_json::to_string_pretty(&m)?;
            s.push('\n');
            s
        }
        None => rm::manifest_json(&graph, &deciders, &projectors, &opts)?,
    };
    match out {
        Some(path) => {
            std::fs::write(&path, &json)?;
            println!("wrote reify manifest to {}", path.display());
        }
        None => print!("{json}"),
    }
    Ok(())
}

fn plugin(cmd: &str, out: Option<PathBuf>, namespace: Option<String>, product: Option<String>, force: bool) -> BoxResult {
    let (name, graph, deciders, projectors, opts) = resolve_inputs(namespace, product)?;
    let plan = super::reify_how::run_external(cmd, &graph, &deciders, &projectors, &opts)?;
    let root = out.unwrap_or_else(|| default_out(&name, "plugin"));
    let stale = remove_stale(&root, &plan);
    let (written, kept) = write_plan(&root, &plan, force)?;
    report(&name, &root, "plugin", "external", &plan, (written, kept, stale), "the plugin tree's own README/tooling; drift gate: `product reify check --out <dir>`");
    Ok(())
}

/// Delete generated files listed in the previous run's manifest that this
/// plan no longer produces (e.g. a decider was removed, or the mode changed).
/// Scaffolds are never in the manifest, so realiser-owned files are safe.
pub(super) fn remove_stale(root: &std::path::Path, plan: &ReifyPlan) -> usize {
    let Ok(prev) = std::fs::read_to_string(root.join("provenance.g.json")) else { return 0 };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&prev) else { return 0 };
    let current: std::collections::BTreeSet<&str> =
        plan.files.iter().map(|f| f.path.as_str()).collect();
    let mut removed = 0;
    for old in v.get("generated_files").and_then(|f| f.as_array()).into_iter().flatten() {
        let Some(path) = old.as_str() else { continue };
        if !current.contains(path) && std::fs::remove_file(root.join(path)).is_ok() {
            removed += 1;
        }
    }
    removed
}

pub(super) fn write_plan(root: &std::path::Path, plan: &ReifyPlan, force: bool) -> Result<(usize, usize), Box<dyn std::error::Error>> {
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
    let projectors = load_projectors(Some(&name))?;
    let current = input_hash(&graph, &name, &deciders, &projectors)?;
    let root = out.unwrap_or_else(|| default_out(&name, "csharp"));
    let prov_path = root.join("provenance.g.json");
    let text = std::fs::read_to_string(&prov_path).map_err(|_| {
        format!(
            "no provenance.g.json at {} — emit first with `product reify csharp`",
            prov_path.display()
        )
    })?;
    let recorded = recorded_hash(&text)?;
    super::design_system::check_ds_drift(&text, Some(&name))?;
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

/// Every authored Projector under the product's projectors dir, sorted by id.
fn load_projectors(product: Option<&str>) -> Result<Vec<product_core::pf::projector::Projector>, Box<dyn std::error::Error>> {
    let dir = super::projector::projectors_dir(product);
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
        out.push(product_core::pf::projector::Projector::from_yaml(&text)?);
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

fn default_out(product: &str, subdir: &str) -> PathBuf {
    super::shared::domain_root().join("reified").join(product).join(subdir)
}
