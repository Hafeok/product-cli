//! Fixture-grade type-hierarchy answers for the mock host.
//!
//! Reads the base list off a fixture declaration — `class Foo : Bar, IBaz`,
//! `record X(…) : ICommand` — across every open document, so the wiring of
//! `prepareTypeHierarchy` / `supertypes` / `subtypes` is exercised in the
//! workspace's hermetic CI, which has no .NET SDK
//! (`dec/ddd/fixtures-not-sdk`).
//!
//! The items carry an opaque `data` blob on purpose. Roslyn resolves the
//! directional calls only when its own blob returns unchanged, so the mock
//! **refuses an item whose `data` was rebuilt rather than carried** — the
//! failure a live host would show as an empty result, made loud here.

use std::collections::HashMap;

use serde_json::{json, Value};

/// One fixture declaration: where it is, what it derives from.
struct Decl {
    name: String,
    kind: u64,
    uri: String,
    line: usize,
    character: usize,
    bases: Vec<String>,
}

/// The `TypeHierarchyItem` for a declaration, `data` included.
fn item(decl: &Decl) -> Value {
    json!({
        "name": decl.name,
        "kind": decl.kind,
        "uri": decl.uri,
        "range": {"start": {"line": decl.line, "character": 0},
                   "end": {"line": decl.line, "character": 200}},
        "selectionRange": {"start": {"line": decl.line, "character": decl.character},
                            "end": {"line": decl.line, "character": decl.character + decl.name.len()}},
        // Stands in for Roslyn's {ProjectGuid, SymbolKeyData, TextDocument}:
        // opaque to the client, load-bearing on the way back. Keyed by
        // position as well as name, so two declarations sharing a name are
        // two symbols here as they are on a real host.
        "data": {"MockSymbolKey": format!("{}#{}#{}", decl.uri, decl.line, decl.name)},
    })
}

/// `textDocument/prepareTypeHierarchy` — the declaration at the position.
pub(crate) fn prepare(docs: &HashMap<String, String>, uri: &str, text: &str, msg: &Value) -> Value {
    let line = msg.pointer("/params/position/line").and_then(Value::as_u64).unwrap_or(0) as usize;
    let mut here = declarations(uri, text);
    here.extend(others(docs, uri));
    match here.iter().find(|d| d.line == line && d.uri == uri) {
        Some(decl) => json!([item(decl)]),
        None => Value::Null,
    }
}

/// `typeHierarchy/supertypes` — the bases the item's declaration names.
pub(crate) fn supertypes(docs: &HashMap<String, String>, msg: &Value) -> Value {
    directional(docs, msg)
        .map(|(all, found)| {
            let bases = &found.bases;
            let items: Vec<Value> =
                all.iter().filter(|d| bases.contains(&d.name)).map(item).collect();
            json!(items)
        })
        .unwrap_or(Value::Null)
}

/// `typeHierarchy/subtypes` — every declaration naming the item as a base.
pub(crate) fn subtypes(docs: &HashMap<String, String>, msg: &Value) -> Value {
    directional(docs, msg)
        .map(|(all, found)| {
            let items: Vec<Value> =
                all.iter().filter(|d| d.bases.contains(&found.name)).map(item).collect();
            json!(items)
        })
        .unwrap_or(Value::Null)
}

/// Resolve the inbound item back to a declaration — **through its `data`
/// blob**, so an item that was rebuilt instead of carried resolves to
/// nothing, exactly as a live host would answer it.
fn directional(
    docs: &HashMap<String, String>,
    msg: &Value,
) -> Option<(Vec<Decl>, Decl)> {
    let key = msg.pointer("/params/item/data/MockSymbolKey").and_then(Value::as_str)?;
    let (head, name) = key.rsplit_once('#')?;
    let (uri, line) = head.rsplit_once('#')?;
    let line: usize = line.parse().ok()?;
    let all: Vec<Decl> = docs.iter().flat_map(|(u, t)| declarations(u, t)).collect();
    let found = all
        .iter()
        .find(|d| d.uri == uri && d.line == line && d.name == name)
        .map(|d| Decl {
            name: d.name.clone(),
            kind: d.kind,
            uri: d.uri.clone(),
            line: d.line,
            character: d.character,
            bases: d.bases.clone(),
        })?;
    Some((all, found))
}

fn others(docs: &HashMap<String, String>, skip: &str) -> Vec<Decl> {
    docs.iter().filter(|(u, _)| *u != skip).flat_map(|(u, t)| declarations(u, t)).collect()
}

const TYPE_KEYWORDS: &[(&str, u64)] =
    &[("class", 5), ("interface", 11), ("struct", 23), ("enum", 10), ("record", 5)];

/// Every fixture type declaration in one document, with its base list.
fn declarations(uri: &str, text: &str) -> Vec<Decl> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        let Some((kind, name)) = type_head(trimmed) else { continue };
        out.push(Decl {
            character: line.find(&name).unwrap_or(0),
            bases: base_list(trimmed, &name),
            name,
            kind,
            uri: uri.to_string(),
            line: i,
        });
    }
    out
}

/// `[modifiers] class|interface|struct|enum|record Name …`
///
/// The **last** keyword of a run wins, so `record struct LocationId` is a
/// struct named `LocationId` rather than a record named `struct` — the
/// value-object identity idiom this corpus is full of.
fn type_head(trimmed: &str) -> Option<(u64, String)> {
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let keyword = |t: &&str| TYPE_KEYWORDS.iter().find(|(k, _)| k == t).map(|(_, kind)| *kind);
    let last = tokens.iter().enumerate().rfind(|(_, t)| keyword(t).is_some())?;
    let (i, kind) = (last.0, keyword(last.1)?);
    let raw = tokens.get(i + 1)?;
    let name: String = raw.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    (!name.is_empty()).then_some((kind, name))
}

/// The names after the `:` of a declaration head, positional record
/// parameter list skipped.
fn base_list(trimmed: &str, name: &str) -> Vec<String> {
    let after_name = match trimmed.split_once(name) {
        Some((_, rest)) => rest,
        None => return Vec::new(),
    };
    let rest = match after_name.split_once(')') {
        Some((head, tail)) if head.contains('(') => tail,
        _ => after_name,
    };
    let Some((_, bases)) = rest.split_once(':') else { return Vec::new() };
    bases
        .split('{')
        .next()
        .unwrap_or("")
        .split(',')
        .map(|b| b.trim().chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect())
        .filter(|b: &String| !b.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_base_list_off_a_declaration_head() {
        assert_eq!(base_list("public sealed class Foo : Bar, IBaz {", "Foo"), ["Bar", "IBaz"]);
        assert_eq!(base_list("public record Upsert(string A, int B) : ICommand;", "Upsert"), ["ICommand"]);
        assert_eq!(base_list("public readonly record struct LocationId(string Value);", "LocationId"), Vec::<String>::new());
        assert_eq!(base_list("internal class Handler : IHandler<Cmd>", "Handler"), ["IHandler"]);
    }

    #[test]
    fn a_record_struct_is_named_by_its_last_keyword() {
        assert_eq!(
            type_head("public readonly record struct LocationId(string Value);"),
            Some((23, "LocationId".to_string()))
        );
        assert_eq!(type_head("public record Upsert(int A) : ICommand;"), Some((5, "Upsert".to_string())));
    }

    #[test]
    fn an_item_whose_data_was_rebuilt_resolves_to_nothing() {
        let mut docs = HashMap::new();
        docs.insert("file:///a.cs".to_string(), "public class Foo : Bar {}\npublic class Bar {}\n".to_string());
        let carried = json!({"params": {"item": {"name": "Foo", "data": {"MockSymbolKey": "file:///a.cs#0#Foo"}}}});
        assert_eq!(supertypes(&docs, &carried).as_array().map(Vec::len), Some(1));
        let rebuilt = json!({"params": {"item": {"name": "Foo", "uri": "file:///a.cs"}}});
        assert_eq!(supertypes(&docs, &rebuilt), Value::Null);
    }
}
