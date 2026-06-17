//! Delivery-slice pointers plus concrete-context assembly (§7.1).
//!
//! `product slice {new,list,show,context}` manages saved pointers into the
//! captured event model. A slice restates nothing — `context` assembles the
//! reachable What subgraph from its anchors as an LLM-ready bundle.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::model::DomainGraph;
use product_core::pf::session::DomainSession;
use product_core::pf::slice::{slice_context, validate_slice, Slice};
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum SliceCommands {
    /// Assemble the concrete LLM build-context for a slice
    Context {
        /// The slice id (filename stem)
        name: String,
        /// Product whose What graph to assemble from (defaults to config)
        #[arg(long)]
        product: Option<String>,
        /// Override the slice's traversal depth
        #[arg(long)]
        depth: Option<usize>,
    },
    /// List the slices under .product/slices/
    List {},
    /// Create a slice pointing at section(s) of the event model
    New {
        /// The slice id (e.g. place-order)
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
    /// Show a slice's pointer
    Show {
        /// The slice id (filename stem)
        name: String,
    },
}

pub(crate) fn handle_slice(cmd: SliceCommands) -> BoxResult {
    match cmd {
        SliceCommands::Context { name, product, depth } => context(&name, product, depth),
        SliceCommands::List {} => list(),
        SliceCommands::New { id, anchors, depth, product, force } => new(&id, anchors, depth, product, force),
        SliceCommands::Show { name } => show(&name),
    }
}

fn slices_dir() -> PathBuf {
    super::shared::domain_root().join(".product").join("slices")
}

fn load_domain(product: Option<String>) -> Result<(String, DomainGraph), Box<dyn std::error::Error>> {
    let p = product
        .or_else(super::shared::default_product_name)
        .ok_or("no product configured — pass --product")?;
    let session = DomainSession::load(&session_dir(&super::shared::domain_root(), &p))
        .map_err(|_| format!("no captured What graph for '{p}' — author one with `product author domain`"))?;
    Ok((p, session.graph))
}

fn load(name: &str) -> Result<Slice, Box<dyn std::error::Error>> {
    let path = slices_dir().join(format!("{name}.yaml"));
    let text = std::fs::read_to_string(&path)
        .map_err(|_| format!("no slice '{name}' at {} — create one with `product slice new`", path.display()))?;
    Ok(Slice::from_yaml(&text)?)
}

fn new(id: &str, anchors: Vec<String>, depth: Option<usize>, product: Option<String>, force: bool) -> BoxResult {
    let (_p, graph) = load_domain(product)?;
    let slice = Slice { id: id.to_string(), anchors, depth };
    let problems = validate_slice(&slice, &graph);
    if !problems.is_empty() {
        for v in &problems {
            eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
        }
        return Err(format!("{} slice problem(s) — every anchor must resolve to a node", problems.len()).into());
    }
    let dir = slices_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{id}.yaml"));
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(&path, slice.to_yaml()?)?;
    println!("Created slice '{id}' → {} anchor(s): {}", slice.anchors.len(), slice.anchors.join(", "));
    Ok(())
}

fn context(name: &str, product: Option<String>, depth: Option<usize>) -> BoxResult {
    let slice = load(name)?;
    let (p, graph) = load_domain(product)?;
    let depth = depth.unwrap_or_else(|| slice.depth());
    let bundle = slice_context(&slice, &graph, depth, &p)
        .ok_or_else(|| format!("slice '{name}' resolves to no nodes in the What graph"))?;
    print!("{bundle}");
    Ok(())
}

fn show(name: &str) -> BoxResult {
    let s = load(name)?;
    println!("slice: {}", s.id);
    println!("anchors: {}", s.anchors.join(", "));
    println!("depth: {}", s.depth());
    Ok(())
}

fn list() -> BoxResult {
    let dir = slices_dir();
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(_) => {
            println!("(no slices — create one with `product slice new <id> --anchor <node>`)");
            return Ok(());
        }
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
        .collect();
    names.sort();
    if names.is_empty() {
        println!("(no slices)");
    }
    for n in names {
        println!("{n}");
    }
    Ok(())
}
