//! Tests for the first-party worker (request, parse, stub, apply).

use super::*;
use serde_json::json;

#[test]
fn build_request_forces_json_output() {
    let req = build_request("fast-cheap", "do the thing");
    assert_eq!(req["model"], json!("fast-cheap"));
    assert_eq!(req["response_format"]["type"], json!("json_object"));
    assert_eq!(req["messages"][1]["content"], json!("do the thing"));
}

#[test]
fn parse_files_reads_the_contract() {
    let obj = json!({ "files": [{ "path": "src/a.rs", "content": "fn a() {}" }] });
    let files = parse_files(&obj).expect("parse");
    assert_eq!(files, vec![ArtifactFile { path: "src/a.rs".into(), content: "fn a() {}".into() }]);
}

#[test]
fn parse_files_rejects_a_missing_array() {
    assert!(parse_files(&json!({ "nope": 1 })).is_err());
}

#[test]
fn stub_files_are_deterministic() {
    let a = stub_files("ctx");
    let b = stub_files("ctx");
    assert_eq!(a, b);
    assert!(a[0].path.starts_with(".product/build/artifacts/STUB-"));
}

#[test]
fn apply_writes_under_root() {
    let dir = tempfile::tempdir().expect("tmp");
    let files = vec![ArtifactFile { path: "src/x.rs".into(), content: "x".into() }];
    let written = apply_files(&files, dir.path()).expect("apply");
    assert_eq!(written.len(), 1);
    assert_eq!(std::fs::read_to_string(dir.path().join("src/x.rs")).expect("read"), "x");
}

#[test]
fn apply_refuses_path_escape() {
    let dir = tempfile::tempdir().expect("tmp");
    let files = vec![ArtifactFile { path: "../evil.rs".into(), content: "x".into() }];
    assert!(apply_files(&files, dir.path()).is_err());
}

#[test]
fn parse_output_accepts_edits_only() {
    let obj = json!({ "edits": [{ "path": "src/mod.rs", "find": "pub mod a;", "replace": "pub mod a;\npub mod b;" }] });
    let (files, edits) = parse_output(&obj).expect("parse");
    assert!(files.is_empty());
    assert_eq!(edits, vec![EditOp { path: "src/mod.rs".into(), find: "pub mod a;".into(), replace: "pub mod a;\npub mod b;".into() }]);
}

#[test]
fn parse_output_rejects_empty_response() {
    assert!(parse_output(&json!({ "junk": 1 })).is_err());
}

#[test]
fn apply_edits_wires_a_unique_span() {
    let dir = tempfile::tempdir().expect("tmp");
    std::fs::write(dir.path().join("mod.rs"), "pub mod a;\npub mod c;\n").expect("seed");
    let edits = vec![EditOp { path: "mod.rs".into(), find: "pub mod a;".into(), replace: "pub mod a;\npub mod b;".into() }];
    apply_edits(&edits, dir.path()).expect("apply");
    assert_eq!(std::fs::read_to_string(dir.path().join("mod.rs")).expect("read"), "pub mod a;\npub mod b;\npub mod c;\n");
}

#[test]
fn apply_edits_refuses_a_missing_target() {
    let dir = tempfile::tempdir().expect("tmp");
    std::fs::write(dir.path().join("mod.rs"), "pub mod a;\n").expect("seed");
    let edits = vec![EditOp { path: "mod.rs".into(), find: "pub mod zzz;".into(), replace: "x".into() }];
    assert!(apply_edits(&edits, dir.path()).is_err());
}

#[test]
fn extract_json_reads_a_raw_object() {
    let v = extract_json("{\"files\":[]}").expect("raw");
    assert!(v.get("files").is_some());
}

#[test]
fn extract_json_unwraps_a_json_fence() {
    let content = "Here is the file:\n```json\n{\"files\":[{\"path\":\"a.rs\",\"content\":\"x\"}]}\n```\nDone.";
    let v = extract_json(content).expect("fenced");
    assert_eq!(v["files"][0]["path"], serde_json::json!("a.rs"));
}

#[test]
fn extract_json_finds_a_prose_wrapped_object() {
    let content = "Sure! The answer is {\"edits\":[{\"path\":\"m.rs\",\"find\":\"a\",\"replace\":\"b\"}]} — hope that helps.";
    let v = extract_json(content).expect("prose");
    assert_eq!(v["edits"][0]["find"], serde_json::json!("a"));
}

#[test]
fn extract_json_ignores_braces_inside_strings() {
    let v = extract_json("text {\"content\":\"fn f() { }\"} tail").expect("strings");
    assert_eq!(v["content"], serde_json::json!("fn f() { }"));
}

#[test]
fn extract_json_errors_on_no_object() {
    assert!(extract_json("no json here at all").is_err());
}

#[test]
fn apply_edits_refuses_an_ambiguous_target() {
    let dir = tempfile::tempdir().expect("tmp");
    std::fs::write(dir.path().join("mod.rs"), "x\nx\n").expect("seed");
    let edits = vec![EditOp { path: "mod.rs".into(), find: "x".into(), replace: "y".into() }];
    assert!(apply_edits(&edits, dir.path()).is_err());
}
