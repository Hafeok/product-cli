//! C# adapter — Roslyn host wiring plus the §9.1 policy table.
//!
//! Host: `roslyn-language-server`, the official prerelease .NET global
//! tool (`--stdio --autoLoadProjects`, verified against 5.11.0). The
//! solution-open handshake plus readiness signal
//! (`workspace/projectInitializationComplete`) are handled by the host
//! layer via this adapter's declarations. Facts come from LSP symbols
//! plus declaration-text slicing only — no Roslyn API anywhere.

use crate::adapter::{Adapter, AdapterFlags};
use crate::protocol::{kind_name, RawSymbol};
use crate::surface::{ChangeKind, PolicyRow, SymbolFacts};

pub const ADAPTER: Adapter = Adapter {
    language: "csharp",
    artifact_class: "code",
    extensions: &["cs"],
    language_id: "csharp",
    default_command: &["roslyn-language-server", "--stdio", "--autoLoadProjects"],
    ready_flag: Some("workspace/projectInitializationComplete"),
    needs_open_handshake: true,
    facts,
    policy,
    visibility_rank,
    posture_warning,
};

/// `InternalsVisibleTo` grants friends access to internals — a seam. Under
/// the app-repo default, suggest flipping to the library posture
/// (PRD §14 question 2, settled by `dec/ddd/internal-not-surface`).
fn posture_warning(file: &std::path::Path, flags: &AdapterFlags) -> Option<String> {
    if !flags.internal_is_surface && internals_visible_to_near(file) {
        Some(
            "InternalsVisibleTo present near this file — internals are a seam here; consider `adapter.csharp.internal_is_surface: true` (library posture)"
                .to_string(),
        )
    } else {
        None
    }
}

const DECL_KINDS: &[&str] = &[
    "class", "interface", "struct", "enum", "method", "property", "field", "constructor",
    "event", "interface-member",
];
const EXPOSED: &[&str] = &["public", "protected"];

/// §9.1 as data. Rows are falsifiable claims; amend them here, never the
/// classifier (`dec/ddd/adapter-policy-tables`).
const BASE_ROWS: &[PolicyRow] = &[
    PolicyRow {
        id: "cs-exported-endpoint",
        changes: &[ChangeKind::Added, ChangeKind::SignatureChanged, ChangeKind::DecoratorChanged,
                   ChangeKind::Removed],
        kinds: &[],
        visibilities: &[],
        exported_only: true,
        surface: true,
        claim: "a configured exported-endpoint attribute makes any change to the symbol boundary-forming",
    },
    PolicyRow {
        id: "cs-interface-member",
        changes: &[ChangeKind::Added, ChangeKind::SignatureChanged, ChangeKind::Removed],
        kinds: &["interface-member"],
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a new or changed member on an exposed interface forces every implementor",
    },
    PolicyRow {
        id: "cs-add-exposed",
        changes: &[ChangeKind::Added],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a new public/protected type or member is contract surface",
    },
    PolicyRow {
        id: "cs-signature-exposed",
        changes: &[ChangeKind::SignatureChanged],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a signature change on public/protected surface (incl. a new constructor parameter on a public type) is a contract change",
    },
    PolicyRow {
        id: "cs-remove-exposed",
        changes: &[ChangeKind::Removed],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "removing public/protected surface is a contract change",
    },
    PolicyRow {
        id: "cs-visibility-boundary",
        changes: &[ChangeKind::VisibilityChanged],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "promoting to or demoting from public/protected moves the contract boundary",
    },
];

/// `internal` rows: the app-repo default treats internal as non-surface;
/// `adapter.csharp.internal_is_surface: true` flips to the library-repo
/// posture (`dec/ddd/internal-not-surface`).
const INTERNAL_NOT_SURFACE: PolicyRow = PolicyRow {
    id: "cs-internal-nonsurface",
    changes: &[],
    kinds: &[],
    visibilities: &["internal"],
    exported_only: false,
    surface: false,
    claim: "internal members are not contract surface in an app repo",
};
const INTERNAL_IS_SURFACE: PolicyRow = PolicyRow {
    id: "cs-internal-surface",
    changes: &[ChangeKind::Added, ChangeKind::SignatureChanged, ChangeKind::Removed,
               ChangeKind::VisibilityChanged],
    kinds: &[],
    visibilities: &["internal"],
    exported_only: false,
    surface: true,
    claim: "in a library repo internal is contract surface (InternalsVisibleTo is a seam)",
};
const PRIVATE_NOT_SURFACE: PolicyRow = PolicyRow {
    id: "cs-private-nonsurface",
    changes: &[],
    kinds: &[],
    visibilities: &["private"],
    exported_only: false,
    surface: false,
    claim: "private members are never contract surface",
};

fn policy(flags: &AdapterFlags) -> Vec<PolicyRow> {
    let internal =
        if flags.internal_is_surface { INTERNAL_IS_SURFACE } else { INTERNAL_NOT_SURFACE };
    let mut rows: Vec<PolicyRow> = BASE_ROWS.to_vec();
    rows.push(internal);
    rows.push(PRIVATE_NOT_SURFACE);
    rows
}

fn visibility_rank(v: &str) -> u8 {
    match v {
        "public" => 3,
        "protected" => 2,
        "internal" => 1,
        _ => 0,
    }
}

/// Derive per-symbol facts: normalized kind, effective visibility
/// (container-capped), body-free signature, attached attributes.
fn facts(text: &str, symbols: &[RawSymbol], flags: &AdapterFlags) -> Vec<SymbolFacts> {
    let lines: Vec<&str> = text.lines().collect();
    let mut effective: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    let mut out = Vec::new();
    for sym in symbols {
        let kind = normalize_kind(sym);
        if !DECL_KINDS.contains(&kind.as_str()) {
            continue;
        }
        let decl = declaration_slice(&lines, sym.start_line as usize);
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

fn default_visibility(sym: &RawSymbol) -> String {
    if sym.container_kind == 11 {
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
