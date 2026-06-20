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
    /// Accessibility verdict for a UI step (§3.2.3) — the computed obligation
    /// union, each discharged by a machine gate or an attestation; exit 1 if any
    /// is undischarged
    Accessibility {
        /// The UI step id
        id: String,
        #[arg(long)]
        product: Option<String>,
    },
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
        DomainCommands::Accessibility { id, product } => accessibility(id, product),
    }
}

/// §3.2.3 — print a UI step's accessibility verdict (obligation union + basis);
/// exit 1 if any obligation is undischarged.
fn accessibility(id: String, product: Option<String>) -> BoxResult {
    let (_, dir) = resolve(product)?;
    let session = load(&dir)?;
    let verdict = product_core::pf::rules_ui::accessibility_verdict(&session.graph, &id)
        .ok_or_else(|| format!("no UI step with id {id:?} in the graph"))?;
    let level = verdict.obligations.iter().filter_map(|o| highest_level(&o.level)).max().unwrap_or(0);
    let level_str = ["—", "A", "AA", "AAA"][(level as usize).min(3)];
    println!("Accessibility verdict for {id}: {}", if verdict.conformant { "conformant" } else { "NOT conformant" });
    println!("  conformance level: {level_str}");
    for o in &verdict.obligations {
        let mark = if o.discharged { "✓" } else { "✗" };
        println!("  {mark} {} [{}] ({}) — {} — from {}", o.criterion, o.level, o.verification, o.basis, o.source);
    }
    if verdict.conformant {
        Ok(())
    } else {
        Err(format!("accessibility: {id} has undischarged obligations").into())
    }
}

/// Map a WCAG level string to a rank (A=1, AA=2, AAA=3) for "highest required".
fn highest_level(level: &str) -> Option<u8> {
    match level {
        "A" => Some(1),
        "AA" => Some(2),
        "AAA" => Some(3),
        _ => None,
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
    // The op did not happen — don't claim the success verb ("Created"). Make
    // it unmistakable that nothing was written, then list the rule(s) broken.
    Err(format!("Rejected {id} — no change made:\n{}", lines.join("\n")).into())
}

fn list(kind: Option<String>, product: Option<String>) -> BoxResult {
    let (_, dir) = resolve(product)?;
    let filter = kind.map(|k| NodeKind::parse(&k)).transpose()?;
    // A missing session is a clear error (tc_906) — except for `list aio`, since
    // the closed-core AIO vocabulary is recognised before any What is captured.
    let graph = match load(&dir) {
        Ok(s) => s.graph,
        Err(_) if filter == Some(NodeKind::Aio) => product_core::pf::DomainGraph::default(),
        Err(e) => return Err(e),
    };
    let rows = list_rows(&graph, filter);
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
    let mut out = structure_rows(g, filter);
    out.extend(ui_layer_rows(g, filter));
    let _ = ALL_KINDS; // kinds enumerated across both helpers in canonical order
    out
}

/// Rows for the §3.1/§3.2 structure + behaviour node kinds.
fn structure_rows(g: &product_core::pf::DomainGraph, filter: Option<NodeKind>) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut push = |k: NodeKind, id: &str, label: String| {
        if filter.is_none_or(|f| f == k) {
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
    out
}

/// Rows for the §3.2.1–§3.2.4 UI layer: pages (with derived top-level marking),
/// flows (with entry page), the application root, AIOs (core + registered), and
/// contexts of use.
fn ui_layer_rows(g: &product_core::pf::DomainGraph, filter: Option<NodeKind>) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut push = |k: NodeKind, id: &str, label: String| {
        if filter.is_none_or(|f| f == k) {
            out.push((k.cli_name().to_string(), id.to_string(), label));
        }
    };
    // §3.2.4 — "top-level" is derived: a page with an inbound edge from the root.
    let top_level: std::collections::HashSet<&str> = g
        .application_roots
        .iter()
        .flat_map(|r| r.navigates_from_root.iter().map(String::as_str))
        .collect();
    for n in &g.wireframe_steps {
        let mark = if top_level.contains(n.id.as_str()) { " [top-level]" } else { "" };
        push(NodeKind::WireframeStep, &n.id, format!("{}{mark}", n.label));
    }
    for n in &g.flows {
        let label = match &n.entry_page {
            Some(e) => format!("{} (entry: {})", n.label, e),
            None => n.label.clone(),
        };
        push(NodeKind::Flow, &n.id, label);
    }
    for n in &g.application_roots {
        push(NodeKind::ApplicationRoot, &n.id, format!("→ {}", n.navigates_from_root.join(", ")));
    }
    // The closed-core AIO vocabulary (§3.2.2) is always recognised, shown first.
    for core in product_core::pf::ids::CORE_AIOS {
        push(NodeKind::Aio, core, "(core)".to_string());
    }
    for n in &g.aios { push(NodeKind::Aio, &n.id, n.label.clone()); }
    for n in &g.contexts_of_use {
        let label = match (&n.dimension, &n.value) {
            (Some(d), Some(v)) => format!("{} [{}={}]", n.label, d, v),
            _ => n.label.clone(),
        };
        push(NodeKind::ContextOfUse, &n.id, label);
    }
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
