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
        .stdout(predicate::str::contains("inventory version 6"))
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
        .stdout(predicate::str::contains("scored fraction: "))
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
        .stdout(predicate::str::contains("ground truth (Shop.Api, 24 edges"))
        .stdout(predicate::str::contains("reader recall 24/24 (100.0%) over C# source, Razor views not covered (CG-R-78)"));
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

#[test]
fn candidates_print_the_vocabulary_label_and_the_fixture_yields_none() {
    let out = product().args(["csharp", "candidates"]).arg(fixture("inventory.json")).assert().success().get_output().stdout.clone();
    let text = String::from_utf8_lossy(&out);
    assert!(text.starts_with("vocabulary: measurement vocabulary, transport-derived (CG-R-105)"), "{text}");
    assert!(text.contains("candidates: 0 (transport-derived)"), "the fixture has a Blazor component, no controller, page, ViewComponent, FastEndpoints endpoint or hosted service: {text}");
    assert!(text.contains("P-EP-1") && text.contains("L-EP-1") && text.contains("reported, not repaired"), "{text}");
}

#[test]
fn candidates_json_carries_proxies_limits_and_empty_slots() {
    let out = product().args(["--format", "json", "csharp", "candidates"]).arg(fixture("inventory.json")).assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).expect("json");
    assert_eq!(v["count"], 0);
    assert_eq!(v["proxies"].as_array().map(Vec::len), Some(4));
    assert_eq!(v["limits"].as_array().map(Vec::len), Some(3));
    assert!(v["recall"].is_null());
}
