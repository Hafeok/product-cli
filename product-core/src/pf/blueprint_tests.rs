//! Tests for blueprint assembly + whole-blueprint validation.

use super::*;
use crate::pf::how_validate::has_blocking;

const HOW: &str = include_str!("../../../schema/examples/how-contract.example.yaml");
const LAYOUT: &str = include_str!("../../../schema/examples/layout-model.example.yaml");
const CELL: &str = include_str!("../../../schema/examples/task-type-definition.example.yaml");

/// Write a full example blueprint tree under a tempdir and return its dir.
fn write_blueprint(name: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join(name);
    std::fs::create_dir_all(root.join("cells")).expect("mkdir");
    std::fs::write(root.join("how-contract.yaml"), HOW).expect("how");
    std::fs::write(root.join("layout.yaml"), LAYOUT).expect("layout");
    std::fs::write(root.join("cells").join("add-crud-resource.yaml"), CELL).expect("cell");
    dir
}

#[test]
fn loads_and_validates_a_full_blueprint() {
    let tmp = write_blueprint("example-rest-api");
    let arch = Blueprint::load_from_dir(&tmp.path().join("example-rest-api"), "example-rest-api").expect("load");
    assert!(arch.how.is_some());
    assert!(arch.layout.is_some());
    assert_eq!(arch.cells.len(), 1);
    let results = arch.validate(None);
    assert!(!has_blocking(&results), "unexpected blocking: {:?}", results.iter().filter(|v| v.severity == "violation").collect::<Vec<_>>());
}

#[test]
fn how_contract_ref_resolves_to_a_shared_contract() {
    // The blueprint's how-contract.yaml is a `ref:` stub pointing at a shared
    // contract one directory up — it must load and validate as if inline.
    let tmp = write_blueprint("example-rest-api");
    let arch_dir = tmp.path().join("example-rest-api");
    std::fs::write(tmp.path().join("shared-how.yaml"), HOW).expect("shared how");
    std::fs::write(arch_dir.join("how-contract.yaml"), "ref: ../shared-how.yaml\n").expect("ref stub");

    let arch = Blueprint::load_from_dir(&arch_dir, "example-rest-api").expect("load");
    assert!(arch.how.is_some(), "ref stub must resolve to the shared contract");
    let results = arch.validate(None);
    assert!(!has_blocking(&results), "referenced contract validates: {:?}",
        results.iter().filter(|v| v.severity == "violation").collect::<Vec<_>>());
}

#[test]
fn missing_how_is_a_blocking_violation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("bare");
    std::fs::create_dir_all(&root).expect("mkdir");
    let arch = Blueprint::load_from_dir(&root, "bare").expect("load");
    let results = arch.validate(None);
    assert!(has_blocking(&results));
    assert!(results.iter().any(|v| v.path == "how" && v.severity == "violation"));
}

#[test]
fn cell_with_mismatched_blueprint_warns() {
    let tmp = write_blueprint("example-rest-api");
    // load under a different name → the cell's blueprint field no longer matches
    let arch = Blueprint::load_from_dir(&tmp.path().join("example-rest-api"), "other-arch").expect("load");
    let results = arch.validate(None);
    assert!(results.iter().any(|v| v.severity == "warning" && v.focus.starts_with("add-crud-resource/") == false && v.path == "blueprint"));
}

#[test]
fn part_violations_are_tagged_with_their_source() {
    let tmp = write_blueprint("a");
    let root = tmp.path().join("a");
    // break the layout: a rule loses its enforces
    std::fs::write(root.join("layout.yaml"), LAYOUT.replacen("    enforces: [explicit-composition-root]\n", "", 1)).expect("w");
    let arch = Blueprint::load_from_dir(&root, "a").expect("load");
    let results = arch.validate(None);
    assert!(results.iter().any(|v| v.focus.starts_with("layout/") && v.path == "enforces"));
}

#[test]
fn missing_dir_is_an_error() {
    assert!(Blueprint::load_from_dir(std::path::Path::new("/no/such/blueprint"), "x").is_err());
}

#[test]
fn layout_rule_enforcing_a_ghost_principle_warns() {
    let tmp = write_blueprint("a");
    let root = tmp.path().join("a");
    // a layout rule that enforces a principle the How never defines
    std::fs::write(root.join("layout.yaml"),
        "version: \"1\"\nblueprint: a\nlayout:\n  - id: r\n    may_exist_here: \"src/**\"\n    enforces: [ghost-principle]\n").expect("w");
    let arch = Blueprint::load_from_dir(&root, "a").expect("load");
    let results = arch.validate(None);
    assert!(results.iter().any(|v| v.severity == "warning" && v.path == "enforces" && v.message.contains("ghost-principle")),
        "expected a dangling-enforces warning: {results:?}");
}
