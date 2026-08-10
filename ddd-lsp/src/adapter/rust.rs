//! Rust adapter — rust-analyzer host wiring plus the M6 policy table.
//!
//! Host: `rust-analyzer`, the rustup component the pinned toolchain already
//! carries. It discovers the workspace itself from `rootUri` (`cargo
//! metadata`), so there is no open handshake; readiness is
//! `experimental/serverStatus` with `quiescent: true`, which the host only
//! sends when the matching client capability is declared. Facts come from
//! LSP symbols plus declaration-text slicing — no rustc, no syn, no
//! `cargo` invocation anywhere in this crate.
//!
//! Visibility is the reason the slicing exists: LSP symbol kinds carry no
//! `pub(crate)` granularity, so the grade is read off the declaration text.
//! That is adapter knowledge by construction — the same technique the C#
//! adapter uses for its own modifiers.

use serde_json::{json, Value};

use crate::adapter::{rust_facts, Adapter, AdapterFlags, ReadySignal};
use crate::surface::{ChangeKind, PolicyRow};

pub const ADAPTER: Adapter = Adapter {
    language: "rust",
    artifact_class: "code",
    extensions: &["rs"],
    language_id: "rust",
    default_command: &["rust-analyzer"],
    ready: ReadySignal::NotificationWhere("experimental/serverStatus", quiescent),
    extra_capabilities: capabilities,
    needs_open_handshake: false,
    facts: rust_facts::facts,
    policy,
    visibility_rank,
    posture_warning,
};

/// rust-analyzer repeats `experimental/serverStatus` and carries the answer
/// in the payload: the first one says `quiescent: false` while indexing runs.
/// Readiness is the payload, not the arrival.
fn quiescent(params: &Value) -> bool {
    params.get("quiescent").and_then(Value::as_bool).unwrap_or(false)
}

/// Without `experimental.serverStatusNotification`, rust-analyzer sends no
/// server-status notification at all — measured, not assumed — and the host
/// would have to guess when semantic requests became trustworthy.
fn capabilities() -> Value {
    json!({"experimental": {"serverStatusNotification": true}})
}

/// Kinds the table rules on. `enum-member` is present deliberately: adding
/// a variant to a `pub` enum breaks a downstream exhaustive `match` exactly
/// as `DDD-adapter-03` describes for C#, and the gap that claim reports is
/// adapter-local, so this adapter does not inherit it.
pub const DECL_KINDS: &[&str] = &[
    "trait", "struct", "enum", "enum-member", "union", "field", "fn", "module", "const",
    "static", "type-alias", "trait-member", "inherent-impl", "trait-impl",
    "trait-impl-unresolved",
];

/// The visibilities a change is contract surface at under the app-repo
/// default.
const EXPOSED: &[&str] = &["public"];

/// The graded visibilities `pub(crate)` collapses into.
const GRADED: &[&str] = &["pub(crate)", "pub(super)", "pub(restricted)"];

/// The M6 table as data. Every row is a falsifiable claim about Rust;
/// wrong rows get amended here, never in the classifier
/// (`dec/ddd/adapter-policy-tables`).
const BASE_ROWS: &[PolicyRow] = &[
    PolicyRow {
        id: "rs-trait-impl",
        changes: &[ChangeKind::Added, ChangeKind::Removed],
        kinds: &["trait-impl"],
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a new trait impl on a reachable type is implicit contract participation — the type now satisfies an interface callers can dispatch on, and no `pub` keyword marks it",
    },
    PolicyRow {
        id: "rs-trait-impl-unresolved",
        changes: &[ChangeKind::Added, ChangeKind::Removed],
        kinds: &["trait-impl-unresolved"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "a trait impl whose trait and self type are both declared outside this file: coherence (the orphan-impl question) is not answerable from one document, so the known escape shape is demanded rather than dropped",
    },
    PolicyRow {
        id: "rs-trait-definition",
        changes: &[ChangeKind::Added, ChangeKind::SignatureChanged, ChangeKind::Removed],
        kinds: &["trait"],
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a public trait definition is contract surface twice over: it binds every implementor and every caller that dispatches on it",
    },
    PolicyRow {
        id: "rs-trait-member",
        changes: &[ChangeKind::Added, ChangeKind::SignatureChanged, ChangeKind::Removed],
        kinds: &["trait-member"],
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a new or changed member on a public trait forces every implementor — a method without a default body breaks them at compile time",
    },
    PolicyRow {
        id: "rs-derive-change",
        changes: &[ChangeKind::DecoratorChanged],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a derive or attribute change on a public item adds or removes trait impls callers may depend on; `#[derive(..)]` is trait-impl authorship in attribute syntax",
    },
    PolicyRow {
        id: "rs-add-exposed",
        changes: &[ChangeKind::Added],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a new `pub` item is contract surface",
    },
    PolicyRow {
        id: "rs-signature-exposed",
        changes: &[ChangeKind::SignatureChanged],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "a parameter, return-type, or generic-bound change on a `pub` item is a contract change",
    },
    PolicyRow {
        id: "rs-remove-exposed",
        changes: &[ChangeKind::Removed],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "removing `pub` surface is a contract change",
    },
    PolicyRow {
        id: "rs-visibility-boundary",
        changes: &[ChangeKind::VisibilityChanged],
        kinds: DECL_KINDS,
        visibilities: EXPOSED,
        exported_only: false,
        surface: true,
        claim: "promoting to or demoting from `pub` moves the contract boundary",
    },
];

/// The graded-visibility pair. `pub(crate)` is Rust's `internal`, so it
/// rides the same config switch and the same default-off posture
/// (`dec/ddd/internal-not-surface`); one asymmetry is recorded on the
/// row itself.
const GRADED_NOT_SURFACE: PolicyRow = PolicyRow {
    id: "rs-crate-visible-nonsurface",
    changes: &[],
    kinds: &[],
    visibilities: GRADED,
    exported_only: false,
    surface: false,
    claim: "`pub(crate)`/`pub(super)` items are not contract surface in an app repo, and unlike C# `internal` there is no InternalsVisibleTo to open them — the compiler enforces the bound",
};
const GRADED_IS_SURFACE: PolicyRow = PolicyRow {
    id: "rs-crate-visible-surface",
    changes: &[ChangeKind::Added, ChangeKind::SignatureChanged, ChangeKind::Removed,
               ChangeKind::VisibilityChanged],
    kinds: &[],
    visibilities: GRADED,
    exported_only: false,
    surface: true,
    claim: "in a multi-crate workspace governed as one unit, `pub(crate)` is the seam between modules that review treats as an interface",
};
const PRIVATE_NOT_SURFACE: PolicyRow = PolicyRow {
    id: "rs-private-nonsurface",
    changes: &[],
    kinds: &[],
    visibilities: &["private"],
    exported_only: false,
    surface: false,
    claim: "private items are never contract surface",
};

fn policy(flags: &AdapterFlags) -> Vec<PolicyRow> {
    let graded = if flags.internal_is_surface { GRADED_IS_SURFACE } else { GRADED_NOT_SURFACE };
    let mut rows: Vec<PolicyRow> = BASE_ROWS.to_vec();
    rows.push(graded);
    rows.push(PRIVATE_NOT_SURFACE);
    rows
}

/// Order by exposure. `pub(crate)` reaches further than `pub(super)`;
/// `pub(in path)` sits between them because the path is not resolved here.
fn visibility_rank(v: &str) -> u8 {
    match v {
        "public" => 4,
        "pub(crate)" => 3,
        "pub(restricted)" => 2,
        "pub(super)" => 1,
        _ => 0,
    }
}

/// Rust's escape from container capping is the re-export: `pub use` of a
/// path rooted in a private module publishes an item this adapter judges
/// at its declaration site, where it looks capped. Flagged, because the
/// understatement is silent otherwise.
fn posture_warning(file: &std::path::Path, _flags: &AdapterFlags) -> Option<String> {
    let text = std::fs::read_to_string(file).ok()?;
    let reexports = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("pub use ") || l.starts_with("pub(crate) use "))
        .count();
    if reexports == 0 {
        return None;
    }
    Some(format!(
        "{reexports} re-export(s) in this file — `pub use` can publish an item whose declaration site is container-capped, so effective visibility here may understate the surface"
    ))
}

#[cfg(test)]
#[path = "rust_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "rust_policy_tests.rs"]
mod policy_tests;
