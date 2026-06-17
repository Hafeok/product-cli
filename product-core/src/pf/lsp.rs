//! LSP protocol — the pure JSON-RPC surface for a rust-analyzer session.
//!
//! Builders for the requests/notifications a coding worker needs (initialize
//! with clippy as the check command, didOpen, pull diagnostics, documentSymbol,
//! references, definition), parsers turning rust-analyzer's replies into plain
//! result types, and the `Content-Length` frame codec. No I/O: the transport
//! and process live in the CLI adapter, so this stays unit-testable.

use serde_json::{json, Value};

/// A position in a document (0-based line + UTF-16 character, per LSP).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// A diagnostic surfaced by rust-analyzer (rustc or, with the clippy check
/// command, clippy).
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub path: String,
    pub line: u32,
    pub character: u32,
    pub severity: String,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

/// A document symbol (function, struct, …).
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub line: u32,
}

/// A source location (used for references and definitions).
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub path: String,
    pub line: u32,
    pub character: u32,
}

/// The rust-analyzer initialization options that turn on clippy as the check
/// command, so flycheck diagnostics carry strict clippy lints.
pub fn init_options() -> Value {
    json!({
        "check": { "command": "clippy", "extraArgs": ["--", "-D", "warnings", "-D", "clippy::unwrap_used"] },
        "checkOnSave": true,
        "diagnostics": { "enable": true }
    })
}

/// The `initialize` request — declares the workspace root and our options.
pub fn initialize_request(id: i64, root_uri: &str) -> Value {
    rpc(id, "initialize", json!({
        "processId": null,
        "rootUri": root_uri,
        "capabilities": { "textDocument": { "publishDiagnostics": {}, "documentSymbol": {}, "references": {}, "definition": {} } },
        "initializationOptions": init_options(),
    }))
}

/// The `initialized` notification that completes the handshake.
pub fn initialized_notification() -> Value {
    notify("initialized", json!({}))
}

/// Open a document so rust-analyzer analyses it.
pub fn did_open(uri: &str, text: &str) -> Value {
    notify("textDocument/didOpen", json!({
        "textDocument": { "uri": uri, "languageId": "rust", "version": 1, "text": text }
    }))
}

/// Notify a full-content change (re-analysis after the worker edits the file).
pub fn did_change(uri: &str, version: i64, text: &str) -> Value {
    notify("textDocument/didChange", json!({
        "textDocument": { "uri": uri, "version": version },
        "contentChanges": [{ "text": text }],
    }))
}

/// Close a document.
pub fn did_close(uri: &str) -> Value {
    notify("textDocument/didClose", json!({ "textDocument": { "uri": uri } }))
}

/// Notify a save (triggers flycheck → cargo clippy → publishDiagnostics).
pub fn did_save(uri: &str) -> Value {
    notify("textDocument/didSave", json!({ "textDocument": { "uri": uri } }))
}

/// Pull diagnostics for a document (rust-analyzer's own analysis).
pub fn diagnostic_request(id: i64, uri: &str) -> Value {
    rpc(id, "textDocument/diagnostic", json!({ "textDocument": { "uri": uri } }))
}

/// Request the document's symbols.
pub fn document_symbol_request(id: i64, uri: &str) -> Value {
    rpc(id, "textDocument/documentSymbol", json!({ "textDocument": { "uri": uri } }))
}

/// Request the references to the symbol at `pos`.
pub fn references_request(id: i64, uri: &str, pos: Position) -> Value {
    rpc(id, "textDocument/references", json!({
        "textDocument": { "uri": uri },
        "position": { "line": pos.line, "character": pos.character },
        "context": { "includeDeclaration": true },
    }))
}

/// Request the definition of the symbol at `pos`.
pub fn definition_request(id: i64, uri: &str, pos: Position) -> Value {
    rpc(id, "textDocument/definition", json!({
        "textDocument": { "uri": uri },
        "position": { "line": pos.line, "character": pos.character },
    }))
}

/// The `shutdown` request.
pub fn shutdown_request(id: i64) -> Value {
    rpc(id, "shutdown", Value::Null)
}

/// The `exit` notification.
pub fn exit_notification() -> Value {
    notify("exit", Value::Null)
}

fn rpc(id: i64, method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

fn notify(method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "method": method, "params": params })
}

/// Frame a message with its `Content-Length` header for the LSP wire.
pub fn encode(value: &Value) -> String {
    let body = value.to_string();
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
}

/// Parse a `Content-Length: N` header line, ignoring case and whitespace.
pub fn content_length(header: &str) -> Option<usize> {
    let (name, val) = header.split_once(':')?;
    if !name.trim().eq_ignore_ascii_case("content-length") {
        return None;
    }
    val.trim().parse().ok()
}

/// A `file://` URI for an absolute path (minimal — assumes a POSIX path).
pub fn path_to_uri(abs: &str) -> String {
    format!("file://{abs}")
}

/// The path from a `file://` URI.
pub fn uri_to_path(uri: &str) -> String {
    uri.strip_prefix("file://").unwrap_or(uri).to_string()
}

/// Map an LSP severity number to a label.
pub fn severity_label(n: u64) -> &'static str {
    match n {
        1 => "error",
        2 => "warning",
        3 => "information",
        _ => "hint",
    }
}

/// Parse `publishDiagnostics` params (`{ uri, diagnostics: [...] }`).
pub fn parse_publish_diagnostics(params: &Value) -> Vec<Diagnostic> {
    let path = params.get("uri").and_then(Value::as_str).map(uri_to_path).unwrap_or_default();
    let items = params.get("diagnostics").and_then(Value::as_array).cloned().unwrap_or_default();
    items.iter().map(|d| one_diagnostic(&path, d)).collect()
}

/// Parse a pull-diagnostic `result` (`{ kind, items: [...] }`) for a known path.
pub fn parse_diagnostics(path: &str, result: &Value) -> Vec<Diagnostic> {
    let items = result.get("items").and_then(Value::as_array).cloned().unwrap_or_default();
    items.iter().map(|d| one_diagnostic(path, d)).collect()
}

fn one_diagnostic(path: &str, d: &Value) -> Diagnostic {
    let start = d.get("range").and_then(|r| r.get("start"));
    Diagnostic {
        path: path.to_string(),
        line: start.and_then(|s| s.get("line")).and_then(Value::as_u64).unwrap_or(0) as u32,
        character: start.and_then(|s| s.get("character")).and_then(Value::as_u64).unwrap_or(0) as u32,
        severity: severity_label(d.get("severity").and_then(Value::as_u64).unwrap_or(1)).to_string(),
        message: d.get("message").and_then(Value::as_str).unwrap_or("").to_string(),
        source: d.get("source").and_then(Value::as_str).map(str::to_string),
        code: code_of(d),
    }
}

fn code_of(d: &Value) -> Option<String> {
    match d.get("code") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    }
}

/// Parse a `documentSymbol` result (DocumentSymbol[] or SymbolInformation[]).
pub fn parse_symbols(result: &Value) -> Vec<Symbol> {
    let arr = result.as_array().cloned().unwrap_or_default();
    arr.iter().map(one_symbol).collect()
}

fn one_symbol(s: &Value) -> Symbol {
    // DocumentSymbol nests `range`; SymbolInformation nests `location.range`.
    let line = s
        .get("range")
        .or_else(|| s.get("location").and_then(|l| l.get("range")))
        .and_then(|r| r.get("start"))
        .and_then(|p| p.get("line"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32;
    Symbol {
        name: s.get("name").and_then(Value::as_str).unwrap_or("").to_string(),
        kind: symbol_kind(s.get("kind").and_then(Value::as_u64).unwrap_or(0)).to_string(),
        line,
    }
}

/// Parse a `references`/`definition` result (Location | Location[]).
pub fn parse_locations(result: &Value) -> Vec<Location> {
    let arr = match result {
        Value::Array(a) => a.clone(),
        Value::Object(_) => vec![result.clone()],
        _ => Vec::new(),
    };
    arr.iter().filter_map(one_location).collect()
}

fn one_location(l: &Value) -> Option<Location> {
    let uri = l.get("uri").or_else(|| l.get("targetUri")).and_then(Value::as_str)?;
    let start = l.get("range").or_else(|| l.get("targetRange")).and_then(|r| r.get("start"));
    Some(Location {
        path: uri_to_path(uri),
        line: start.and_then(|s| s.get("line")).and_then(Value::as_u64).unwrap_or(0) as u32,
        character: start.and_then(|s| s.get("character")).and_then(Value::as_u64).unwrap_or(0) as u32,
    })
}

fn symbol_kind(n: u64) -> &'static str {
    match n {
        5 => "class",
        6 => "method",
        11 => "interface",
        12 => "function",
        13 => "variable",
        14 => "constant",
        23 => "struct",
        10 => "enum",
        _ => "symbol",
    }
}

#[cfg(test)]
#[path = "lsp_tests.rs"]
mod tests;
