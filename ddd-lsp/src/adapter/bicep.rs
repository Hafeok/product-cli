//! Bicep adapter — bicep-ls host wiring plus the §9.2 policy table.
//!
//! Host: `bicep-ls` from the `Azure.Bicep.LangServer` .NET global tool —
//! plain stdio LSP, no open handshake, ready after `initialized`
//! (Microsoft documents exactly this integration surface). Facts come
//! from LSP symbols plus declaration slicing; Bicep syntax makes ground
//! provenance first-class, so each param carries its provenance class
//! for the seam entry (PRD §9.2).

use crate::adapter::{no_extra_capabilities, Adapter, AdapterFlags, ReadySignal};
use crate::protocol::RawSymbol;
use crate::surface::{ChangeKind, PolicyRow, SymbolFacts};

pub const ADAPTER: Adapter = Adapter {
    language: "bicep",
    artifact_class: "configuration",
    extensions: &["bicep", "bicepparam"],
    language_id: "bicep",
    default_command: &["bicep-ls"],
    ready: ReadySignal::AfterInitialized,
    extra_capabilities: no_extra_capabilities,
    needs_open_handshake: false,
    hosted: true,
    facts,
    policy,
    visibility_rank,
    posture_warning,
    enrich: None,
};

fn posture_warning(_file: &std::path::Path, _flags: &AdapterFlags) -> Option<String> {
    None
}

/// §9.2 as data: the module contract is params, outputs, plus resource
/// name/scope expressions; decorator changes are tolerance changes.
const ROWS: &[PolicyRow] = &[
    PolicyRow {
        id: "bi-param-membership",
        changes: &[ChangeKind::Added, ChangeKind::Removed],
        kinds: &["param"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a module param added or removed changes the module contract",
    },
    PolicyRow {
        id: "bi-param-retype",
        changes: &[ChangeKind::SignatureChanged],
        kinds: &["param"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "retyping a module param changes the module contract",
    },
    PolicyRow {
        id: "bi-param-tolerance",
        changes: &[ChangeKind::DecoratorChanged],
        kinds: &["param"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "@allowed and other param decorators are tolerance — changing them is a contract change",
    },
    PolicyRow {
        id: "bi-output-contract",
        changes: &[ChangeKind::Added, ChangeKind::Removed, ChangeKind::SignatureChanged],
        kinds: &["output"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a module output change is a contract change",
    },
    PolicyRow {
        id: "bi-resource-contract",
        changes: &[ChangeKind::SignatureChanged],
        kinds: &["resource"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "resource name/scope expressions cross module boundaries — changing them is a contract change",
    },
    PolicyRow {
        id: "bi-internal",
        changes: &[],
        kinds: &[],
        visibilities: &["internal"],
        exported_only: false,
        surface: false,
        claim: "variables plus resource bodies are module-internal",
    },
];

fn policy(_flags: &AdapterFlags) -> Vec<PolicyRow> {
    ROWS.to_vec()
}

fn visibility_rank(v: &str) -> u8 {
    if v == "module-contract" {
        1
    } else {
        0
    }
}

/// Derive facts per declaration; the first token names the kind (the LSP
/// symbol kind number varies across bicep-ls versions, the syntax does not).
fn facts(text: &str, symbols: &[RawSymbol], _flags: &AdapterFlags) -> Vec<SymbolFacts> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for sym in symbols {
        let start = sym.start_line as usize;
        let Some(decl_line) = lines.get(start) else { continue };
        let kind = match decl_line.split_whitespace().next() {
            Some("param") => "param",
            Some("output") => "output",
            Some("var") => "var",
            Some("resource") => "resource",
            Some("module") => "module",
            Some("type") => "type",
            Some(_) | None => continue,
        };
        let visibility = if kind == "param" || kind == "output" {
            "module-contract"
        } else {
            "internal"
        };
        let mut extra = std::collections::BTreeMap::new();
        if kind == "param" {
            extra.insert("ground_provenance".to_string(), provenance(decl_line).to_string());
        }
        out.push(SymbolFacts {
            name: sym.name.clone(),
            container: sym.container.clone(),
            kind: kind.to_string(),
            visibility: visibility.to_string(),
            signature: signature(kind, &lines, sym),
            decorators: decorator_lines(&lines, start),
            exported: false,
            extra,
            sel_line: sym.sel_line,
            sel_char: sym.sel_char,
        });
    }
    out
}

/// The contract-bearing slice per kind: params keep name+type (defaults
/// are not contract — PRD lists add/remove/retype plus decorators);
/// outputs keep the whole declaration; resources keep their header plus
/// `name:`/`scope:` lines — the expressions that cross module boundaries.
fn signature(kind: &str, lines: &[&str], sym: &RawSymbol) -> String {
    let start = sym.start_line as usize;
    let decl = lines.get(start).copied().unwrap_or("");
    let collapsed = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    match kind {
        "param" => collapsed(decl.split('=').next().unwrap_or(decl)),
        "resource" => {
            let mut parts = vec![collapsed(decl.split('=').next().unwrap_or(decl))];
            for line in lines.iter().skip(start + 1).take(sym.end_line as usize - start) {
                let t = line.trim();
                if t.starts_with("name:") || t.starts_with("scope:") {
                    parts.push(collapsed(t));
                }
            }
            parts.join(" | ")
        }
        _ => collapsed(decl),
    }
}

/// Decorator lines immediately above the declaration (`@...`).
fn decorator_lines(lines: &[&str], start: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = start;
    while i > 0 {
        let prev = lines[i - 1].trim();
        if prev.starts_with('@') {
            out.push(prev.to_string());
            i -= 1;
        } else {
            break;
        }
    }
    out.reverse();
    out
}

/// §9.2 ground-provenance mapping: literal default = controlled; no
/// default = observed (deploy-time caller input); a default computed by a
/// function call = inferred.
fn provenance(decl_line: &str) -> &'static str {
    match decl_line.split_once('=') {
        None => "observed",
        Some((_, default)) => {
            if default.contains('(') {
                "inferred"
            } else {
                "controlled"
            }
        }
    }
}
