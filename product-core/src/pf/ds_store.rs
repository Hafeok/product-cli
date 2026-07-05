//! Design-system artifact store — manifests vendored under `.product/design-systems/`.
//!
//! Makes a §11 design system an addressable artifact, parallel to blueprints
//! and deliverables: `save` validates a manifest's declaration half and vendors
//! it (plus every implementation source it references) into
//! `<product base>/design-systems/<id>/`; `load` reads one back by id. The How
//! contract binds the chosen system by id (+ version), and reify resolves the
//! binding through here.

use std::path::{Path, PathBuf};

use crate::error::{ProductError, Result};
use crate::fileops::write_file_atomic;

use super::manifest::{parse_ds, validate_ds, DsManifest};
use super::provenance::content_hash;

pub const MANIFEST_FILE: &str = "design-system.manifest.yaml";

/// A stored design system, loaded back by id: the parsed manifest, its raw
/// YAML (the hash input), and the store directory its bundle files live under.
#[derive(Debug)]
pub struct StoredDs {
    pub manifest: DsManifest,
    pub source: String,
    pub dir: PathBuf,
}

impl StoredDs {
    /// Hex SHA-256 over the manifest YAML — the identity `reify check` pins.
    pub fn hash(&self) -> String {
        content_hash(&self.source)
    }
}

/// `<product base>/design-systems` for a product base directory.
pub fn store_root(product_base: &Path) -> PathBuf {
    product_base.join("design-systems")
}

/// Vendor a manifest (read from `manifest_path`) into the store. Validates the
/// declaration half first — an unwhole manifest is rejected, nothing is saved.
/// Copies the manifest plus every implementation `source`/`preview` file it
/// references (resolved beside the manifest). Returns the stored system.
pub fn save(product_base: &Path, manifest_path: &Path) -> Result<StoredDs> {
    let source = std::fs::read_to_string(manifest_path)
        .map_err(|e| ProductError::IoError(format!("{}: {e}", manifest_path.display())))?;
    let manifest = parse_ds(&source).map_err(ProductError::ConfigError)?;
    let findings = validate_ds(&manifest);
    if !findings.is_empty() {
        return Err(ProductError::ConfigError(format!(
            "manifest is not whole (§11.3) — nothing saved:\n  - {}",
            findings.join("\n  - ")
        )));
    }
    let base = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let dir = store_root(product_base).join(&manifest.design_system.id);
    write_file_atomic(&dir.join(MANIFEST_FILE), &source)?;
    for rel in referenced_files(&manifest) {
        vendor_file(base, &dir, &rel)?;
    }
    Ok(StoredDs { manifest, source, dir })
}

/// Every relative bundle file the manifest references (implementation
/// sources + previews), deduplicated in order.
fn referenced_files(m: &DsManifest) -> Vec<String> {
    let mut out = Vec::new();
    for c in &m.design_system.components {
        for imp in c.implementation.values() {
            for rel in std::iter::once(&imp.source).chain(imp.preview.iter()) {
                if !out.contains(rel) {
                    out.push(rel.clone());
                }
            }
        }
    }
    out
}

/// Copy one referenced bundle file into the store, preserving its relative
/// path. A missing source is tolerated (the bundle check reports it); an
/// escaping path is not.
fn vendor_file(base: &Path, dir: &Path, rel: &str) -> Result<()> {
    if rel.starts_with('/') || rel.split('/').any(|seg| seg == "..") {
        return Err(ProductError::ConfigError(format!(
            "implementation path '{rel}' must be relative and stay beside the manifest"
        )));
    }
    let src = base.join(rel);
    let Ok(content) = std::fs::read_to_string(&src) else { return Ok(()) };
    write_file_atomic(&dir.join(rel), &content)
}

/// Load a stored design system by id.
pub fn load(product_base: &Path, id: &str) -> Result<StoredDs> {
    let dir = store_root(product_base).join(id);
    let path = dir.join(MANIFEST_FILE);
    let source = std::fs::read_to_string(&path).map_err(|_| {
        ProductError::NotFound(format!(
            "no design system '{id}' — add one with `product design-system add <manifest>`"
        ))
    })?;
    let manifest = parse_ds(&source).map_err(ProductError::ConfigError)?;
    Ok(StoredDs { manifest, source, dir })
}

/// The ids of every stored design system, sorted.
pub fn list(product_base: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(store_root(product_base)) else { return Vec::new() };
    let mut out: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().join(MANIFEST_FILE).exists())
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHOLE: &str = "design_system:\n  id: acme\n  version: \"1.0\"\n  components:\n    - id: rail\n      implementation:\n        web: { source: components/Rail.jsx }\n  tokens:\n    - { id: color.fg, type: color }\n";

    #[test]
    fn save_vendors_manifest_and_sources_then_load_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let src_dir = dir.path().join("srcds");
        std::fs::create_dir_all(src_dir.join("components")).expect("mkdir");
        std::fs::write(src_dir.join(MANIFEST_FILE), WHOLE).expect("write");
        std::fs::write(src_dir.join("components/Rail.jsx"), "rail").expect("write");
        let base = dir.path().join(".product");
        let saved = save(&base, &src_dir.join(MANIFEST_FILE)).expect("save");
        assert_eq!(saved.manifest.design_system.id, "acme");
        assert!(saved.dir.join("components/Rail.jsx").exists(), "source is vendored");
        let loaded = load(&base, "acme").expect("load");
        assert_eq!(loaded.source, WHOLE);
        assert_eq!(loaded.hash(), saved.hash());
        assert_eq!(list(&base), vec!["acme".to_string()]);
    }

    #[test]
    fn unwhole_manifest_is_rejected_and_nothing_is_saved() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bad = "design_system:\n  id: bad\n  components:\n    - id: rail\n      tokens: [ghost.token]\n";
        let p = dir.path().join(MANIFEST_FILE);
        std::fs::write(&p, bad).expect("write");
        let base = dir.path().join(".product");
        assert!(save(&base, &p).is_err());
        assert!(list(&base).is_empty());
    }

    #[test]
    fn escaping_implementation_path_is_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bad = "design_system:\n  id: esc\n  components:\n    - id: rail\n      implementation:\n        web: { source: ../../etc/passwd }\n";
        let p = dir.path().join(MANIFEST_FILE);
        std::fs::write(&p, bad).expect("write");
        assert!(save(&dir.path().join(".product"), &p).is_err());
    }

    #[test]
    fn load_of_unknown_id_names_the_add_command() {
        let dir = tempfile::tempdir().expect("tempdir");
        let e = load(dir.path(), "ghost").expect_err("miss");
        assert!(format!("{e}").contains("design-system add"));
    }
}
