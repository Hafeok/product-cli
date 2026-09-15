//! The separations the flow depends on, asserted against the manifests.
//!
//! Each of these is a rule the PRD states in prose and this repo happens to
//! hold as a dependency edge. A prose rule is re-broken by whoever has not
//! read it; an edge is re-broken by a compile error.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// The `[dependencies]` names a crate declares.
fn dependencies_of(crate_name: &str) -> Vec<String> {
    let manifest = workspace_root().join(crate_name).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|e| panic!("{}: {e}", manifest.display()));
    parse_dependency_names(&text)
}

/// Names under `[dependencies]`, stopping at the next section header.
fn parse_dependency_names(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == "[dependencies]";
            continue;
        }
        if !inside || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = trimmed.split_once('=') {
            names.push(name.trim().to_string());
        }
    }
    names
}

/// §3 — the domain model must be authorable without candidates in view.
///
/// The candidate vocabulary is transport-shaped by construction. If the model
/// were authored by walking the candidate list, the accrual vocabulary would
/// inherit the defect and every act would become an endpoint with a better
/// name. In this repo the event model is `product domain`, and the rule holds
/// because `product-core` cannot see `.spec/` at all — `map` is where the two
/// meet for the first time, which is exactly where the PRD puts them.
#[test]
fn the_event_model_cannot_see_the_candidate_list() {
    let deps = dependencies_of("product-core");
    assert!(
        !deps.iter().any(|d| d == "spec-core"),
        "product-core owns the event model; depending on spec-core would let \
         `model` display candidates, which §3 forbids. Found: {deps:?}"
    );
}

/// The downstream-consumer contract: `product-core` stays free of the
/// governance stacks built on top of it.
#[test]
fn product_core_stays_free_of_the_stacks_above_it() {
    let deps = dependencies_of("product-core");
    for forbidden in ["ddd-core", "ledger-core", "spec-core", "spec-mcp"] {
        assert!(
            !deps.iter().any(|d| d == forbidden),
            "product-core must not depend on {forbidden}: {deps:?}"
        );
    }
}

/// `spec-core` is the record substrate, and it reuses rather than re-states.
#[test]
fn spec_core_borrows_the_hashing_and_identity_laws() {
    let deps = dependencies_of("spec-core");
    assert!(
        deps.iter().any(|d| d == "ledger-core"),
        "one canonicalisation law and one identity law, never a second scheme: {deps:?}"
    );
}

/// The gate binary must not link the agent host's concerns, and vice versa.
#[test]
fn the_gate_binary_does_not_depend_on_the_mcp_surface() {
    let deps = dependencies_of("spec-cli");
    assert!(
        !deps.iter().any(|d| d == "spec-mcp"),
        "`spec` is the half a model may not call; it has no business carrying \
         the surface a model calls: {deps:?}"
    );
}

/// The `.spec/` store is the only thing the flow writes, and one crate writes it.
#[test]
fn only_spec_core_writes_the_store() {
    let write_call = "write_file_atomic";
    for crate_name in ["spec-cli", "spec-mcp"] {
        let src = workspace_root().join(crate_name).join("src");
        assert!(
            !contains_recursively(&src, write_call),
            "{crate_name} writes the store directly; every write goes through spec-core, \
             which judges the store-as-it-would-be first"
        );
    }
}

fn contains_recursively(dir: &Path, needle: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if path.is_dir() && contains_recursively(&path, needle) {
            return true;
        }
        if path.extension().is_some_and(|e| e == "rs")
            && std::fs::read_to_string(&path).is_ok_and(|t| t.contains(needle))
        {
            return true;
        }
    }
    false
}
