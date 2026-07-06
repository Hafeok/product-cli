//! Where a product's artifacts live on disk (§ per-product scoping). Every
//! product — the one named at `product init` and every one added after it —
//! has a single home: `.product/products/<name>/`, holding its What graph
//! (`<name>.ttl` + session cache) alongside its How/Delivery/Build artifacts
//! (blueprints, DeployableUnits, deliverables, deciders, work units,
//! how-contract). `product product list/new/show` manages these homes.
//!
//! Two legacy layouts still resolve, so pre-`products/` repos keep working
//! unmigrated: a root product whose artifacts sit directly under `.product/`,
//! and What graphs under `.product/author-domain/<name>/`. The fallback only
//! triggers when the scoped home does not exist yet, so a migrated (or fresh)
//! repo always resolves to `products/<name>/`.
//!
//! Both the authoring surface (CLI + MCP writes) and the explorer projection
//! resolve through here, so writes and reads always agree.

use std::path::{Path, PathBuf};

use crate::config::ProductConfig;

/// The `.product/products/<name>` home for `product` under `repo_root`,
/// regardless of whether it exists yet.
pub fn product_home(repo_root: &Path, product: &str) -> PathBuf {
    repo_root.join(".product").join("products").join(product.trim())
}

/// The `.product` base directory for `product`'s artifacts under `repo_root`:
/// `.product/products/<name>/`. Falls back to the legacy shared `.product/`
/// when the scoped home does not exist and the repo's root product (the
/// config `name`) left its artifacts there pre-migration.
pub fn product_base(repo_root: &Path, product: &str) -> PathBuf {
    let pd = repo_root.join(".product");
    let product = product.trim();
    if product.is_empty() {
        return pd;
    }
    let scoped = product_home(repo_root, product);
    if scoped.exists() {
        return scoped;
    }
    if is_root_product(repo_root, product) && has_legacy_root_artifacts(&pd) {
        return pd;
    }
    scoped
}

/// Whether `product` is the repo's configured root product (config `name`).
fn is_root_product(repo_root: &Path, product: &str) -> bool {
    ProductConfig::load_from_root(repo_root)
        .map(|c| c.name.trim() == product)
        .unwrap_or(false)
}

/// Whether the shared `.product/` still carries pre-`products/` root-product
/// artifacts (the legacy layout this module migrated away from).
fn has_legacy_root_artifacts(pd: &Path) -> bool {
    const MARKERS: [&str; 13] = [
        "how-contract.yaml",
        "deciders",
        "projectors",
        "primitives",
        "features",
        "deliverables",
        "releases",
        "targets",
        "blueprints",
        "archetypes",
        "deployable-units",
        "design-systems",
        "work-units",
    ];
    MARKERS.iter().any(|m| pd.join(m).exists())
}

/// Every product with a home under `.product/products/` or a legacy What graph
/// under `.product/author-domain/`, plus the configured root product name —
/// sorted, deduplicated.
pub fn list_products(repo_root: &Path) -> Vec<String> {
    let pd = repo_root.join(".product");
    let mut names: Vec<String> = Vec::new();
    for dir in [pd.join("products"), pd.join("author-domain")] {
        if let Ok(it) = std::fs::read_dir(&dir) {
            names.extend(
                it.flatten()
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().into_string().ok()),
            );
        }
    }
    if let Ok(c) = ProductConfig::load_from_root(repo_root) {
        let name = c.name.trim().to_string();
        if !name.is_empty() {
            names.push(name);
        }
    }
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_config(root: &Path, name: &str) {
        std::fs::create_dir_all(root.join(".product")).expect("mkdir");
        std::fs::write(root.join(".product/config.toml"), format!("name = \"{name}\"\n"))
            .expect("config");
    }

    #[test]
    fn every_product_scopes_to_its_products_home() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_config(dir.path(), "root");
        assert!(product_base(dir.path(), "root").ends_with(".product/products/root"));
        assert!(product_base(dir.path(), "acme").ends_with(".product/products/acme"));
    }

    #[test]
    fn legacy_root_artifacts_keep_resolving_to_shared_dot_product() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_config(dir.path(), "root");
        std::fs::create_dir_all(dir.path().join(".product/deciders")).expect("mkdir");
        assert!(product_base(dir.path(), "root").ends_with(".product"));
        // Other products stay scoped even in a legacy repo.
        assert!(product_base(dir.path(), "acme").ends_with(".product/products/acme"));
    }

    #[test]
    fn an_existing_home_wins_over_the_legacy_fallback() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_config(dir.path(), "root");
        std::fs::create_dir_all(dir.path().join(".product/deciders")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join(".product/products/root")).expect("mkdir");
        assert!(product_base(dir.path(), "root").ends_with(".product/products/root"));
    }

    #[test]
    fn empty_product_falls_back_to_shared_dot_product() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(product_base(dir.path(), "  ").ends_with(".product"));
    }

    #[test]
    fn list_products_unions_homes_legacy_graphs_and_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_config(dir.path(), "root");
        std::fs::create_dir_all(dir.path().join(".product/products/acme")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join(".product/author-domain/legacy")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join(".product/author-domain/acme")).expect("mkdir");
        assert_eq!(list_products(dir.path()), vec!["acme", "legacy", "root"]);
    }
}
