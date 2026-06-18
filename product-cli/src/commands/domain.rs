//! Domain (What) graph CRUD plus inspection commands.
//!
//! `product domain {list,show,new,edit,rm,validate,export}` lets you interact
//! with a captured What graph directly from the CLI — no agent session — by
//! reading/writing the persisted `session.json` under
//! `.product/author-domain/<product>/`. Writes go through the same in-loop
//! conformance checker as the MCP `add_*` tools, so the CLI cannot corrupt the
//! graph.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::edit::{create, edit, remove};
use product_core::pf::ids::{NodeKind, ALL_KINDS};
use product_core::pf::ops::OpResult;
use product_core::pf::session::DomainSession;
use product_core::pf::{bundle, query, turtle, validate};
use serde_json::json;
use std::path::{Path, PathBuf};

use super::domain_fields::NodeFields;
use super::BoxResult;

type Resolved = Result<(String, PathBuf), Box<dyn std::error::Error>>;

#[derive(Subcommand)]
// `new`/`edit` flatten the full NodeFields flag set; the size gap to the small
// read variants is expected for a clap subcommand enum.
#[allow(clippy::large_enum_variant)]
pub enum DomainCommands {
    /// Assemble an LLM context bundle around a node (focus + neighbourhood)
    Context {
        /// The focus node id (entity, context, flow, …)
        id: String,
        /// Traversal depth in hops from the focus node
        #[arg(long, default_value_t = 2)]
        depth: usize,
        #[arg(long)]
        product: Option<String>,
    },
    /// Edit fields of an existing node by id
    Edit {
        /// The node id to edit
        id: String,
        #[command(flatten)]
        fields: NodeFields,
        #[arg(long)]
        product: Option<String>,
    },
    /// Print the captured graph as Turtle
    Export {
        #[arg(long)]
        product: Option<String>,
    },
    /// List nodes, optionally filtered by kind
    List {
        /// Optional kind filter: entity, context, value-object, relation,
        /// invariant, mapping, command, event, read-model, wireframe-step, flow
        kind: Option<String>,
        #[arg(long)]
        product: Option<String>,
    },
    /// Create a node: <kind> <id> with --field flags
    New {
        /// The node kind (entity, context, event, …)
        kind: String,
        /// The new node id (^[A-Za-z][A-Za-z0-9_-]*$)
        id: String,
        #[command(flatten)]
        fields: NodeFields,
        #[arg(long)]
        product: Option<String>,
    },
    /// Delete a node by id
    Rm {
        /// The node id to delete
        id: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// Show a node and its links
    Show {
        /// The node id to show
        id: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// Validate the graph against the framework shapes (exit 1 on violations)
    Validate {
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_domain_cmd(cmd: DomainCommands) -> BoxResult {
    match cmd {
        DomainCommands::Context { id, depth, product } => context(id, depth, product),
        DomainCommands::List { kind, product } => list(kind, product),
        DomainCommands::Show { id, product } => show(id, product),
        DomainCommands::New { kind, id, fields, product } => new(kind, id, fields, product),
        DomainCommands::Edit { id, fields, product } => edit_node(id, fields, product),
        DomainCommands::Rm { id, product } => rm(id, product),
        DomainCommands::Validate { product } => validate_cmd(product),
        DomainCommands::Export { product } => export(product),
    }
}

/// Resolve (product, session-dir): explicit `--product` or the repo's `name`.
fn resolve(product: Option<String>) -> Resolved {
    let p = product
        .or_else(super::shared::default_product_name)
        .ok_or("no product — pass --product or set `name` in product.toml")?;
    product_core::pf::ids::validate_id(&p)?;
    Ok((p.clone(), session_dir(&super::shared::domain_root(), &p)))
}

fn load(dir: &Path) -> Result<DomainSession, Box<dyn std::error::Error>> {
    DomainSession::load(dir).map_err(|_| {
        "no domain graph for this product yet — create one with \
         `product domain new <kind> <id> …` or `product author domain`"
            .into()
    })
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn new(kind: String, id: String, fields: NodeFields, product: Option<String>) -> BoxResult {
    let (p, dir) = resolve(product)?;
    let kind = NodeKind::parse(&kind)?;
    let mut session = DomainSession::load(&dir)
        .unwrap_or(DomainSession::start(&p, None, vec![], None, now())?);
    let result = create(&mut session, kind, &id, &fields.to_map());
    session.save(&dir)?;
    report("Created", &id, result)
}

fn edit_node(id: String, fields: NodeFields, product: Option<String>) -> BoxResult {
    let (_, dir) = resolve(product)?;
    let mut session = load(&dir)?;
    let result = edit(&mut session, &id, &fields.to_map());
    session.save(&dir)?;
    report("Updated", &id, result)
}

fn rm(id: String, product: Option<String>) -> BoxResult {
    let (_, dir) = resolve(product)?;
    let mut session = load(&dir)?;
    let result = remove(&mut session, &id);
    session.save(&dir)?;
    let dangling = validate::validate_graph(&session.graph);
    let out = report("Removed", &id, result);
    if out.is_ok() && !dangling.is_empty() {
        eprintln!("warning: removing {id} left {} dangling reference(s):", dangling.len());
        for v in &dangling {
            eprintln!("  - [{}] {}", v.focus, v.message);
        }
    }
    out
}

/// Turn an `OpResult` into a CLI outcome: print on success, error on rejection.
fn report(verb: &str, id: &str, result: OpResult) -> BoxResult {
    if result.ok {
        println!("{verb} {id}");
        return Ok(());
    }
    let lines: Vec<String> = result
        .violations
        .iter()
        .map(|v| format!("  - [{}] {}", v.path, v.message))
        .collect();
    Err(format!("{verb} {id} rejected:\n{}", lines.join("\n")).into())
}

fn list(kind: Option<String>, product: Option<String>) -> BoxResult {
    let (_, dir) = resolve(product)?;
    let session = load(&dir)?;
    let filter = kind.map(|k| NodeKind::parse(&k)).transpose()?;
    let rows = list_rows(&session.graph, filter);
    if rows.is_empty() {
        println!("(no nodes)");
        return Ok(());
    }
    let kw = rows.iter().map(|r| r.0.len()).max().unwrap_or(4);
    let iw = rows.iter().map(|r| r.1.len()).max().unwrap_or(2);
    for (k, id, label) in rows {
        println!("{k:<kw$}  {id:<iw$}  {label}");
    }
    Ok(())
}

/// Build `(kind, id, label)` rows for `list`, honouring an optional filter.
fn list_rows(g: &product_core::pf::DomainGraph, filter: Option<NodeKind>) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let want = |k: NodeKind| filter.is_none_or(|f| f == k);
    let mut push = |k: NodeKind, id: &str, label: String| {
        if want(k) {
            out.push((k.cli_name().to_string(), id.to_string(), label));
        }
    };
    for n in &g.contexts { push(NodeKind::BoundedContext, &n.id, n.label.clone()); }
    for n in &g.entities { push(NodeKind::Entity, &n.id, format!("{} [{}]", n.label, n.context)); }
    for n in &g.value_objects { push(NodeKind::ValueObject, &n.id, format!("{} [{}]", n.label, n.context)); }
    for n in &g.relations { push(NodeKind::Relation, &n.id, format!("{} -{}-> {}", n.from, n.cardinality, n.to)); }
    for n in &g.invariants { push(NodeKind::Invariant, &n.id, n.statement.clone()); }
    for n in &g.context_mappings { push(NodeKind::ContextMapping, &n.id, format!("{} <-> {}", n.concept_a, n.concept_b)); }
    for n in &g.commands { push(NodeKind::Command, &n.id, format!("{} [{}]", n.label, n.context)); }
    for n in &g.events { push(NodeKind::Event, &n.id, format!("{} changes {}", n.label, n.changes)); }
    for n in &g.read_models { push(NodeKind::ReadModel, &n.id, n.label.clone()); }
    for n in &g.wireframe_steps { push(NodeKind::WireframeStep, &n.id, n.label.clone()); }
    for n in &g.flows { push(NodeKind::Flow, &n.id, n.label.clone()); }
    let _ = ALL_KINDS; // kinds enumerated above in canonical order
    out
}

fn show(id: String, product: Option<String>) -> BoxResult {
    let (_, dir) = resolve(product)?;
    let session = load(&dir)?;
    let node = query::node_value(&session.graph, &id)
        .ok_or_else(|| format!("no node with id {id:?} in the graph"))?;
    let links = query::describe(&session.graph, &id)?;
    let combined = json!({ "node": node, "links": links });
    println!("{}", serde_json::to_string_pretty(&combined)?);
    Ok(())
}

fn validate_cmd(product: Option<String>) -> BoxResult {
    let (_, dir) = resolve(product)?;
    let session = load(&dir)?;
    let violations = validate::validate_graph(&session.graph);
    if violations.is_empty() {
        println!("conformant — {} node(s), 0 violations", session.graph.node_count());
        return Ok(());
    }
    eprintln!("non-conformant — {} violation(s):", violations.len());
    for v in &violations {
        eprintln!("  - [{}] {}: {}", v.focus, v.path, v.message);
    }
    Err(format!("{} conformance violation(s)", violations.len()).into())
}

fn export(product: Option<String>) -> BoxResult {
    let (p, dir) = resolve(product)?;
    let session = load(&dir)?;
    print!("{}", turtle::to_turtle(&session.graph, &p));
    Ok(())
}

fn context(id: String, depth: usize, product: Option<String>) -> BoxResult {
    let (p, dir) = resolve(product)?;
    let session = load(&dir)?;
    let bundle = bundle::bundle(&session.graph, &id, depth, &p)
        .ok_or_else(|| format!("no node with id {id:?} in the graph"))?;
    print!("{bundle}");
    Ok(())
}
