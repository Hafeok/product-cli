//! `product csharp …` over the committed fixture inventory (the .NET reader's
//! output for `tests/fixtures/csharp-inventory/Fixture`). CI has no .NET SDK
//! (`dec/ddd/fixtures-not-sdk`); the fixture is regenerated locally with the
//! invocation in `tests/fixtures/csharp-inventory/README.md`.

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/csharp-inventory").join(name)
}

fn product() -> Command {
    Command::cargo_bin("product").expect("binary")
}

#[test]
fn inventory_check_reports_counts() {
    product()
        .args(["csharp", "inventory"])
        .arg(fixture("inventory.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("inventory version 1"))
        .stdout(predicate::str::contains("3 project(s)"));
}

#[test]
fn unknown_inventory_version_is_refused() {
    let dir = tempfile::tempdir().expect("tmp");
    let path = dir.path().join("inv.json");
    std::fs::write(&path, r#"{"inventory_version":"9"}"#).expect("write");
    product()
        .args(["csharp", "inventory"])
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("'9' is not known"));
}

#[test]
fn reach_prints_its_convention_first() {
    product()
        .args(["csharp", "reach", "--roots", "entry-point,public"])
        .arg(fixture("inventory.json"))
        .assert()
        .success()
        .stdout(predicate::str::starts_with("roots: entry-point, public\nthrough implementations (DI): false\n"))
        .stdout(predicate::str::contains("by namespace:"));
}

#[test]
fn reach_json_carries_the_flag() {
    product()
        .args(["--format", "json", "csharp", "reach", "--through-implementations"])
        .arg(fixture("inventory.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("\"through_implementations\":true"));
}

#[test]
fn delta_reports_by_act_with_the_proxy_stated() {
    product()
        .args(["csharp", "delta", "--event-model"])
        .arg(fixture("ordering.eventmodel.yaml"))
        .arg(fixture("inventory.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("known divergence"))
        .stdout(predicate::str::contains("command:PlaceOrder"))
        .stdout(predicate::str::contains("boundary runs through T:Shop.Api.Orders.CheckoutService"))
        .stdout(predicate::str::contains("declares act 'Nonexistent'"));
}
