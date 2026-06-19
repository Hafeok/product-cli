//! A worked example product — the bookstore What model, seeded for demos.
//!
//! `product init --demo` calls [`seed_bookstore`] to drop a small, conformant
//! model on disk so a workshop participant has something real to explore in
//! seconds (`product status`, `product domain show Order`, `product guide`).

use serde_json::{json, Map, Value};
use std::path::Path;

use crate::error::ProductError;
use crate::pf::edit::create;
use crate::pf::ids::NodeKind;
use crate::pf::session::DomainSession;

fn add(
    session: &mut DomainSession,
    kind: &str,
    id: &str,
    fields: Vec<(&str, Value)>,
) -> Result<(), ProductError> {
    let map: Map<String, Value> = fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    let node_kind =
        NodeKind::parse(kind).map_err(|e| ProductError::IoError(format!("demo: bad kind {kind}: {e}")))?;
    let result = create(session, node_kind, id, &map);
    if !result.ok {
        let detail: Vec<String> = result
            .violations
            .iter()
            .map(|v| format!("[{}] {}", v.focus, v.message))
            .collect();
        return Err(ProductError::IoError(format!(
            "demo: node {id} unexpectedly rejected: {}",
            detail.join("; ")
        )));
    }
    Ok(())
}

/// Seed a small, conformant "bookstore" What model under `repo_root` for
/// `product` (a Catalog context, Book/Order entities, an OrderPlaced event, a
/// PlaceOrder command, an OrderSummary read model). Returns the node count.
pub fn seed_bookstore(repo_root: &Path, product: &str) -> Result<usize, ProductError> {
    let dir = crate::author::domain::session_dir(repo_root, product);
    let now = chrono::Utc::now().to_rfc3339();
    let mut session = DomainSession::start(product, None, vec![], None, now)
        .map_err(|e| ProductError::IoError(format!("demo: session start: {e}")))?;

    add(&mut session, "context", "Catalog",
        vec![("label", json!("Catalog")), ("purpose", json!("Browse and buy books"))])?;
    add(&mut session, "entity", "Book", vec![
        ("label", json!("Book")), ("context", json!("Catalog")),
        ("definition", json!("A book offered for sale")), ("is_aggregate_root", json!(true)),
    ])?;
    add(&mut session, "entity", "Order", vec![
        ("label", json!("Order")), ("context", json!("Catalog")),
        ("definition", json!("A customer order")), ("is_aggregate_root", json!(true)),
    ])?;
    add(&mut session, "event", "OrderPlaced", vec![
        ("label", json!("Order placed")), ("context", json!("Catalog")), ("changes", json!("Order")),
    ])?;
    add(&mut session, "command", "PlaceOrder", vec![
        ("label", json!("Place order")), ("targets", json!("Order")), ("emits", json!(["OrderPlaced"])),
    ])?;
    add(&mut session, "read-model", "OrderSummary",
        vec![("label", json!("Order summary")), ("projects", json!(["OrderPlaced"]))])?;

    session.save(&dir).map_err(|e| ProductError::IoError(format!("demo: save: {e}")))?;
    Ok(6)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_a_conformant_bookstore() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let n = seed_bookstore(tmp.path(), "bookstore").expect("seed");
        assert_eq!(n, 6);
        // The saved session reloads and is conformant.
        let dir = crate::author::domain::session_dir(tmp.path(), "bookstore");
        let session = DomainSession::load(&dir).expect("reload");
        assert!(crate::pf::validate::validate_graph(&session.graph).is_empty());
        assert_eq!(session.graph.counts().iter().map(|(_, c)| *c).sum::<usize>(), 6);
    }
}
