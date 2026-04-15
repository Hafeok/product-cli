//! Schema migration for product.toml (extracted from config.rs for file-length compliance)

use crate::config::{ProductConfig, CURRENT_SCHEMA_VERSION};
use std::path::Path;

/// Run schema migration from current version to CURRENT_SCHEMA_VERSION.
/// Returns (files_updated, files_unchanged).
pub fn migrate_schema(
    root: &Path,
    config: &ProductConfig,
    dry_run: bool,
) -> crate::error::Result<(usize, usize)> {
    let version: u32 = config.schema_version.parse().unwrap_or(0);
    if version >= CURRENT_SCHEMA_VERSION {
        return Ok((0, 0));
    }

    let mut updated = 0;
    let mut unchanged = 0;

    // v0 → v1: add depends-on field to feature files missing it
    if version < 1 {
        let features_dir = config.resolve_path(root, &config.paths.features);
        if features_dir.exists() {
            for entry in std::fs::read_dir(&features_dir)
                .map_err(|e| crate::error::ProductError::IoError(e.to_string()))?
                .flatten()
            {
                if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                    let content = std::fs::read_to_string(entry.path())
                        .map_err(|e| crate::error::ProductError::IoError(e.to_string()))?;
                    if !content.contains("depends-on:") {
                        if dry_run {
                            println!("  would add depends-on: [] to {}", entry.path().display());
                            updated += 1;
                        } else {
                            let new_content = content.replace(
                                "\nstatus:",
                                "\ndepends-on: []\nstatus:",
                            );
                            if new_content != content {
                                crate::fileops::write_file_atomic(&entry.path(), &new_content)?;
                                updated += 1;
                                println!("  updated: {}", entry.path().display());
                            } else {
                                let new_content2 = content.replace(
                                    "\nphase:",
                                    "\nphase: \ndepends-on: []",
                                );
                                if new_content2 != content {
                                    crate::fileops::write_file_atomic(&entry.path(), &new_content2)?;
                                    updated += 1;
                                } else {
                                    unchanged += 1;
                                }
                            }
                        }
                    } else {
                        unchanged += 1;
                    }
                }
            }
        }

        if !dry_run {
            let toml_path = root.join("product.toml");
            if toml_path.exists() {
                let content = std::fs::read_to_string(&toml_path)
                    .map_err(|e| crate::error::ProductError::IoError(e.to_string()))?;
                let new_content = content.replace(
                    &format!("schema-version = \"{}\"", version),
                    &format!("schema-version = \"{}\"", CURRENT_SCHEMA_VERSION),
                );
                crate::fileops::write_file_atomic(&toml_path, &new_content)?;
                println!("  updated product.toml schema-version to {}", CURRENT_SCHEMA_VERSION);
            }
        }
    }

    Ok((updated, unchanged))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn migrate_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let tp = dir.path().join("product.toml");
        std::fs::write(&tp, "name = \"test\"\nschema-version = \"0\"\n\n[paths]\nfeatures = \"f\"\n").unwrap();
        std::fs::create_dir_all(dir.path().join("f")).unwrap();
        std::fs::write(dir.path().join("f/FT-001-test.md"),
            "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\nadrs: []\ntests: []\n---\n\nBody.\n").unwrap();
        (dir, tp)
    }
    #[test]
    fn schema_migrate_v0_dry_run() {
        let (dir, tp) = migrate_fixture();
        let cfg = ProductConfig::load(&tp).unwrap();
        let (updated, _) = migrate_schema(dir.path(), &cfg, true).unwrap();
        assert!(updated > 0, "dry-run should report files to update");
        let c = std::fs::read_to_string(dir.path().join("f/FT-001-test.md")).unwrap();
        assert!(!c.contains("depends-on"), "dry-run should not modify files");
    }
    #[test]
    fn schema_migrate_v0_execute() {
        let (dir, tp) = migrate_fixture();
        let cfg = ProductConfig::load(&tp).unwrap();
        let (updated, _) = migrate_schema(dir.path(), &cfg, false).unwrap();
        assert!(updated > 0);
        let c = std::fs::read_to_string(dir.path().join("f/FT-001-test.md")).unwrap();
        assert!(c.contains("depends-on"), "file should now have depends-on");
        let t = std::fs::read_to_string(&tp).unwrap();
        assert!(t.contains("schema-version = \"1\""), "product.toml should be bumped to v1");
    }
    #[test]
    fn schema_migrate_already_current() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("product.toml"), "name = \"test\"\nschema-version = \"1\"\n").unwrap();
        let cfg = ProductConfig::load(&dir.path().join("product.toml")).unwrap();
        let (updated, _) = migrate_schema(dir.path(), &cfg, false).unwrap();
        assert_eq!(updated, 0, "no migration needed for current version");
    }
}
