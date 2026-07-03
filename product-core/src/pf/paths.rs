//! Where a product's How / Delivery / Build artifacts live on disk (§ per-product
//! scoping). The repo's **root product** — the one named in `product.toml` —
//! keeps the shared `.product/` (back-compat for the self-hosted product-cli);
//! **every other product** is scoped to `.product/products/<name>/`, so a
//! showcase like acme carries its own blueprint, DeployableUnits, deliverables,
//! deciders, work units, and how-contract without colliding with the root.
//!
//! Both the authoring surface (CLI + MCP writes) and the explorer projection
//! resolve through here, so writes and reads always agree.

use std::path::{Path, PathBuf};

use crate::config::ProductConfig;

/// The `.product` base directory for `product`'s artifacts under `repo_root`.
pub fn product_base(repo_root: &Path, product: &str) -> PathBuf {
    let pd = repo_root.join(".product");
    let product = product.trim();
    if product.is_empty() {
        return pd;
    }
    match ProductConfig::load_from_root(repo_root).ok() {
        // the root product (product.toml name) keeps the shared .product/
        Some(c) if c.name.trim() == product => pd,
        // every other product is scoped
        _ => pd.join("products").join(product),
    }
}
