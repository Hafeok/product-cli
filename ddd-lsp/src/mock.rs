//! Fixture-grade mock LSP host — the hermetic stand-in CI runs against.
//!
//! Speaks real Content-Length framing over stdio, tracks open documents,
//! answers the M3 request set from naive fixture-syntax parsers
//! (`mock_parse`), and can imitate the Roslyn solution-open handshake
//! (readiness only after `solution/open`/`project/open` plus a delay).
//! This is test scaffolding for the workspace's hermetic CI
//! (`dec/ddd/fixtures-not-sdk`), not a language adapter: real-host runs
//! are the `DDD_LSP_E2E=1` gated tests.

pub mod hierarchy;
pub mod parse;

use std::collections::HashMap;
use std::io::{BufReader, Stdin, Stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

use crate::jsonrpc;
use crate::protocol::from_uri;

/// How the mock behaves at startup.
pub struct MockConfig {
    /// Demand `solution/open`/`project/open` before readiness, then delay.
    pub roslyn_handshake: bool,
    pub ready_delay: Duration,
    /// Advertise `typeHierarchyProvider: true`, then answer `null` — the
    /// over-advertising shape observed live on kotlin-lsp, kept as a
    /// fixture because it is the direction that fabricates ground.
    pub over_advertise_type_hierarchy: bool,
}

/// Serve the mock over this process's stdio until EOF or `exit`.
pub fn serve_stdio(cfg: MockConfig) {
    let stdin: Stdin = std::io::stdin();
    let out: Arc<Mutex<Stdout>> = Arc::new(Mutex::new(std::io::stdout()));
    let ready = Arc::new(AtomicBool::new(false));
    let mut docs: HashMap<String, String> = HashMap::new();
    let mut reader = BufReader::new(stdin.lock());
    while let Ok(Some(msg)) = jsonrpc::read_message(&mut reader) {
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("").to_string();
        if method == "exit" {
            break;
        }
        if let Some(id) = msg.get("id").filter(|v| !v.is_null()).cloned() {
            let result = respond(&cfg, &ready, &docs, &method, &msg);
            send(&out, &json!({"jsonrpc": "2.0", "id": id, "result": result}));
        } else {
            notify(&cfg, &ready, &out, &mut docs, &method, &msg);
        }
    }
}

fn send(out: &Arc<Mutex<Stdout>>, msg: &Value) {
    if let Ok(mut o) = out.lock() {
        let _ = jsonrpc::write_message(&mut *o, msg);
        let _ = o.flush();
    }
}

/// Handle a notification: document sync, handshake, readiness.
fn notify(
    cfg: &MockConfig,
    ready: &Arc<AtomicBool>,
    out: &Arc<Mutex<Stdout>>,
    docs: &mut HashMap<String, String>,
    method: &str,
    msg: &Value,
) {
    match method {
        "initialized" if !cfg.roslyn_handshake => ready.store(true, Ordering::SeqCst),
        "solution/open" | "project/open" => {
            let (ready, out) = (ready.clone(), out.clone());
            let delay = cfg.ready_delay;
            std::thread::spawn(move || {
                std::thread::sleep(delay);
                send(&out, &json!({"jsonrpc": "2.0", "method": "workspace/_test_restoreNeeded",
                                   "params": {}}));
                send(&out, &json!({"jsonrpc": "2.0",
                                   "method": "workspace/projectInitializationComplete",
                                   "params": {}}));
                ready.store(true, Ordering::SeqCst);
            });
        }
        "textDocument/didOpen" => {
            let uri = msg.pointer("/params/textDocument/uri").and_then(Value::as_str);
            let text = msg.pointer("/params/textDocument/text").and_then(Value::as_str);
            if let (Some(uri), Some(text)) = (uri, text) {
                docs.insert(uri.to_string(), text.to_string());
                publish_sidecar_diagnostics(out, uri);
            }
        }
        "textDocument/didChange" => {
            let uri = msg.pointer("/params/textDocument/uri").and_then(Value::as_str);
            let text = msg.pointer("/params/contentChanges/0/text").and_then(Value::as_str);
            if let (Some(uri), Some(text)) = (uri, text) {
                docs.insert(uri.to_string(), text.to_string());
            }
        }
        _ => {}
    }
}

/// Push-model diagnostics: a `<file>.diag.json` sidecar publishes on open.
fn publish_sidecar_diagnostics(out: &Arc<Mutex<Stdout>>, uri: &str) {
    if let Some(items) = sidecar_items(uri) {
        send(out, &json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                          "params": {"uri": uri, "diagnostics": items}}));
    }
}

fn sidecar_items(uri: &str) -> Option<Value> {
    let path = from_uri(uri);
    let sidecar = path.with_file_name(format!(
        "{}.diag.json",
        path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default()
    ));
    let text = std::fs::read_to_string(sidecar).ok()?;
    serde_json::from_str(&text).ok()
}

/// Answer a request; requests never hang, mirroring a live host.
fn respond(
    cfg: &MockConfig,
    ready: &Arc<AtomicBool>,
    docs: &HashMap<String, String>,
    method: &str,
    msg: &Value,
) -> Value {
    let uri = msg
        .pointer("/params/textDocument/uri")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let text = docs.get(&uri).cloned().unwrap_or_default();
    match method {
        "initialize" => json!({"capabilities": capabilities()}),
        "shutdown" => Value::Null,
        "textDocument/documentSymbol" => {
            if cfg.roslyn_handshake && !ready.load(Ordering::SeqCst) {
                json!([])
            } else {
                parse::document_symbols(&uri, &text)
            }
        }
        "workspace/symbol" => workspace_symbols(docs, msg),
        "textDocument/definition" => definition(docs, &uri, &text, msg),
        "textDocument/prepareTypeHierarchy" => {
            if cfg.over_advertise_type_hierarchy {
                Value::Null
            } else {
                hierarchy::prepare(docs, &uri, &text, msg)
            }
        }
        "typeHierarchy/supertypes" => hierarchy::supertypes(docs, msg),
        "typeHierarchy/subtypes" => hierarchy::subtypes(docs, msg),
        "textDocument/references" => references(docs, &uri, &text, msg),
        "textDocument/hover" => {
            let word = word_at(msg, &text);
            json!({"contents": {"kind": "markdown", "value": format!("mock: `{word}`")}})
        }
        "textDocument/signatureHelp" => {
            let line = line_at(msg, &text);
            json!({"signatures": [{"label": line.trim()}],
                   "activeSignature": 0, "activeParameter": 0})
        }
        "textDocument/diagnostic" => {
            json!({"kind": "full", "items": sidecar_items(&uri).unwrap_or(json!([]))})
        }
        "textDocument/rename" => rename(&uri, &text, msg),
        _ => Value::Null,
    }
}

fn position_of(msg: &Value) -> (usize, usize) {
    let line = msg.pointer("/params/position/line").and_then(Value::as_u64).unwrap_or(0);
    let ch = msg.pointer("/params/position/character").and_then(Value::as_u64).unwrap_or(0);
    (line as usize, ch as usize)
}

fn line_at(msg: &Value, text: &str) -> String {
    let (line, _) = position_of(msg);
    text.lines().nth(line).unwrap_or("").to_string()
}

/// The identifier under the request position.
fn word_at(msg: &Value, text: &str) -> String {
    let (line_no, ch) = position_of(msg);
    let line = text.lines().nth(line_no).unwrap_or("");
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let bytes: Vec<char> = line.chars().collect();
    if bytes.is_empty() {
        return String::new();
    }
    let mut start = ch.min(bytes.len().saturating_sub(1));
    while start > 0 && is_word(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = start;
    while end < bytes.len() && is_word(bytes[end]) {
        end += 1;
    }
    bytes[start..end].iter().collect()
}

/// Whole-word occurrences of the target across every open document.
fn references(docs: &HashMap<String, String>, uri: &str, text: &str, msg: &Value) -> Value {
    let word = word_at(msg, text);
    if word.is_empty() {
        return json!([]);
    }
    let mut locations = Vec::new();
    let mut scan = |doc_uri: &str, doc: &str| {
        for (ln, line) in doc.lines().enumerate() {
            for (col, _) in whole_word_hits(line, &word) {
                locations.push(json!({"uri": doc_uri, "range": {
                    "start": {"line": ln, "character": col},
                    "end": {"line": ln, "character": col + word.len()},
                }}));
            }
        }
    };
    scan(uri, text);
    for (u, doc) in docs {
        if u != uri {
            scan(u, doc);
        }
    }
    json!(locations)
}

fn whole_word_hits(line: &str, word: &str) -> Vec<(usize, usize)> {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let mut hits = Vec::new();
    let mut from = 0;
    while let Some(idx) = line[from..].find(word) {
        let at = from + idx;
        let before_ok = at == 0 || !line[..at].chars().next_back().map(is_word).unwrap_or(false);
        let after = &line[at + word.len()..];
        let after_ok = !after.chars().next().map(is_word).unwrap_or(false);
        if before_ok && after_ok {
            hits.push((at, at + word.len()));
        }
        from = at + word.len();
    }
    hits
}

/// What this mock says it can do. `typeHierarchyProvider` is advertised
/// unconditionally — in the over-advertising mode that is the whole point:
/// the claim stands, the answer does not come.
fn capabilities() -> Value {
    json!({
        "documentSymbolProvider": true,
        "workspaceSymbolProvider": true,
        "definitionProvider": true,
        "referencesProvider": true,
        "hoverProvider": true,
        "typeHierarchyProvider": true,
    })
}

/// `workspace/symbol`: whole-word declaration matches across open
/// documents, as flat `SymbolInformation`.
fn workspace_symbols(docs: &HashMap<String, String>, msg: &Value) -> Value {
    let query = msg.pointer("/params/query").and_then(Value::as_str).unwrap_or_default();
    if query.is_empty() {
        return json!([]);
    }
    let mut out = Vec::new();
    for (uri, text) in docs {
        for symbol in parse::document_symbols(uri, text).as_array().into_iter().flatten() {
            collect_matching(symbol, query, uri, &mut out);
        }
    }
    json!(out)
}

fn collect_matching(symbol: &Value, query: &str, uri: &str, out: &mut Vec<Value>) {
    let name = symbol.get("name").and_then(Value::as_str).unwrap_or_default();
    if name.contains(query) {
        out.push(json!({
            "name": name,
            "kind": symbol.get("kind").cloned().unwrap_or(json!(0)),
            "location": {"uri": uri, "range": symbol.get("range").cloned().unwrap_or(json!({}))},
            "containerName": "mock",
        }));
    }
    for child in symbol.get("children").and_then(Value::as_array).into_iter().flatten() {
        collect_matching(child, query, uri, out);
    }
}

/// `textDocument/definition`: the first declaration of the word under the
/// position, across open documents.
fn definition(docs: &HashMap<String, String>, uri: &str, text: &str, msg: &Value) -> Value {
    let word = word_at(msg, text);
    if word.is_empty() {
        return Value::Null;
    }
    let mut search: Vec<(&String, &String)> = docs.iter().collect();
    search.sort_by_key(|(u, _)| (*u != uri, (*u).clone()));
    for (doc_uri, doc) in search {
        for (ln, line) in doc.lines().enumerate() {
            let declares = line.contains(&format!("class {word}"))
                || line.contains(&format!("interface {word}"))
                || line.contains(&format!("record {word}"))
                || line.contains(&format!("struct {word}"))
                || line.contains(&format!("enum {word}"));
            if !declares {
                continue;
            }
            let col = line.find(&word).unwrap_or(0);
            return json!([{"uri": doc_uri, "range": {
                "start": {"line": ln, "character": col},
                "end": {"line": ln, "character": col + word.len()},
            }}]);
        }
    }
    Value::Null
}

/// A WorkspaceEdit renaming every whole-word occurrence in the document.
fn rename(uri: &str, text: &str, msg: &Value) -> Value {
    let word = word_at(msg, text);
    let new_name = msg.pointer("/params/newName").and_then(Value::as_str).unwrap_or("renamed");
    let mut edits = Vec::new();
    for (ln, line) in text.lines().enumerate() {
        for (col, end) in whole_word_hits(line, &word) {
            edits.push(json!({"range": {
                "start": {"line": ln, "character": col},
                "end": {"line": ln, "character": end},
            }, "newText": new_name}));
        }
    }
    json!({"changes": {uri: edits}})
}
