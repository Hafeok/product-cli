//! Drift tests binding the embedded template to the tree on disk.

use super::*;
use std::path::PathBuf;

fn template_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(TEMPLATE_DIR)
}

#[test]
fn the_embedded_manifest_matches_the_tree_on_disk() {
    let dir = template_dir();
    let mut on_disk = Vec::new();
    walk(&dir, &dir, &mut on_disk);
    on_disk.sort();
    let mut embedded: Vec<String> = TEMPLATE.iter().map(|f| f.path.to_string()).collect();
    embedded.sort();
    assert_eq!(
        on_disk, embedded,
        "the template tree and the embedded manifest disagree — wire the new file into template.rs"
    );
}

#[test]
fn embedded_bytes_match_the_files_on_disk() {
    for f in TEMPLATE {
        let disk = std::fs::read_to_string(template_dir().join(f.path)).expect("read template file");
        assert_eq!(disk, f.text, "{} drifted from its embedded copy", f.path);
    }
}

#[test]
fn the_version_constant_matches_the_template_header() {
    let header = std::fs::read_to_string(template_dir().join("TEMPLATE.md")).expect("read TEMPLATE.md");
    let declared = header
        .lines()
        .find_map(|l| l.strip_prefix("**Template version: "))
        .and_then(|l| l.split('*').next())
        .map(|v| v.trim().to_string())
        .expect("TEMPLATE.md declares a version");
    assert_eq!(declared, TEMPLATE_VERSION);
}

#[test]
fn template_md_is_the_only_verbatim_file() {
    let verbatim: Vec<&str> = TEMPLATE.iter().filter(|f| !f.substituted).map(|f| f.path).collect();
    assert_eq!(verbatim, vec!["TEMPLATE.md"]);
}

fn walk(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("read template dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, out);
        } else if let Ok(rel) = path.strip_prefix(root) {
            out.push(rel.display().to_string());
        }
    }
}
