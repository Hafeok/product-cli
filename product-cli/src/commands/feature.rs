//! Delivery-feature pointers plus concrete-context assembly (§7.1).
//!
//! `product feature {new,list,show,context}` manages saved pointers into the
//! captured event model. A feature restates nothing — `context` assembles the
//! reachable What subgraph from its anchors as an LLM-ready bundle.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use product_core::pf::feature::{feature_context, validate_feature, Feature};
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum FeatureCommands {
    /// Assemble the concrete LLM build-context for a feature
    Context {
        /// The feature id (filename stem)
        name: String,
        /// Product whose What graph to assemble from (defaults to config)
        #[arg(long)]
        product: Option<String>,
        /// Override the feature's traversal depth
        #[arg(long)]
        depth: Option<usize>,
    },
    /// List the features under .product/features/
    List {
        #[arg(long)]
        product: Option<String>,
    },
    /// Create a feature pointing at subgraph(s) of the event model
    New {
        /// The feature id (e.g. place-order)
        id: String,
        /// An anchor node id (a flow, context, aggregate…); repeatable
        #[arg(long = "anchor", required = true)]
        anchors: Vec<String>,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        product: Option<String>,
        #[arg(long)]
        force: bool,
    },
    /// Show a feature's pointer
    Show {
        /// The feature id (filename stem)
        name: String,
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_feature(cmd: FeatureCommands) -> BoxResult {
    match cmd {
        FeatureCommands::Context { name, product, depth } => context(&name, product, depth),
        FeatureCommands::List { product } => list(product),
        FeatureCommands::New { id, anchors, depth, product, force } => new(&id, anchors, depth, product, force),
        FeatureCommands::Show { name, product } => show(&name, product),
    }
}

fn features_dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "features")
}

fn load_domain(product: Option<String>) -> Result<(String, DomainGraph), Box<dyn std::error::Error>> {
    let p = product
        .or_else(super::shared::default_product_name)
        .ok_or("no product configured — pass --product")?;
    let session = DomainSession::load(&session_dir(&super::shared::domain_root(), &p))
        .map_err(|_| format!("no captured What graph for '{p}' — author one with `product author domain`"))?;
    Ok((p, session.graph))
}

fn load(name: &str, product: Option<&str>) -> Result<Feature, Box<dyn std::error::Error>> {
    let path = features_dir(product).join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| format!("no feature '{name}' at {} — create one with `product feature new`", path.display()))?;
    Ok(Feature::from_yaml(&text)?)
}

fn new(id: &str, anchors: Vec<String>, depth: Option<usize>, product: Option<String>, force: bool) -> BoxResult {
    let (p, graph) = load_domain(product)?;
    let feature = Feature { id: id.to_string(), anchors, depth };
    let problems = validate_feature(&feature, &graph);
    if !problems.is_empty() {
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} feature problem(s) — every anchor must resolve to a node", problems.len()).into());
    }
    let dir = features_dir(Some(&p));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(&path, feature.to_yaml()?)?;
    println!("Created feature '{id}' → {} anchor(s): {}", feature.anchors.len(), feature.anchors.join(", "));
    Ok(())
}

fn context(name: &str, product: Option<String>, depth: Option<usize>) -> BoxResult {
    let feature = load(name, product.as_deref())?;
    let (p, graph) = load_domain(product)?;
    let depth = depth.unwrap_or_else(|| feature.depth());
    let bundle = feature_context(&feature, &graph, depth, &p)
        .ok_or_else(|| format!("feature '{name}' resolves to no nodes in the What graph"))?;
    print!("{bundle}");
    Ok(())
}

fn show(name: &str, product: Option<String>) -> BoxResult {
    let s = load(name, product.as_deref())?;
    println!("feature: {}", s.id);
    println!("anchors: {}", s.anchors.join(", "));
    println!("depth: {}", s.depth());
    Ok(())
}

fn list(product: Option<String>) -> BoxResult {
    let dir = features_dir(product.as_deref());
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(_) => {
            println!("(no features — create one with `product feature new <id> --anchor <node>`)");
            return Ok(());
        }
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no features)");
    }
    for n in names {
        println!("{n}");
    }
    Ok(())
}
