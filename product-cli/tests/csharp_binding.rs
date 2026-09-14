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
        .stdout(predicate::str::contains("inventory version 4"))
        .stdout(predicate::str::contains("4 project(s)"));
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
        .stdout(predicate::str::starts_with("roots: entry-point, public\n"))
        .stdout(predicate::str::contains("of the denominator"))
        .stdout(predicate::str::contains("of composition edges"))
        .stdout(predicate::str::contains("by namespace:"));
}

#[test]
fn reach_measures_a_ground_truth_when_given_one() {
    product()
        .args(["csharp", "reach", "--roots", "entry-point,implements:T:Microsoft.AspNetCore.Components.ComponentBase", "--ground-truth"])
        .arg(fixture("ground-truth.yaml"))
        .arg(fixture("inventory.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("ground truth (Shop.Api, 20 edges"))
        .stdout(predicate::str::contains("reader recall 20/20 (100.0%)"));
}

#[test]
fn reach_json_carries_resolution_and_tracked_sets() {
    product()
        .args(["--format", "json", "csharp", "reach", "--track", "implements:T:Shop.Api.Infrastructure.IHandler`1"])
        .arg(fixture("inventory.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("\"coverage_percent\""))
        .stdout(predicate::str::contains("\"tracked\":[{\"label\":\"implements:T:Shop.Api.Infrastructure.IHandler`1\""));
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
