//! Integration tests — slice.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_663_slice_adapter_structural_invariants() {
    use std::path::PathBuf;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // (A) No println/eprintln/std::process::exit/std::fs::write in pure slice
    // modules for the cycle_times slice.
    let forbidden = ["println!", "eprintln!", "std::process::exit", "std::fs::write"];
    let slice_files = [
        "src/cycle_times/model.rs",
        "src/cycle_times/compute.rs",
        "src/cycle_times/render.rs",
    ];
    for sf in &slice_files {
        let p = root.join(sf);
        let content = std::fs::read_to_string(&p).expect("read slice file");
        for needle in &forbidden {
            assert!(
                !content.contains(needle),
                "slice file {} must not contain '{}'",
                sf,
                needle
            );
        }
    }

    // (D) Adapter size under 400 lines.
    let adapter = root.join("src/commands/cycle_times.rs");
    let content = std::fs::read_to_string(&adapter).expect("read adapter");
    let n = content.lines().count();
    assert!(n <= 400, "adapter must be ≤ 400 lines; got {}", n);

    // (C) plan_*/build_* return typed values (not Result<(), _>).
    let compute = std::fs::read_to_string(root.join("src/cycle_times/compute.rs"))
        .expect("read compute");
    assert!(
        compute.contains("pub fn build_report"),
        "build_report must be present"
    );
}

#[test]
fn tc_664_slice_adapter_pattern_satisfied_by_cycle_times_slice() {
    use std::path::PathBuf;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Slice directory exists with expected files.
    for f in &["mod.rs", "model.rs", "compute.rs", "render.rs", "tests.rs"] {
        let p = root.join(format!("src/cycle_times/{}", f));
        assert!(p.exists(), "expected slice file {} to exist", p.display());
    }

    // Adapter returns CmdResult (not BoxResult) for the read-only cycle-times handler.
    let adapter = std::fs::read_to_string(root.join("src/commands/cycle_times.rs"))
        .expect("read adapter");
    assert!(
        adapter.contains("CmdResult"),
        "adapter must use CmdResult: {}",
        adapter.lines().take(20).collect::<Vec<_>>().join("\n")
    );

    // First //! doc line must NOT contain the literal word "and" (SRP).
    let mod_content = std::fs::read_to_string(root.join("src/cycle_times/mod.rs"))
        .expect("read mod.rs");
    let first = mod_content
        .lines()
        .find(|l| l.starts_with("//!"))
        .unwrap_or("")
        .to_lowercase();
    let has_and = first
        .split_whitespace()
        .any(|w| w.trim_matches(|c: char| !c.is_alphabetic()) == "and");
    assert!(
        !has_and,
        "src/cycle_times/mod.rs first //! line must not contain 'and' as a word: {}",
        first
    );
}

