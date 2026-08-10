//! HTML+CSS pair adapter — the M7 policy table over a hostless producer.
//!
//! The governed unit is the **pair**: an HTML document plus the CSS it
//! consumes, declared in `.ddd/config.yaml` (`pair.units`). The failures
//! this class exists for live at their seam — orphan classes, dead
//! selectors, undeclared token changes — so a single-file view of either
//! side is the bug, not the feature. There is no language server behind
//! this adapter (M7 experiment §2): facts are parsed from source text in
//! `htmlcss_facts`, the cross-file checks live in `htmlcss_pair`, and the
//! host fields below are inert (`hosted: false`).

use crate::adapter::{no_extra_capabilities, Adapter, AdapterFlags, ReadySignal};
use crate::surface::{ChangeKind, PolicyRow};

pub const ADAPTER: Adapter = Adapter {
    language: "htmlcss",
    artifact_class: "html-css",
    extensions: &["html", "htm", "css"],
    language_id: "html",
    default_command: &[],
    ready: ReadySignal::AfterInitialized,
    extra_capabilities: no_extra_capabilities,
    needs_open_handshake: false,
    hosted: false,
    facts: super::htmlcss_facts::facts,
    policy,
    visibility_rank,
    posture_warning,
    enrich: Some(super::htmlcss_pair::enrich),
};

/// Kinds the table rules on. Everything else the pair contains — markup
/// structure, styling bodies, inline styles — is deliberately not lifted
/// by the facts fn, so no row needs to exempt it.
pub const DECL_KINDS: &[&str] = &["token", "class", "layer-order", "stylesheet-link"];

/// The M7 table as data, pre-registered in docs/ddd-m7-experiment.md §4.5.
/// Every row is a falsifiable claim about the pair; wrong rows get amended
/// here, never in the classifier (`dec/ddd/adapter-policy-tables`).
const ROWS: &[PolicyRow] = &[
    PolicyRow {
        id: "web-token-membership",
        changes: &[ChangeKind::Added, ChangeKind::Removed],
        kinds: &["token"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "design tokens are the stylesheet's params: a custom property added or removed changes the contract every consuming rule and document depends on",
    },
    PolicyRow {
        id: "web-token-retype",
        changes: &[ChangeKind::SignatureChanged],
        kinds: &["token"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "retyping a token (dimension <-> color <-> keyword) is a contract change; a value change within the type is no event at all — that silence is what a token buys",
    },
    PolicyRow {
        id: "web-class-membership",
        changes: &[ChangeKind::Added, ChangeKind::Removed],
        kinds: &["class"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "the class vocabulary is the pair's shared contract — consumed by HTML, defined by owned CSS; membership changes are where orphan classes and dead selectors are born",
    },
    PolicyRow {
        id: "web-layer-order",
        changes: &[ChangeKind::Added, ChangeKind::Removed, ChangeKind::SignatureChanged],
        kinds: &["layer-order"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "@layer order is the visibility grading of the whole sheet: reordering it re-ranks every rule's authority at once",
    },
    PolicyRow {
        id: "web-stylesheet-link",
        changes: &[ChangeKind::Added, ChangeKind::Removed],
        kinds: &["stylesheet-link"],
        visibilities: &[],
        exported_only: false,
        surface: true,
        claim: "which stylesheets an HTML document consumes is the pair boundary itself; rewiring it moves every other contract in this table",
    },
];

fn policy(_flags: &AdapterFlags) -> Vec<PolicyRow> {
    ROWS.to_vec()
}

/// Cascade exposure: unlayered declarations outrank every `@layer`, so
/// they are the more exposed side of any paired change.
fn visibility_rank(v: &str) -> u8 {
    if v == "unlayered" {
        2
    } else if v.starts_with("layer:") {
        1
    } else {
        0
    }
}

/// A sheet that grades some rules with `@layer` while leaving others
/// unlayered has an escape from its own grading — the unlayered rules win
/// the cascade regardless of the declared order.
fn posture_warning(file: &std::path::Path, _flags: &AdapterFlags) -> Option<String> {
    if file.extension().map(|e| e != "css").unwrap_or(true) {
        return None;
    }
    let text = std::fs::read_to_string(file).ok()?;
    if !text.contains("@layer") {
        return None;
    }
    let unlayered = super::htmlcss_facts::selector_class_names(&text)
        .iter()
        .any(|(off, _)| !layered_at(&text, *off));
    if unlayered {
        Some(
            "this sheet mixes @layer with unlayered rules — unlayered declarations outrank every layer, so the declared grading understates their exposure"
                .to_string(),
        )
    } else {
        None
    }
}

/// Whether the byte offset sits inside any `@layer` block.
fn layered_at(text: &str, offset: usize) -> bool {
    let mut depth_layer = 0i32;
    for (pos, c) in text.char_indices() {
        if pos >= offset {
            break;
        }
        match c {
            '{' => {
                let head_start = text[..pos].rfind(['{', '}', ';']).map(|p| p + 1).unwrap_or(0);
                if text[head_start..pos].trim_start().starts_with("@layer") || depth_layer > 0 {
                    depth_layer += 1;
                }
            }
            '}' if depth_layer > 0 => depth_layer -= 1,
            _ => {}
        }
    }
    depth_layer > 0
}

#[cfg(test)]
#[path = "htmlcss_policy_tests.rs"]
mod policy_tests;
