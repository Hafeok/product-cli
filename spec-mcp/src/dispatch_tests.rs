use super::*;

const INVENTORY: &str = r#"{
  "form": "spec.inventory.v1",
  "root": ".",
  "revision": "abc123",
  "scanned_at": "2026-09-15T00:00:00+00:00",
  "entry_points": [
    {"id":"ep/settle","kind":"http-route","symbol":"Settle",
     "transport":"HttpPost /settle","file":"src/Api.cs","line":4}
  ],
  "candidates": [
    {"id":"cand/settle","entry_point":"ep/settle",
     "observed":{"transport":"HttpPost /settle"},"unfilled_slots":["name","settles"]}
  ]
}"#;

fn repo() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn call(name: &str, args: &Value, root: &Path) -> Option<Result<Value, String>> {
    dispatch(name, args, root)
}

#[test]
fn every_withheld_verb_is_refused_by_name() {
    let dir = repo();
    for withheld in WITHHELD {
        let answer = call(withheld, &json!({}), dir.path())
            .expect("the dispatcher must answer, not fall through");
        let message = answer.expect_err("a withheld verb is refused");
        assert!(message.contains("names a principal"), "{withheld}: {message}");
    }
}

#[test]
fn a_verb_this_surface_does_not_own_falls_through() {
    let dir = repo();
    assert!(call("product_domain_new", &json!({}), dir.path()).is_none());
}

#[test]
fn candidates_reads_empty_without_an_inventory() {
    let dir = repo();
    let value = call("spec_candidates", &json!({}), dir.path())
        .expect("ours")
        .expect("reads");
    assert_eq!(value["candidates"].as_array().map(Vec::len), Some(0));
    assert!(value["note"].as_str().is_some_and(|n| n.contains("import")));
}

#[test]
fn candidates_tells_the_model_that_ratifying_is_not_its_job() {
    let dir = repo();
    std::fs::create_dir_all(dir.path().join(".spec")).expect("mkdir");
    std::fs::write(dir.path().join(".spec/inventory.json"), INVENTORY).expect("write");

    let value = call("spec_candidates", &json!({}), dir.path())
        .expect("ours")
        .expect("reads");
    let note = value["note"].as_str().unwrap_or_default();
    assert!(note.contains("by a person"), "{note}");
    assert!(note.contains("do not claim to have filed"), "{note}");
}

#[test]
fn check_returns_the_two_verdict_classes_apart() {
    let dir = repo();
    let value = call("spec_check", &json!({}), dir.path()).expect("ours").expect("reads");
    assert!(value.get("structural").is_some());
    assert!(value.get("policy").is_some());
    assert!(value.get("metrics").is_some());
}

#[test]
fn implement_opens_a_pending_record_and_hands_the_closure_over() {
    let dir = repo();
    let value = call(
        "spec_implement",
        &json!({"slice": "checkout-totals", "act_ref": "act/settle", "by": "agent@example.invalid"}),
        dir.path(),
    )
    .expect("ours")
    .expect("writes");

    assert_eq!(value["status"], "pending-closure");
    let hand_off = value["hand_off"].as_str().unwrap_or_default();
    assert!(hand_off.starts_with("spec close "), "{hand_off}");
    assert!(value["note"].as_str().is_some_and(|n| n.contains("not yours to do")));
}

#[test]
fn implement_requires_the_fields_that_make_a_record_meaningful() {
    let dir = repo();
    let message = call("spec_implement", &json!({"slice": "x"}), dir.path())
        .expect("ours")
        .expect_err("act_ref is required");
    assert!(message.contains("act_ref"), "{message}");
}

#[test]
fn the_record_implement_opened_is_visible_and_open() {
    let dir = repo();
    call(
        "spec_implement",
        &json!({"slice": "s", "act_ref": "act/a", "by": "agent@example.invalid"}),
        dir.path(),
    )
    .expect("ours")
    .expect("writes");

    let value = call("spec_records", &json!({"open": true}), dir.path())
        .expect("ours")
        .expect("reads");
    assert_eq!(value["records"].as_array().map(Vec::len), Some(1));
    assert_eq!(value["records"][0]["open"], true);
}

#[test]
fn policy_show_reports_the_default_as_structural_only() {
    let dir = repo();
    let value = call("spec_policy_show", &json!({}), dir.path()).expect("ours").expect("reads");
    assert!(value["in_force"].is_null());
    assert!(value["note"].as_str().is_some_and(|n| n.contains("structural verdicts only")));
}
