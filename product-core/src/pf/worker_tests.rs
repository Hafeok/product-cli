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
