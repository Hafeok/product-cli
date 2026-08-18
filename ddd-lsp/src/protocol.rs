//! LSP payload shaping — uris, positions, normalized result forms.
//!
//! Everything here is generic LSP: no language names, no host quirks.
//! Symbols arrive either as a `DocumentSymbol` tree or a flat
//! `SymbolInformation` list; both flatten to [`RawSymbol`] rows carrying
//! the container path, so adapters see one shape.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Value};

/// A file path as a `file://` uri (minimal percent-encoding).
pub fn to_uri(path: &Path) -> String {
    let mut out = String::from("file://");
    for b in path.to_string_lossy().bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'-' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The path behind a `file://` uri (percent-decoding the escaped bytes).
pub fn from_uri(uri: &str) -> PathBuf {
    let raw = uri.strip_prefix("file://").unwrap_or(uri);
    let mut bytes = Vec::with_capacity(raw.len());
    let mut it = raw.bytes();
    while let Some(b) = it.next() {
        if b == b'%' {
            let hi = it.next().unwrap_or(b'0');
            let lo = it.next().unwrap_or(b'0');
            let hex = [hi, lo];
            let s = std::str::from_utf8(&hex).unwrap_or("00");
            bytes.push(u8::from_str_radix(s, 16).unwrap_or(b'?'));
        } else {
            bytes.push(b);
        }
    }
    PathBuf::from(String::from_utf8_lossy(&bytes).into_owned())
}

/// `{textDocument, position}` params for position-taking requests.
pub fn position_params(file: &Path, line: u64, character: u64) -> Value {
    json!({
        "textDocument": {"uri": to_uri(file)},
        "position": {"line": line, "character": character},
    })
}

/// The LSP `SymbolKind` table (1-26) as readable names.
pub fn kind_name(kind: u64) -> &'static str {
    const NAMES: [&str; 26] = [
        "file", "module", "namespace", "package", "class", "method", "property", "field",
        "constructor", "enum", "interface", "function", "variable", "constant", "string",
        "number", "boolean", "array", "object", "key", "null", "enum-member", "struct",
        "event", "operator", "type-parameter",
    ];
    NAMES.get((kind as usize).wrapping_sub(1)).copied().unwrap_or("unknown")
}

/// One symbol row: name, LSP kind, ranges, container path.
#[derive(Debug, Clone, Serialize)]
pub struct RawSymbol {
    pub name: String,
    pub kind: u64,
    /// `Outer/Inner` path of enclosing symbol names; empty at top level.
    pub container: String,
    /// LSP kind (number) of the direct container; 0 at top level.
    pub container_kind: u64,
    pub start_line: u64,
    pub end_line: u64,
    pub sel_line: u64,
    pub sel_char: u64,
    pub detail: Option<String>,
}

/// Flatten a `textDocument/documentSymbol` result of either shape.
pub fn flatten_symbols(result: &Value) -> Vec<RawSymbol> {
    let mut out = Vec::new();
    if let Some(items) = result.as_array() {
        for item in items {
            if item.get("range").is_some() {
                flatten_document_symbol(item, "", 0, &mut out);
            } else if let Some(sym) = symbol_information_row(item) {
                out.push(sym);
            }
        }
    }
    out
}

fn flatten_document_symbol(v: &Value, container: &str, container_kind: u64, out: &mut Vec<RawSymbol>) {
    let name = v.get("name").and_then(Value::as_str).unwrap_or("?").to_string();
    let kind = v.get("kind").and_then(Value::as_u64).unwrap_or(0);
    let sym = RawSymbol {
        name: name.clone(),
        kind,
        container: container.to_string(),
        container_kind,
        start_line: v.pointer("/range/start/line").and_then(Value::as_u64).unwrap_or(0),
        end_line: v.pointer("/range/end/line").and_then(Value::as_u64).unwrap_or(0),
        sel_line: v.pointer("/selectionRange/start/line").and_then(Value::as_u64).unwrap_or(0),
        sel_char: v.pointer("/selectionRange/start/character").and_then(Value::as_u64).unwrap_or(0),
        detail: v.get("detail").and_then(Value::as_str).map(str::to_string),
    };
    out.push(sym);
    let child_container =
        if container.is_empty() { name } else { format!("{container}/{name}") };
    if let Some(children) = v.get("children").and_then(Value::as_array) {
        for child in children {
            flatten_document_symbol(child, &child_container, kind, out);
        }
    }
}

/// A flat `SymbolInformation` entry (no selectionRange, no children).
fn symbol_information_row(v: &Value) -> Option<RawSymbol> {
    let name = v.get("name").and_then(Value::as_str)?.to_string();
    let start = v.pointer("/location/range/start/line").and_then(Value::as_u64).unwrap_or(0);
    Some(RawSymbol {
        name,
        kind: v.get("kind").and_then(Value::as_u64).unwrap_or(0),
        container: v.get("containerName").and_then(Value::as_str).unwrap_or("").to_string(),
        container_kind: 0,
        start_line: start,
        end_line: v.pointer("/location/range/end/line").and_then(Value::as_u64).unwrap_or(start),
        sel_line: start,
        sel_char: v.pointer("/location/range/start/character").and_then(Value::as_u64).unwrap_or(0),
        detail: None,
    })
}

/// One normalized location row.
#[derive(Debug, Clone, Serialize)]
pub struct FileRange {
    pub file: PathBuf,
    pub line: u64,
    pub character: u64,
    pub end_line: u64,
    pub end_character: u64,
}

/// Normalize `Location`, `Location[]`, or `LocationLink[]` results.
pub fn normalize_locations(result: &Value) -> Vec<FileRange> {
    let mut out = Vec::new();
    let items: Vec<&Value> = match result {
        Value::Array(a) => a.iter().collect(),
        Value::Object(_) => vec![result],
        _ => Vec::new(),
    };
    for item in items {
        let (uri_key, range_key) = if item.get("targetUri").is_some() {
            ("targetUri", "targetRange")
        } else {
            ("uri", "range")
        };
        let Some(uri) = item.get(uri_key).and_then(Value::as_str) else { continue };
        let range = |leaf: &str| {
            item.pointer(&format!("/{range_key}/{leaf}")).and_then(Value::as_u64).unwrap_or(0)
        };
        out.push(FileRange {
            file: from_uri(uri),
            line: range("start/line"),
            character: range("start/character"),
            end_line: range("end/line"),
            end_character: range("end/character"),
        });
    }
    out
}

/// One normalized `TypeHierarchyItem`.
///
/// Type-hierarchy items are not `Location`s: they carry the symbol's own
/// identity — name, kind, detail — beside the range, which is the half the
/// `rdfs:subClassOf` mapping row needs. Normalizing them through
/// [`normalize_locations`] would keep the uri and silently drop that.
///
/// `raw` is the item exactly as the server sent it, and it is what
/// [`hierarchy_params`] hands back on the supertypes/subtypes call. Roslyn
/// answers those only when its opaque `data` blob (`ProjectGuid`,
/// `SymbolKeyData`, `TextDocument`) returns unchanged, so the round-trip is
/// verbatim by construction rather than by copying the fields we happen to
/// know about today.
#[derive(Debug, Clone, Serialize)]
pub struct HierarchyItem {
    pub name: String,
    pub kind: u64,
    pub detail: Option<String>,
    pub file: PathBuf,
    pub line: u64,
    pub character: u64,
    /// The server's opaque `data`, present only when it sent one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// The unmodified item; carried back on the next call, never rebuilt.
    #[serde(skip)]
    pub raw: Value,
}

/// Normalize a `TypeHierarchyItem[]` result (prepare, supertypes, subtypes).
///
/// A server with nothing to say answers `null`, which is an empty result
/// here — the caller decides whether that means "no hierarchy" or "the
/// instrument did not answer" (see `capability`).
pub fn normalize_hierarchy_items(result: &Value) -> Vec<HierarchyItem> {
    let Some(items) = result.as_array() else { return Vec::new() };
    items.iter().filter_map(hierarchy_item).collect()
}

fn hierarchy_item(v: &Value) -> Option<HierarchyItem> {
    let name = v.get("name").and_then(Value::as_str)?.to_string();
    let uri = v.get("uri").and_then(Value::as_str).unwrap_or_default();
    let at = |leaf: &str| v.pointer(&format!("/selectionRange/{leaf}")).and_then(Value::as_u64);
    Some(HierarchyItem {
        name,
        kind: v.get("kind").and_then(Value::as_u64).unwrap_or(0),
        detail: v.get("detail").and_then(Value::as_str).map(str::to_string),
        file: from_uri(uri),
        line: at("start/line")
            .or_else(|| v.pointer("/range/start/line").and_then(Value::as_u64))
            .unwrap_or(0),
        character: at("start/character")
            .or_else(|| v.pointer("/range/start/character").and_then(Value::as_u64))
            .unwrap_or(0),
        data: v.get("data").cloned(),
        raw: v.clone(),
    })
}

/// `{item}` params for `typeHierarchy/supertypes` / `subtypes`, carrying the
/// server's own item back untouched.
pub fn hierarchy_params(item: &HierarchyItem) -> Value {
    json!({"item": item.raw})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_round_trips_a_spaced_path() {
        let p = PathBuf::from("/tmp/a dir/File.cs");
        let uri = to_uri(&p);
        assert_eq!(uri, "file:///tmp/a%20dir/File.cs");
        assert_eq!(from_uri(&uri), p);
    }

    #[test]
    fn flattens_a_document_symbol_tree_with_container_paths() {
        let v = serde_json::json!([{
            "name": "Outer", "kind": 5,
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 9, "character": 1}},
            "selectionRange": {"start": {"line": 0, "character": 13}, "end": {"line": 0, "character": 18}},
            "children": [{
                "name": "Ping", "kind": 6,
                "range": {"start": {"line": 2, "character": 4}, "end": {"line": 4, "character": 5}},
                "selectionRange": {"start": {"line": 2, "character": 16}, "end": {"line": 2, "character": 20}}
            }]
        }]);
        let flat = flatten_symbols(&v);
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[1].container, "Outer");
        assert_eq!(flat[1].container_kind, 5);
        assert_eq!(kind_name(flat[1].kind), "method");
    }

    #[test]
    fn normalizes_location_and_link_shapes() {
        let v = serde_json::json!([
            {"uri": "file:///a.cs", "range": {"start": {"line": 1, "character": 2}, "end": {"line": 1, "character": 5}}},
            {"targetUri": "file:///b.cs", "targetRange": {"start": {"line": 3, "character": 0}, "end": {"line": 3, "character": 4}}}
        ]);
        let locs = normalize_locations(&v);
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0].file, PathBuf::from("/a.cs"));
        assert_eq!(locs[1].line, 3);
    }

    #[test]
    fn normalizes_a_type_hierarchy_item_keeping_its_identity() {
        let v = serde_json::json!([{
            "name": "ICommand", "kind": 11, "detail": "Acme.Messaging",
            "uri": "file:///src/ICommand.cs",
            "range": {"start": {"line": 4, "character": 0}, "end": {"line": 6, "character": 1}},
            "selectionRange": {"start": {"line": 4, "character": 17}, "end": {"line": 4, "character": 25}},
            "data": {"ProjectGuid": "9f1c…", "SymbolKeyData": "opaque", "TextDocument": {"uri": "file:///src/ICommand.cs"}}
        }]);
        let items = normalize_hierarchy_items(&v);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "ICommand");
        assert_eq!(kind_name(items[0].kind), "interface");
        assert_eq!(items[0].file, PathBuf::from("/src/ICommand.cs"));
        assert_eq!((items[0].line, items[0].character), (4, 17));
        assert_eq!(items[0].detail.as_deref(), Some("Acme.Messaging"));
    }

    /// The one wiring detail the entry probe named as most likely to be got
    /// wrong: supertypes/subtypes resolve only when the prepare item comes
    /// back byte-identical, opaque `data` included.
    #[test]
    fn the_opaque_data_blob_round_trips_verbatim() {
        let item = serde_json::json!({
            "name": "LocationId", "kind": 23, "uri": "file:///src/LocationId.cs",
            "range": {"start": {"line": 2, "character": 0}, "end": {"line": 2, "character": 60}},
            "selectionRange": {"start": {"line": 2, "character": 29}, "end": {"line": 2, "character": 39}},
            "data": {"ProjectGuid": "3f2a", "SymbolKeyData": "\\u0001Acme|Location", "TextDocument": {"uri": "file:///src/LocationId.cs"}},
            "tags": [1],
            "anUnknownFieldAFutureServerAdds": {"nested": [1, 2, 3]}
        });
        let items = normalize_hierarchy_items(&serde_json::json!([item.clone()]));
        let params = hierarchy_params(&items[0]);
        assert_eq!(params["item"], item, "the item must go back exactly as it arrived");
        assert_eq!(items[0].data, item.get("data").cloned());
    }

    #[test]
    fn a_silent_hierarchy_result_normalizes_to_nothing() {
        assert!(normalize_hierarchy_items(&Value::Null).is_empty());
        assert!(normalize_hierarchy_items(&serde_json::json!([])).is_empty());
    }
}
