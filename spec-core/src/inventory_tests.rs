use super::*;

pub(crate) const SAMPLE: &str = r#"{
  "form": "spec.inventory.v1",
  "root": "/tmp/x",
  "revision": "abc123",
  "scanned_at": "2026-09-15T00:00:00+00:00",
  "symbols": [
    {"id":"Shop.Api.BasketController","kind":"class","name":"BasketController",
     "namespace":"Shop.Api","file":"src/BasketController.cs","line":6}
  ],
  "composition_edges": [
    {"from":"Shop.Api.BasketController","to":"IBasketStore","via":"ctor-param"}
  ],
  "entry_points": [
    {"id":"Shop.Api.BasketController.Settle#HttpPost","kind":"http-route",
     "symbol":"Shop.Api.BasketController.Settle","transport":"HttpPost /baskets/{id}/settle",
     "file":"src/BasketController.cs","line":10}
  ],
  "candidates": [
    {"id":"cand/shop-api-basketcontroller-settle-httppost",
     "entry_point":"Shop.Api.BasketController.Settle#HttpPost",
     "observed":{"kind":"http-route","transport":"HttpPost /baskets/{id}/settle"},
     "unfilled_slots":["name","settles"]}
  ],
  "claims": [
    {"kind":"slice","value":"checkout-totals","symbol":"Shop.Api.BasketController",
     "file":"src/BasketController.cs","line":6},
    {"kind":"realises-fact","value":"det/basket-rounding-is-half-even",
     "symbol":"Shop.Api.BasketController.Settle","file":"src/BasketController.cs","line":10}
  ]
}"#;

pub(crate) fn sample() -> Inventory {
    serde_json::from_str(SAMPLE).expect("the sample inventory parses")
}

pub(crate) fn write_sample(repo_root: &std::path::Path) {
    let path = Inventory::path(repo_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, SAMPLE).expect("write inventory");
}

#[test]
fn the_wire_form_round_trips() {
    let inventory = sample();
    assert_eq!(inventory.form, "spec.inventory.v1");
    assert_eq!(inventory.entry_points.len(), 1);
    assert_eq!(inventory.candidates[0].unfilled_slots, ["name", "settles"]);
}

#[test]
fn an_absent_inventory_is_not_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(Inventory::load_opt(dir.path()).expect("load").is_none());
}

#[test]
fn a_written_inventory_reads_back() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_sample(dir.path());
    let loaded = Inventory::load_opt(dir.path()).expect("load").expect("present");
    assert_eq!(loaded, sample());
}

#[test]
fn a_malformed_inventory_names_its_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = Inventory::path(dir.path());
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&path, "{ not json").expect("write");
    let err = Inventory::load_opt(dir.path()).expect_err("malformed");
    assert!(err.to_string().contains("inventory.json"), "{err}");
}

#[test]
fn claims_are_read_back_by_kind() {
    let inventory = sample();
    assert_eq!(inventory.claims_of("slice").len(), 1);
    assert_eq!(inventory.claims_of("realises-fact")[0].value, "det/basket-rounding-is-half-even");
    assert!(inventory.claims_of("invented-kind").is_empty());
}

#[test]
fn an_inventory_written_before_claims_existed_still_parses() {
    let older = r#"{"form":"spec.inventory.v1","root":".","revision":"r","scanned_at":"t"}"#;
    let parsed: Inventory = serde_json::from_str(older).expect("older inventories stay readable");
    assert!(parsed.claims.is_empty());
    assert!(parsed.entry_points.is_empty());
}

#[test]
fn entry_points_and_symbols_look_up() {
    let inventory = sample();
    assert!(inventory.entry_point("Shop.Api.BasketController.Settle#HttpPost").is_some());
    assert!(inventory.entry_point("nope").is_none());
    assert_eq!(inventory.symbols_in("src/BasketController.cs").len(), 1);
}
