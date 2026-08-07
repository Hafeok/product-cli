//! `ddd what` — which What-graph boundaries carry no governing declaration.

use std::path::PathBuf;

use ddd_core::what::WhatReport;
use product_core::error::{ProductError, Result};

pub fn run(root: Option<PathBuf>, product: Option<String>, all: bool, strict: bool) -> Result<()> {
    let repo_root = super::resolve_root(root)?;
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let products = match product {
        Some(p) => vec![p],
        None => product_core::pf::paths::list_products(&repo_root),
    };
    if products.is_empty() {
        return Err(ProductError::NotFound(
            "no product under .product/ — the What adapter has nothing to govern".to_string(),
        ));
    }
    let mut escapes = 0;
    for name in &products {
        let graph = load_graph(&repo_root, name)?;
        let report = ddd_core::what::report(&store, &graph, name);
        escapes += report.escapes.len();
        print_report(&report, all);
    }
    if strict && escapes > 0 {
        return Err(ProductError::ConfigError(format!(
            "{escapes} What boundary/ies carry no declaration"
        )));
    }
    Ok(())
}

fn load_graph(repo_root: &std::path::Path, product: &str) -> Result<product_core::pf::model::DomainGraph> {
    let dir = product_core::author::domain::session_dir(repo_root, product);
    let session = product_core::pf::session::DomainSession::load(&dir).map_err(|e| {
        ProductError::NotFound(format!("no What graph for product '{product}' at {}: {e}", dir.display()))
    })?;
    Ok(session.graph)
}

fn print_report(report: &WhatReport, all: bool) {
    println!("== what governance: {} ==", report.product);
    for e in &report.escapes {
        let at = if e.container.is_empty() { String::new() } else { format!(" in {}", e.container) };
        println!("UNDECLARED {}/{}{} — {}", e.kind, e.id, at, e.rule_claim.unwrap_or(""));
    }
    if all {
        for g in &report.governed {
            println!(
                "declared  {}/{} — {}",
                g.kind,
                g.id,
                g.governed_by.as_deref().unwrap_or("?")
            );
        }
    }
    // Coverage, not just findings (dec/ddd/report-coverage-explicit).
    println!(
        "{} surface element(s): {} declared, {} undeclared — {} internal element(s) outside the table, {} classified",
        report.surface_total(),
        report.governed.len(),
        report.escapes.len(),
        report.internal,
        report.classified,
    );
}
