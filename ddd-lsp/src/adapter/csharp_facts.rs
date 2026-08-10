//! C# facts extraction — LSP symbols plus declaration-text slicing.
//!
//! The Roslyn host supplies symbol names, kinds, ranges; everything else —
//! normalized kind, effective (container-capped) visibility, body-free
//! signature, attached attributes — is sliced from the source text here.
//! No Roslyn API anywhere.

use crate::adapter::AdapterFlags;
use crate::protocol::{kind_name, RawSymbol};
use crate::surface::SymbolFacts;

use super::csharp::{visibility_rank, DECL_KINDS};

/// Derive per-symbol facts: normalized kind, effective visibility
/// (container-capped), body-free signature, attached attributes.
pub fn facts(text: &str, symbols: &[RawSymbol], flags: &AdapterFlags) -> Vec<SymbolFacts> {
    let lines: Vec<&str> = text.lines().collect();
    let mut effective: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    let mut out = Vec::new();
    for sym in symbols {
        let kind = normalize_kind(sym);
        if !DECL_KINDS.contains(&kind.as_str()) {
            continue;
        }
        let decl = declaration_of(&lines, &kind, sym);
        let own_vis = declared_visibility(&decl, sym);
        let container_vis =
            effective.get(&sym.container).cloned().unwrap_or_else(|| "public".to_string());
        let vis = cap_visibility(&own_vis, &container_vis);
        let path = if sym.container.is_empty() {
            sym.name.clone()
        } else {
            format!("{}/{}", sym.container, sym.name)
        };
        effective.insert(path, vis.clone());
        let decorators = attribute_lines(&lines, sym.start_line as usize);
        let exported = decorators
            .iter()
            .any(|d| flags.exported_attributes.iter().any(|a| d.contains(a.as_str())));
        out.push(SymbolFacts {
            name: bare_name(&sym.name),
            container: bare_container(&sym.container),
            kind,
            visibility: vis,
            signature: strip_visibility(&decl),
            decorators,
            exported,
            extra: Default::default(),
            sel_line: sym.sel_line,
            sel_char: sym.sel_char,
        });
    }
    out
}

/// An enum member is one line with no terminator token, so the generic
/// slice would run on into the following declarations; everything else
/// gets the multi-line slice.
fn declaration_of(lines: &[&str], kind: &str, sym: &RawSymbol) -> String {
    if kind == "enum-member" {
        lines
            .get(sym.start_line as usize)
            .map(|l| l.trim().trim_end_matches(',').to_string())
            .unwrap_or_default()
    } else {
        declaration_slice(lines, sym.start_line as usize)
    }
}

/// Roslyn decorates symbol names (`Ping() : void`, `PublicApi(int)`);
/// facts carry the bare identifier so demands and declarations match on
/// the name a human would write.
fn bare_name(raw: &str) -> String {
    raw.split(['(', ' ', '<'])
        .next()
        .unwrap_or(raw)
        .to_string()
}

fn bare_container(raw: &str) -> String {
    raw.split('/').map(bare_name).collect::<Vec<_>>().join("/")
}

/// LSP kind → the table vocabulary; members of interfaces get their own
/// kind so the implementor-forcing row can name them.
fn normalize_kind(sym: &RawSymbol) -> String {
    if sym.container_kind == 11 {
        return "interface-member".to_string();
    }
    match kind_name(sym.kind) {
        "function" => "method".to_string(),
        other => other.to_string(),
    }
}

/// The declaration text: from its first line to the body/terminator,
/// whitespace-collapsed. Bodies (`{`, `=>`), field initializers, and
/// property accessors are cut; default parameter values inside parens are
/// kept — they are part of the signature.
fn declaration_slice(lines: &[&str], start: usize) -> String {
    let mut decl = String::new();
    for line in lines.iter().skip(start).take(8) {
        let mut piece = *line;
        let mut done = false;
        let mut depth = 0i32;
        for (i, ch) in line.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                '{' | ';' => {
                    piece = &line[..i];
                    done = true;
                }
                '=' if depth == 0 => {
                    piece = &line[..i];
                    done = true;
                }
                _ => {}
            }
            if done {
                break;
            }
        }
        decl.push_str(piece);
        decl.push(' ');
        if done || piece.contains(')') {
            break;
        }
    }
    decl.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The signature must not carry visibility keywords — a visibility flip
/// is its own change kind, judged at the more exposed side, never a
/// signature change.
fn strip_visibility(decl: &str) -> String {
    decl.split_whitespace()
        .filter(|t| !matches!(*t, "public" | "private" | "protected" | "internal"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The declared visibility keyword; defaults follow the language: types
/// default internal, members private, interface members public.
fn declared_visibility(decl: &str, sym: &RawSymbol) -> String {
    let tokens: Vec<&str> = decl.split_whitespace().collect();
    let mut public = false;
    let mut protected = false;
    let mut internal = false;
    let mut private = false;
    for t in &tokens {
        match *t {
            "public" => public = true,
            "protected" => protected = true,
            "internal" => internal = true,
            "private" => private = true,
            _ => {}
        }
    }
    if private {
        return "private".to_string();
    }
    if protected {
        return "protected".to_string();
    }
    if public {
        return "public".to_string();
    }
    if internal {
        return "internal".to_string();
    }
    default_visibility(sym)
}

/// Interface members (container kind 11) and enum members (container kind
/// 10) carry no modifier and are as exposed as their container — the cap in
/// [`cap_visibility`] brings them down to it.
fn default_visibility(sym: &RawSymbol) -> String {
    if sym.container_kind == 11 || sym.container_kind == 10 {
        "public".to_string()
    } else if sym.container.is_empty() {
        "internal".to_string()
    } else {
        "private".to_string()
    }
}

/// Effective visibility is capped by the container's: a public member of
/// an internal class is internal surface at most.
fn cap_visibility(own: &str, container: &str) -> String {
    if visibility_rank(own) <= visibility_rank(container) {
        own.to_string()
    } else {
        container.to_string()
    }
}

/// Attribute lines immediately above the declaration (`[...]`).
fn attribute_lines(lines: &[&str], start: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = start;
    while i > 0 {
        let prev = lines[i - 1].trim();
        if prev.starts_with('[') && prev.ends_with(']') {
            out.push(prev.to_string());
            i -= 1;
        } else {
            break;
        }
    }
    out.reverse();
    out
}

/// Does the project around `file` grant internals to friends? Presence
/// suggests the library posture — the caller warns toward flipping
/// `adapter.csharp.internal_is_surface` (PRD §14 question 2, settled).
pub fn internals_visible_to_near(file: &std::path::Path) -> bool {
    let Some(project_dir) = enclosing_project_dir(file) else { return false };
    let mut checked = 0usize;
    let mut stack = vec![project_dir];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().map(|e| e == "cs").unwrap_or(false) && checked < 200 {
                checked += 1;
                if std::fs::read_to_string(&p)
                    .map(|t| t.contains("InternalsVisibleTo"))
                    .unwrap_or(false)
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Walk up from the file to the nearest directory holding a `.csproj`.
fn enclosing_project_dir(file: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut dir = file.parent();
    for _ in 0..12 {
        let d = dir?;
        let has_project = std::fs::read_dir(d).ok().into_iter().flatten().flatten().any(|e| {
            e.path().extension().map(|x| x == "csproj").unwrap_or(false)
        });
        if has_project {
            return Some(d.to_path_buf());
        }
        dir = d.parent();
    }
    None
}
