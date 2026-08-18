//! The declaration-level LSP subset the code extractor reads.
//!
//! Six operations, named by the G-track PRD §5.2: `documentSymbol`,
//! `workspaceSymbol`, `typeHierarchy`, `references`, `definition`, `hover`.
//! Not completion, not refactoring, not diagnostics — holding the extractor
//! to the declared surface is what keeps adapter maturity risk (§5.4) off
//! it.
//!
//! Each function here is a thin call over [`Host::request`], normalized
//! through `protocol`. The host layer is method-generic, so wiring an
//! operation is additive: nothing below this module changed to add
//! `definition` or `typeHierarchy`.

use std::path::Path;

use serde_json::{json, Value};

use crate::error::Result;
use crate::host::Host;
use crate::protocol::{
    self, hierarchy_params, normalize_hierarchy_items, normalize_locations, position_params,
    to_uri, FileRange, HierarchyItem, RawSymbol,
};

/// One operation of the declaration-level subset.
///
/// The extractor names rows by this, the capability probe reports by this,
/// and a mapping row that cannot fire says which one did not answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    DocumentSymbol,
    WorkspaceSymbol,
    Definition,
    References,
    Hover,
    TypeHierarchy,
}

impl Operation {
    /// Every operation of the subset, in probe order — `DocumentSymbol`
    /// first because the probe's anchor is derived from its result.
    pub const ALL: [Operation; 6] = [
        Operation::DocumentSymbol,
        Operation::WorkspaceSymbol,
        Operation::Definition,
        Operation::References,
        Operation::Hover,
        Operation::TypeHierarchy,
    ];

    /// The name used in reports.
    pub fn as_str(&self) -> &'static str {
        match self {
            Operation::DocumentSymbol => "documentSymbol",
            Operation::WorkspaceSymbol => "workspaceSymbol",
            Operation::Definition => "definition",
            Operation::References => "references",
            Operation::Hover => "hover",
            Operation::TypeHierarchy => "typeHierarchy",
        }
    }

    /// The `ServerCapabilities` key a host advertises this under, if it
    /// advertises it at all. Read as a claim, never as a verdict — see
    /// [`crate::capability`].
    pub fn capability_key(&self) -> &'static str {
        match self {
            Operation::DocumentSymbol => "documentSymbolProvider",
            Operation::WorkspaceSymbol => "workspaceSymbolProvider",
            Operation::Definition => "definitionProvider",
            Operation::References => "referencesProvider",
            Operation::Hover => "hoverProvider",
            Operation::TypeHierarchy => "typeHierarchyProvider",
        }
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `textDocument/documentSymbol` — the declarations in one file.
pub fn document_symbols(host: &mut Host, file: &Path) -> Result<Vec<RawSymbol>> {
    host.ensure_open(file)?;
    let result =
        host.request("textDocument/documentSymbol", json!({"textDocument": {"uri": to_uri(file)}}))?;
    Ok(protocol::flatten_symbols(&result))
}

/// `workspace/symbol` — declarations matching a query across the workspace.
pub fn workspace_symbols(host: &mut Host, query: &str) -> Result<Vec<RawSymbol>> {
    let result = host.request("workspace/symbol", json!({"query": query}))?;
    Ok(protocol::flatten_symbols(&result))
}

/// `textDocument/definition` — where the symbol under the position is
/// declared.
///
/// The result is `Location | Location[] | LocationLink[]`, every shape
/// `normalize_locations` already accepted before this operation was wired:
/// no new normaliser, as the entry probe found.
pub fn definition(host: &mut Host, file: &Path, line: u64, character: u64) -> Result<Vec<FileRange>> {
    host.ensure_open(file)?;
    let result = host.request("textDocument/definition", position_params(file, line, character))?;
    Ok(normalize_locations(&result))
}

/// `textDocument/references` — the use sites of the symbol under the
/// position; the raw reference edges the usage-inferred mapping row reads.
pub fn references(
    host: &mut Host,
    file: &Path,
    line: u64,
    character: u64,
    include_declaration: bool,
) -> Result<Vec<FileRange>> {
    host.ensure_open(file)?;
    let mut params = position_params(file, line, character);
    params["context"] = json!({"includeDeclaration": include_declaration});
    let result = host.request("textDocument/references", params)?;
    Ok(normalize_locations(&result))
}

/// `textDocument/hover` — the server-rendered declaration header, doc
/// comment included, flattened to plain text.
pub fn hover(host: &mut Host, file: &Path, line: u64, character: u64) -> Result<Option<String>> {
    host.ensure_open(file)?;
    let result = host.request("textDocument/hover", position_params(file, line, character))?;
    Ok(hover_text(&result))
}

/// `textDocument/prepareTypeHierarchy` — the anchor item the two
/// directional calls take.
pub fn prepare_type_hierarchy(
    host: &mut Host,
    file: &Path,
    line: u64,
    character: u64,
) -> Result<Vec<HierarchyItem>> {
    host.ensure_open(file)?;
    let result =
        host.request("textDocument/prepareTypeHierarchy", position_params(file, line, character))?;
    Ok(normalize_hierarchy_items(&result))
}

/// `typeHierarchy/supertypes` — what the item derives from.
pub fn supertypes(host: &mut Host, item: &HierarchyItem) -> Result<Vec<HierarchyItem>> {
    let result = host.request("typeHierarchy/supertypes", hierarchy_params(item))?;
    Ok(normalize_hierarchy_items(&result))
}

/// `typeHierarchy/subtypes` — what derives from the item.
pub fn subtypes(host: &mut Host, item: &HierarchyItem) -> Result<Vec<HierarchyItem>> {
    let result = host.request("typeHierarchy/subtypes", hierarchy_params(item))?;
    Ok(normalize_hierarchy_items(&result))
}

/// Flatten a `Hover` result's `contents` to plain text.
///
/// The field is a union three versions of the protocol deep: a
/// `MarkupContent`, a marked string, or an array of either.
fn hover_text(result: &Value) -> Option<String> {
    let contents = result.get("contents")?;
    let text = match contents {
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(marked_string)
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(_) => marked_string(contents)?,
        _ => return None,
    };
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

fn marked_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Object(_) => v.get("value").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operation_has_a_distinct_capability_key() {
        let mut keys: Vec<&str> = Operation::ALL.iter().map(|o| o.capability_key()).collect();
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(keys.len(), before);
    }

    #[test]
    fn document_symbol_probes_first_because_the_anchor_comes_from_it() {
        assert_eq!(Operation::ALL[0], Operation::DocumentSymbol);
    }

    #[test]
    fn hover_flattens_every_contents_shape() {
        let markup = json!({"contents": {"kind": "markdown", "value": "readonly record struct LocationId"}});
        assert_eq!(hover_text(&markup).as_deref(), Some("readonly record struct LocationId"));
        let plain = json!({"contents": "a plain marked string"});
        assert_eq!(hover_text(&plain).as_deref(), Some("a plain marked string"));
        let list = json!({"contents": [{"language": "csharp", "value": "class Foo"}, "Identifies a location."]});
        assert_eq!(hover_text(&list).as_deref(), Some("class Foo\nIdentifies a location."));
    }

    #[test]
    fn a_silent_hover_is_none_not_an_empty_string() {
        assert_eq!(hover_text(&Value::Null), None);
        assert_eq!(hover_text(&json!({"contents": ""})), None);
        assert_eq!(hover_text(&json!({"contents": {"kind": "markdown", "value": "  "}})), None);
    }
}
