//! C# adapter — Roslyn host wiring plus the §9.1 policy table.
//!
//! Host: `roslyn-language-server`, the official prerelease .NET global
//! tool (`--stdio --autoLoadProjects`, verified against 5.11.0). The
//! solution-open handshake plus readiness signal
//! (`workspace/projectInitializationComplete`) are handled by the host
//! layer via this adapter's declarations. Facts come from LSP symbols
//! plus declaration-text slicing only — no Roslyn API anywhere.

use crate::adapter::{csharp_facts, no_extra_capabilities, Adapter, AdapterFlags, ReadySignal};
use crate::surface::{ChangeKind, PolicyRow};

pub const ADAPTER: Adapter = Adapter {
    language: "csharp",
    artifact_class: "code",
    extensions: &["cs"],
    language_id: "csharp",
    default_command: &["roslyn-language-server", "--stdio", "--autoLoadProjects"],
    ready: ReadySignal::Notification("workspace/projectInitializationComplete"),
    extra_capabilities: no_extra_capabilities,
    needs_open_handshake: true,
    hosted: true,
    facts: csharp_facts::facts,
    policy,
    visibility_rank,
    posture_warning,
    enrich: None,
};

/// `InternalsVisibleTo` grants friends access to internals — a seam. Under
/// the app-repo default, suggest flipping to the library posture
/// (PRD §14 question 2, settled by `dec/ddd/internal-not-surface`).
fn posture_warning(file: &std::path::Path, flags: &AdapterFlags) -> Option<String> {
    if !flags.internal_is_surface && csharp_facts::internals_visible_to_near(file) {
        Some(
            "InternalsVisibleTo present near this file — internals are a seam here; consider `adapter.csharp.internal_is_surface: true` (library posture)"
                .to_string(),
        )
    } else {
        None
    }
}

/// `enum-member` is present per `dec/ddd/enum-member-gap-priced`: LSP kind 22
/// used to be dropped here before any row was consulted, which is the silent
/// hole `DDD-adapter-03` reports.
pub(crate) const DECL_KINDS: &[&str] = &[
    "class", "interface", "struct", "enum", "enum-member", "method", "property", "field",
    "constructor", "event", "interface-member",
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
        id: "cs-enum-member",
        changes: &[ChangeKind::Added, ChangeKind::Removed],
        kinds: &["enum-member"],
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a member added to or removed from an exposed enum breaks every consumer whose switch expression was exhaustive (DDD-adapter-03)",
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

pub(crate) fn visibility_rank(v: &str) -> u8 {
    match v {
        "public" => 3,
        "protected" => 2,
        "internal" => 1,
        _ => 0,
    }
}
