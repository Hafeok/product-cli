//! Rust symbol facts — visibility grading, signature slicing, impl shapes.
//!
//! rust-analyzer's `documentSymbol` carries no visibility at all, so the
//! grade (`pub` / `pub(crate)` / `pub(super)` / `pub(in path)` / private) is
//! read off the declaration text here. Impl blocks arrive as one symbol
//! named `impl Trait for Type`, which is where trait participation is read
//! from; they carry no visibility of their own, so they are judged at the
//! visibility of the type they are written for.

use std::collections::BTreeMap;

use crate::adapter::rust::DECL_KINDS;
use crate::adapter::AdapterFlags;
use crate::protocol::{kind_name, RawSymbol};
use crate::surface::SymbolFacts;

/// Derive per-symbol facts. Two passes: type visibilities first, because an
/// impl block can be written above the type it is for.
pub fn facts(text: &str, symbols: &[RawSymbol], flags: &AdapterFlags) -> Vec<SymbolFacts> {
    let lines: Vec<&str> = text.lines().collect();
    let types = type_visibilities(&lines, symbols);
    let mut effective: BTreeMap<String, String> = BTreeMap::new();
    let mut out = Vec::new();
    for sym in symbols {
        let kind = normalize_kind(sym, &types);
        if !DECL_KINDS.contains(&kind.as_str()) {
            continue;
        }
        let head = decl_start(&lines, sym.start_line as usize);
        let decl = declaration_slice(&lines, head);
        let container_vis =
            effective.get(&sym.container).cloned().unwrap_or_else(|| "public".to_string());
        let vis = visibility_of(sym, &kind, &decl, &container_vis, &types);
        let path = if sym.container.is_empty() {
            sym.name.clone()
        } else {
            format!("{}/{}", sym.container, sym.name)
        };
        effective.insert(path, vis.clone());
        let decorators = attribute_lines(&lines, sym.start_line as usize, head);
        let exported = decorators
            .iter()
            .any(|d| flags.exported_attributes.iter().any(|a| d.contains(a.as_str())));
        out.push(SymbolFacts {
            name: sym.name.clone(),
            container: sym.container.clone(),
            extra: extra_for(&kind, &sym.name),
            kind,
            visibility: vis,
            signature: strip_visibility(&decl),
            decorators,
            exported,
            sel_line: sym.sel_line,
            sel_char: sym.sel_char,
        });
    }
    out
}

/// Declaration-site visibility of every named type in the file, so an impl
/// block can be judged at the reachability of what it is written for.
fn type_visibilities(lines: &[&str], symbols: &[RawSymbol]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for sym in symbols {
        if is_impl_symbol(&sym.name) {
            continue;
        }
        let kind = kind_name(sym.kind);
        if !matches!(kind, "struct" | "enum" | "interface" | "class" | "type-parameter" | "module")
        {
            continue;
        }
        let decl = declaration_slice(lines, decl_start(lines, sym.start_line as usize));
        map.insert(
            base_name(&sym.name).to_string(),
            declared_visibility(&decl).unwrap_or_else(|| "private".to_string()),
        );
    }
    map
}

/// The visibility a symbol is judged at, capped by what encloses it.
fn visibility_of(
    sym: &RawSymbol,
    kind: &str,
    decl: &str,
    container_vis: &str,
    types: &BTreeMap<String, String>,
) -> String {
    if kind.ends_with("impl") || kind == "trait-impl-unresolved" {
        // An impl block has no visibility keyword; it is exactly as
        // reachable as the type it is written for. An unknown type is
        // assumed reachable, so the demand is raised rather than dropped.
        let (_, self_type) = parse_impl(&sym.name);
        return types.get(&self_type).cloned().unwrap_or_else(|| "public".to_string());
    }
    let own = declared_visibility(decl).unwrap_or_else(|| {
        if inherits_container(sym) {
            container_vis.to_string()
        } else {
            "private".to_string()
        }
    });
    cap_visibility(&own, container_vis)
}

/// Trait members and enum variants take no visibility keyword — they are as
/// visible as what declares them.
fn inherits_container(sym: &RawSymbol) -> bool {
    matches!(sym.container_kind, 10 | 11)
}

/// Effective visibility is capped by the container's: a `pub fn` in a
/// private module is not reachable.
fn cap_visibility(own: &str, container: &str) -> String {
    let rank = crate::adapter::rust::ADAPTER.visibility_rank;
    if rank(own) <= rank(container) {
        own.to_string()
    } else {
        container.to_string()
    }
}

/// LSP kind plus the impl-name shape → the table's vocabulary.
fn normalize_kind(sym: &RawSymbol, types: &BTreeMap<String, String>) -> String {
    if is_impl_symbol(&sym.name) {
        return impl_kind(&sym.name, types);
    }
    if sym.container_kind == 11 {
        return "trait-member".to_string();
    }
    match kind_name(sym.kind) {
        "interface" => "trait".to_string(),
        "method" | "function" => "fn".to_string(),
        "class" => "struct".to_string(),
        "namespace" | "package" => "module".to_string(),
        "constant" => "const".to_string(),
        "variable" => "static".to_string(),
        "type-parameter" => "type-alias".to_string(),
        other => other.to_string(),
    }
}

/// `impl Type` is inherent; `impl Trait for Type` is participation. When
/// neither side is declared in this file, coherence is unanswerable here —
/// the orphan-impl shape — so it gets its own kind rather than being
/// silently treated as an ordinary trait impl.
fn impl_kind(name: &str, types: &BTreeMap<String, String>) -> String {
    let (trait_name, self_type) = parse_impl(name);
    let Some(trait_name) = trait_name else {
        return "inherent-impl".to_string();
    };
    if types.contains_key(&trait_name) || types.contains_key(&self_type) {
        "trait-impl".to_string()
    } else {
        "trait-impl-unresolved".to_string()
    }
}

/// An impl block's symbol name: `impl Type`, or `impl<T> Type<T>` with the
/// generic list written tight against the keyword.
fn is_impl_symbol(name: &str) -> bool {
    name.strip_prefix("impl").map(|rest| rest.starts_with(['<', ' '])).unwrap_or(false)
}

/// Split `impl<G> Trait<A> for Type<B>` into its trait plus self type, each
/// reduced to the bare path segment.
fn parse_impl(name: &str) -> (Option<String>, String) {
    let rest = name.strip_prefix("impl").unwrap_or(name).trim_start();
    let rest = skip_generics(rest);
    match rest.split_once(" for ") {
        Some((t, s)) => (Some(base_name(t).to_string()), base_name(s).to_string()),
        None => (None, base_name(rest).to_string()),
    }
}

/// Drop a leading `<...>` generic parameter list.
fn skip_generics(s: &str) -> &str {
    if !s.starts_with('<') {
        return s;
    }
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return s[i + 1..].trim_start();
                }
            }
            _ => {}
        }
    }
    s
}

/// The bare path segment of a type expression: `foo::Bar<T>` → `Bar`.
fn base_name(s: &str) -> &str {
    let s = s.trim().trim_start_matches('&').trim();
    let s = s.split(['<', ' ', '(']).next().unwrap_or(s);
    s.rsplit("::").next().unwrap_or(s)
}

/// Impl blocks carry their two halves onto the seam row, so a declaration
/// can name what participates in what.
fn extra_for(kind: &str, name: &str) -> BTreeMap<String, String> {
    let mut extra = BTreeMap::new();
    if !kind.ends_with("impl") && kind != "trait-impl-unresolved" {
        return extra;
    }
    let (trait_name, self_type) = parse_impl(name);
    if let Some(t) = trait_name {
        extra.insert("trait".to_string(), t);
    }
    extra.insert("self_type".to_string(), self_type);
    if kind == "trait-impl-unresolved" {
        extra.insert(
            "coherence".to_string(),
            "unresolved-in-file: neither the trait nor the self type is declared here".to_string(),
        );
    }
    extra
}

/// The first line of the declaration proper, skipping attribute lines that
/// rust-analyzer includes in an item's range.
fn decl_start(lines: &[&str], start: usize) -> usize {
    let mut i = start;
    while i < lines.len() {
        let t = lines[i].trim();
        if t.starts_with("#[") || t.starts_with("#![") || t.starts_with("///") || t.starts_with("//!")
        {
            i += 1;
        } else {
            return i;
        }
    }
    start
}

/// The declaration text from its first line to the body or terminator,
/// whitespace-collapsed. Bodies (`{`), terminators (`;`), and initialisers
/// (`=`) are cut at depth zero only, so generic bounds, default type
/// parameters, and `where` clauses survive intact.
fn declaration_slice(lines: &[&str], start: usize) -> String {
    let mut decl = String::new();
    let (mut paren, mut angle, mut bracket) = (0i32, 0i32, 0i32);
    for line in lines.iter().skip(start).take(8) {
        let mut piece: &str = line;
        let mut done = false;
        let mut prev = '\0';
        for (i, ch) in line.char_indices() {
            let flat = paren == 0 && angle <= 0 && bracket == 0;
            match ch {
                '(' => paren += 1,
                ')' => paren -= 1,
                '[' => bracket += 1,
                ']' => bracket -= 1,
                '<' => angle += 1,
                // `->` is not a closing bracket.
                '>' if prev != '-' => angle -= 1,
                '{' | ';' | '=' if flat => {
                    piece = &line[..i];
                    done = true;
                }
                _ => {}
            }
            prev = ch;
            if done {
                break;
            }
        }
        decl.push_str(piece);
        decl.push(' ');
        if done {
            break;
        }
    }
    decl.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The declared visibility grade, or `None` when no keyword is written.
fn declared_visibility(decl: &str) -> Option<String> {
    let rest = decl.trim_start().strip_prefix("pub")?;
    match rest.chars().next() {
        None => Some("public".to_string()),
        Some(c) if c.is_whitespace() => Some("public".to_string()),
        Some('(') => Some(restricted_grade(rest)),
        // `pub` was the start of some other identifier.
        _ => None,
    }
}

/// `pub(crate)` / `pub(super)` / `pub(self)` / `pub(in path)`.
fn restricted_grade(rest: &str) -> String {
    let inner = rest
        .trim_start_matches('(')
        .split(')')
        .next()
        .unwrap_or("")
        .trim();
    match inner {
        "crate" => "pub(crate)".to_string(),
        "super" => "pub(super)".to_string(),
        // `pub(self)` is private by another name.
        "self" => "private".to_string(),
        _ => "pub(restricted)".to_string(),
    }
}

/// The signature must not carry the visibility grade — a grade change is
/// its own change kind, judged at the more exposed side.
fn strip_visibility(decl: &str) -> String {
    let d = decl.trim_start();
    let rest = match d.strip_prefix("pub") {
        Some(r) => {
            let r = r.trim_start();
            match r.strip_prefix('(') {
                Some(after) => after.find(')').map(|i| &after[i + 1..]).unwrap_or(r),
                None => r,
            }
        }
        None => d,
    };
    rest.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Attributes attached to the declaration, whether the host's range starts
/// above them or at them.
fn attribute_lines(lines: &[&str], range_start: usize, head: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut i = range_start;
    while i > 0 {
        let prev = lines[i - 1].trim();
        if prev.starts_with("#[") && prev.ends_with(']') {
            out.push(prev.to_string());
            i -= 1;
        } else {
            break;
        }
    }
    out.reverse();
    for line in lines.iter().take(head).skip(range_start) {
        let t = line.trim();
        if t.starts_with("#[") {
            out.push(t.to_string());
        }
    }
    out
}
