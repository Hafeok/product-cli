//! Tests for layout-conformance against a real filesystem tree.

use super::*;
use crate::pf::layout::LayoutModel;

/// Build a small repo tree under a tempdir.
fn tree(files: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for f in files {
        let p = dir.path().join(f);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(p, "x").expect("write");
    }
    dir
}

fn model(yaml: &str) -> LayoutModel {
    LayoutModel::from_yaml(yaml).expect("parse layout")
}

#[test]
fn must_exist_passes_and_fails() {
    let t = tree(&["schema/shapes/shapes.shacl.ttl"]);
    let ok = model("version: \"1\"\nlayout:\n  - id: shapes\n    must_exist: \"schema/shapes/shapes.shacl.ttl\"\n    cardinality: \"exactly 1\"\n    enforces: [x]\n");
    assert_eq!(check_layout(&ok, t.path()), vec![]);

    let missing = model("version: \"1\"\nlayout:\n  - id: nope\n    must_exist: \"schema/missing.ttl\"\n    cardinality: \"exactly 1\"\n    enforces: [x]\n");
    assert!(check_layout(&missing, t.path()).iter().any(|v| v.path == "must_exist"));
}

#[test]
fn must_exist_one_per_scope() {
    // each member dir must have a Cargo.toml
    let t = tree(&["product-core/Cargo.toml", "product-cli/Cargo.toml", "product-mcp/src/lib.rs"]);
    let m = model("version: \"1\"\nlayout:\n  - id: manifest\n    for_each: \"product-*\"\n    must_exist: \"{dir}/Cargo.toml\"\n    cardinality: \"1 per scope\"\n    enforces: [x]\n");
    // product-mcp has no Cargo.toml → one violation
    let vs = check_layout(&m, t.path());
    assert_eq!(vs.len(), 1, "{vs:?}");
    assert!(vs[0].message.contains("product-mcp"));
}

#[test]
fn must_not_exist_flags_secrets() {
    let t = tree(&["src/app.secrets.json"]);
    let m = model("version: \"1\"\nlayout:\n  - id: nosec\n    must_not_exist: \"**/*.secrets.*\"\n    rationale: r\n    enforces: [x]\n");
    assert!(check_layout(&m, t.path()).iter().any(|v| v.path == "must_not_exist"));
}

#[test]
fn must_co_exist_requires_siblings() {
    let t = tree(&["product-core/src/pf/mod.rs", "product-core/src/pf/model.rs"]);
    let ok = model("version: \"1\"\nlayout:\n  - id: hasmod\n    must_co_exist:\n      when: \"product-core/src/pf\"\n      require: [\"mod.rs\"]\n    enforces: [x]\n");
    assert_eq!(check_layout(&ok, t.path()), vec![]);

    let bad = model("version: \"1\"\nlayout:\n  - id: needslib\n    must_co_exist:\n      when: \"product-core/src/pf\"\n      require: [\"lib.rs\"]\n    enforces: [x]\n");
    assert!(check_layout(&bad, t.path()).iter().any(|v| v.path == "must_co_exist"));
}

#[test]
fn no_orphans_allowlist() {
    let t = tree(&["product-core/src/pf/a.rs", "product-core/src/pf/b.rs"]);
    // allow rule covers pf/** → no orphans
    let ok = model("version: \"1\"\nlayout:\n  - id: pf-here\n    may_exist_here: \"product-core/src/pf/**\"\n    enforces: [x]\n  - id: orphans\n    no_orphans: \"product-core/src/pf/**\"\n    enforces: [x]\n");
    assert_eq!(check_layout(&ok, t.path()), vec![]);

    // no allow rule → both files are orphans
    let bad = model("version: \"1\"\nlayout:\n  - id: orphans\n    no_orphans: \"product-core/src/pf/**\"\n    enforces: [x]\n");
    let vs = check_layout(&bad, t.path());
    assert_eq!(vs.iter().filter(|v| v.path == "no_orphans").count(), 2, "{vs:?}");
}
