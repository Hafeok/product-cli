//! Products — the top-level containers everything else is scoped to.
//!
//! `product product {new,list,show}` manages the product homes under
//! `.product/products/<name>/`: `init` creates the first one, `new` adds
//! another (multi-product repos), `list` names them all, and `show` reports
//! one home's What/How/Delivery state. The per-node §3.0 `product` *graph
//! node* is separate — that is `product domain new product <id>`.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::guide::FrameworkState;
use product_core::pf::paths::{list_products, product_home};
use product_core::pf::session::DomainSession;

use super::BoxResult;

#[derive(Subcommand)]
pub enum ProductCommands {
    /// List every product under .product/products/ (plus legacy homes)
    List,
    /// Add a product — creates its home .product/products/<id>/ with an empty What graph
    #[command(visible_alias = "add")]
    New {
        /// The product id (^[A-Za-z][A-Za-z0-9_-]*$)
        id: String,
        /// A human title for the product
        #[arg(long)]
        title: Option<String>,
    },
    /// Show a product's home and its What/How/Delivery state
    #[command(visible_alias = "view")]
    Show {
        /// The product id
        id: String,
    },
}

pub(crate) fn handle_product(cmd: ProductCommands) -> BoxResult {
    match cmd {
        ProductCommands::List => list(),
        ProductCommands::New { id, title } => new(&id, title),
        ProductCommands::Show { id } => show(&id),
    }
}

fn new(id: &str, title: Option<String>) -> BoxResult {
    product_core::pf::ids::validate_id(id)?;
    let root = super::shared::domain_root();
    let dir = session_dir(&root, id);
    if DomainSession::load(&dir).is_ok() {
        return Err(format!(
            "product '{id}' already exists at {} — inspect it with `product product show {id}`",
            dir.display()
        )
        .into());
    }
    let _lock = super::shared::acquire_write_lock().ok(); // best-effort outside a repo
    let session = DomainSession::start(id, title, vec![], None, chrono::Utc::now().to_rfc3339())?;
    let home = product_home(&root, id);
    session.save(&home)?;
    println!("Created product '{id}' → {}", home.display());
    println!("  capture its What: product domain new <kind> <id> … --product {id}");
    Ok(())
}

fn list() -> BoxResult {
    let root = super::shared::domain_root();
    let names = list_products(&root);
    if names.is_empty() {
        println!("(no products — run `product init`, or add one with `product product new <id>`)");
        return Ok(());
    }
    let default = super::shared::default_product_name();
    let w = names.iter().map(|n| n.len()).max().unwrap_or(4);
    for name in names {
        let dir = session_dir(&root, &name);
        let nodes = DomainSession::load(&dir)
            .map(|s| format!("{} node(s)", s.graph.node_count()))
            .unwrap_or_else(|_| "no What graph yet".to_string());
        let mark = if default.as_deref() == Some(name.as_str()) { " (default)" } else { "" };
        println!("{name:<w$}  {}  {nodes}{mark}", rel_home(&name, &dir, &root));
    }
    Ok(())
}

fn show(id: &str) -> BoxResult {
    let root = super::shared::domain_root();
    if !list_products(&root).iter().any(|n| n == id) {
        return Err(format!(
            "no product '{id}' — list them with `product product list`, add one with `product product new {id}`"
        )
        .into());
    }
    let dir = session_dir(&root, id);
    let s = FrameworkState::probe(&root, id);
    println!("product: {id}");
    println!("home:    {}", rel_home(id, &dir, &root));
    match DomainSession::load(&dir) {
        Ok(session) => {
            if let Some(t) = &session.title {
                println!("title:   {t}");
            }
            println!("what:    {} node(s), {} violation(s)", s.what_total, s.violations);
        }
        Err(_) => println!("what:    (no graph yet — `product domain new <kind> <id> … --product {id}`)"),
    }
    println!("how:     {}", if s.has_how { "contract present" } else { "(no how-contract.yaml)" });
    println!(
        "build:   {} decider(s), {} projector(s), {} feature(s), {} deliverable(s), {} release(s)",
        s.deciders, s.projectors, s.features, s.deliverables, s.releases
    );
    Ok(())
}

/// The home shown to the user: relative to the repo root when possible.
fn rel_home(product: &str, dir: &std::path::Path, root: &std::path::Path) -> String {
    let shown = dir.strip_prefix(root).unwrap_or(dir);
    let s = shown.display().to_string();
    if s.contains("author-domain") {
        format!("{s} (legacy — new artifacts land in .product/products/{product}/)")
    } else {
        s
    }
}
