//! Tests for the LSP protocol surface (builders, frame codec, parsers).

use super::*;
use serde_json::json;

#[test]
fn encode_frames_with_content_length() {
    let msg = encode(&json!({ "a": 1 }));
    assert!(msg.starts_with("Content-Length: "));
    assert!(msg.contains("\r\n\r\n"));
    let body = msg.split("\r\n\r\n").nth(1).expect("body");
    let len: usize = msg.lines().next().and_then(content_length).expect("len");
    assert_eq!(len, body.len());
}

#[test]
fn content_length_is_case_insensitive() {
    assert_eq!(content_length("content-length: 42"), Some(42));
    assert_eq!(content_length("Content-Length:  7 "), Some(7));
    assert_eq!(content_length("Content-Type: x"), None);
}

#[test]
fn init_options_select_clippy() {
    let opts = init_options();
    assert_eq!(opts["check"]["command"], json!("clippy"));
    assert_eq!(opts["checkOnSave"], json!(true));
}

#[test]
fn initialize_request_carries_root_and_options() {
    let req = initialize_request(1, "file:///work");
    assert_eq!(req["method"], json!("initialize"));
    assert_eq!(req["id"], json!(1));
    assert_eq!(req["params"]["rootUri"], json!("file:///work"));
    assert_eq!(req["params"]["initializationOptions"]["check"]["command"], json!("clippy"));
}

#[test]
fn notifications_have_no_id() {
    let n = initialized_notification();
    assert!(n.get("id").is_none());
    assert_eq!(n["method"], json!("initialized"));
}

#[test]
fn uri_round_trips() {
    let uri = path_to_uri("/home/x/src/a.rs");
    assert_eq!(uri, "file:///home/x/src/a.rs");
    assert_eq!(uri_to_path(&uri), "/home/x/src/a.rs");
}

#[test]
fn parse_publish_diagnostics_maps_severity_and_span() {
    let params = json!({
        "uri": "file:///work/src/a.rs",
        "diagnostics": [{
            "range": { "start": { "line": 6, "character": 4 }, "end": { "line": 6, "character": 9 } },
            "severity": 2,
            "message": "this loop could be written as a `for` loop",
            "source": "clippy",
            "code": "clippy::while_let_on_iterator"
        }]
    });
    let ds = parse_publish_diagnostics(&params);
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].path, "/work/src/a.rs");
    assert_eq!(ds[0].line, 6);
    assert_eq!(ds[0].severity, "warning");
    assert_eq!(ds[0].source.as_deref(), Some("clippy"));
    assert_eq!(ds[0].code.as_deref(), Some("clippy::while_let_on_iterator"));
}

#[test]
fn parse_symbols_reads_document_symbols() {
    let result = json!([
        { "name": "to_snake_case", "kind": 12, "range": { "start": { "line": 2, "character": 0 } } },
        { "name": "Casing", "kind": 23, "location": { "range": { "start": { "line": 10, "character": 0 } } } }
    ]);
    let syms = parse_symbols(&result);
    assert_eq!(syms[0], Symbol { name: "to_snake_case".into(), kind: "function".into(), line: 2 });
    assert_eq!(syms[1], Symbol { name: "Casing".into(), kind: "struct".into(), line: 10 });
}

#[test]
fn parse_locations_handles_single_and_array() {
    let single = json!({ "uri": "file:///a.rs", "range": { "start": { "line": 3, "character": 1 } } });
    assert_eq!(parse_locations(&single), vec![Location { path: "/a.rs".into(), line: 3, character: 1 }]);
    let arr = json!([{ "targetUri": "file:///b.rs", "targetRange": { "start": { "line": 0, "character": 0 } } }]);
    assert_eq!(parse_locations(&arr), vec![Location { path: "/b.rs".into(), line: 0, character: 0 }]);
}
