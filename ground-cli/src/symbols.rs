//! Turning one file's LSP symbol tree into declarations.
//!
//! Pure: no host, no I/O. It also yields each declaration's **anchor** — the
//! line and column of the declaration's own name — because the position
//! operations need it and a citation does not. The anchor stays inside the
//! reader; nothing about a column belongs in a fact that will be ratified.

use ddd_lsp::protocol::RawSymbol;
use product_core::ground::facts::{DeclKind, Declaration, PropertyDecl};

/// LSP `SymbolKind`s that name a type declaration.
const TYPE_KINDS: [u64; 5] = [5, 10, 11, 23, 26];
/// LSP `SymbolKind`s that name a member carrying a type.
const MEMBER_KINDS: [u64; 3] = [7, 8, 22];

/// Where to point a position-taking request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub name: String,
    pub file: String,
    pub line: u64,
    pub character: u64,
}

/// One declaration with the anchor its own name sits at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Read {
    pub declaration: Declaration,
    pub anchor: Anchor,
    /// A generic parameter list stripped from the declared name, kept so the
    /// reader can report how many declarations were generic.
    pub generics: Option<String>,
    /// Declared inside another type rather than directly in a namespace.
    pub nested: bool,
    /// Members read as fields rather than properties. The declaration
    /// surface carries no visibility, so a private implementation field is
    /// indistinguishable from a domain property here — counted so the
    /// indistinguishability is reported rather than absorbed.
    pub field_members: usize,
}

/// Every type declaration in one file's symbol tree.
pub fn declarations_in(relative_path: &str, symbols: &[RawSymbol]) -> Vec<Read> {
    let mut out = Vec::new();
    for symbol in symbols.iter().filter(|s| TYPE_KINDS.contains(&s.kind)) {
        let Some(kind) = decl_kind(symbol.kind) else { continue };
        let (name, generics) = split_generics(&symbol.name);
        if name.is_empty() {
            continue;
        }
        out.push(Read {
            anchor: Anchor {
                name: name.clone(),
                file: relative_path.to_string(),
                line: symbol.sel_line,
                character: symbol.sel_char,
            },
            declaration: Declaration {
                name,
                kind,
                module: module_of(symbol),
                file: relative_path.to_string(),
                start_line: symbol.start_line,
                end_line: symbol.end_line,
                doc: None,
                properties: members_of(symbols, &symbol.name),
            },
            generics,
            nested: is_nested(symbol),
            field_members: field_count(symbols, &symbol.name),
        });
    }
    out
}

/// LSP `SymbolKind`s that name a namespace or module.
const NAMESPACE_KINDS: [u64; 3] = [2, 3, 4];

/// A type declared inside another type, rather than directly in a namespace.
fn is_nested(symbol: &RawSymbol) -> bool {
    symbol.container_kind != 0 && !NAMESPACE_KINDS.contains(&symbol.container_kind)
}

/// Members the server reported as fields (kind 8) rather than properties.
fn field_count(symbols: &[RawSymbol], owner: &str) -> usize {
    symbols
        .iter()
        .filter(|s| s.kind == 8)
        .filter(|s| s.container.rsplit('/').next() == Some(owner))
        .filter(|s| typed_member(&s.name).is_some())
        .count()
}

/// Members declared directly inside a type, with the types the server
/// rendered inline on the symbol name (`Id : ProfileId`).
fn members_of(symbols: &[RawSymbol], owner: &str) -> Vec<PropertyDecl> {
    symbols
        .iter()
        .filter(|s| MEMBER_KINDS.contains(&s.kind))
        .filter(|s| s.container.rsplit('/').next() == Some(owner))
        .filter_map(|s| typed_member(&s.name))
        .collect()
}

/// `Name : Type` as a typed member renders. A member the server rendered
/// without a type is skipped rather than guessed at.
pub fn typed_member(rendered: &str) -> Option<PropertyDecl> {
    let (name, type_name) = rendered.split_once(':')?;
    let (name, type_name) = (name.trim(), type_name.trim());
    (!name.is_empty() && !type_name.is_empty())
        .then(|| PropertyDecl { name: name.to_string(), type_name: type_name.to_string() })
}

/// The namespace a declaration sits in: the **outermost** segment of the
/// container path.
///
/// Taking the innermost segment reads a nested type's *enclosing type* as
/// its module, which proposed a module axis named after a class. The path is
/// outermost-first, so the namespace is its head.
fn module_of(symbol: &RawSymbol) -> String {
    symbol.container.split('/').next().unwrap_or("").to_string()
}

/// Strip a generic parameter list from a declared name.
pub fn split_generics(name: &str) -> (String, Option<String>) {
    match name.split_once('<') {
        Some((base, rest)) => {
            (base.trim().to_string(), Some(rest.trim_end_matches('>').trim().to_string()))
        }
        None => (name.trim().to_string(), None),
    }
}

fn decl_kind(lsp_kind: u64) -> Option<DeclKind> {
    match lsp_kind {
        5 => Some(DeclKind::Class),
        10 => Some(DeclKind::Enum),
        11 => Some(DeclKind::Interface),
        23 => Some(DeclKind::Struct),
        26 => Some(DeclKind::Class),
        _ => None,
    }
}

#[cfg(test)]
#[path = "symbols_tests.rs"]
mod tests;
