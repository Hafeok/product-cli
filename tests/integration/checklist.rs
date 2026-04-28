//! Integration tests — checklist.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_021_checklist_generate() {
    let h = fixture_checklist_three_features();

    let out = h.run(&["checklist", "generate"]);
    out.assert_exit(0);

    let checklist = h.read("docs/checklist.md");

    // Should contain correct status markers
    assert!(
        checklist.contains("FT-001") && checklist.contains("[~]"),
        "Checklist should show FT-001 as in-progress [~].\nChecklist:\n{}",
        checklist
    );
    assert!(
        checklist.contains("FT-002") && checklist.contains("[x]"),
        "Checklist should show FT-002 as complete [x].\nChecklist:\n{}",
        checklist
    );
    assert!(
        checklist.contains("FT-003") && checklist.contains("[ ]"),
        "Checklist should show FT-003 as planned [ ].\nChecklist:\n{}",
        checklist
    );

    // Should not contain YAML front-matter delimiters
    assert!(
        !checklist.starts_with("---"),
        "Checklist should not contain YAML front-matter.\nChecklist:\n{}",
        checklist
    );

    // Should contain phase headers
    assert!(
        checklist.contains("## Phase 1"),
        "Checklist should have Phase 1 header.\nChecklist:\n{}",
        checklist
    );
    assert!(
        checklist.contains("## Phase 2"),
        "Checklist should have Phase 2 header.\nChecklist:\n{}",
        checklist
    );
}

#[test]
fn tc_022_checklist_no_manual_edit_warning() {
    let h = fixture_checklist_three_features();

    let out = h.run(&["checklist", "generate"]);
    out.assert_exit(0);

    let checklist = h.read("docs/checklist.md");

    // Must begin with the header and warning block
    assert!(
        checklist.starts_with("# Implementation Checklist"),
        "Checklist should start with '# Implementation Checklist'.\nChecklist:\n{}",
        checklist
    );
    assert!(
        checklist.contains("Do not edit directly"),
        "Checklist should contain 'Do not edit directly' warning.\nChecklist:\n{}",
        checklist
    );
    assert!(
        checklist.contains("product checklist generate"),
        "Warning should reference 'product checklist generate'.\nChecklist:\n{}",
        checklist
    );
}

#[test]
fn tc_023_checklist_roundtrip() {
    let h = fixture_checklist_three_features();

    // First generation
    let out = h.run(&["checklist", "generate"]);
    out.assert_exit(0);

    let checklist_v1 = h.read("docs/checklist.md");
    // FT-001 starts as in-progress
    assert!(
        checklist_v1.contains("FT-001") && checklist_v1.contains("[~]"),
        "Initial checklist should show FT-001 as in-progress.\nChecklist:\n{}",
        checklist_v1
    );

    // Change FT-001 status from in-progress to complete
    h.write(
        "docs/features/FT-001-alpha.md",
        "---\nid: FT-001\ntitle: Alpha Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nAlpha body.\n",
    );

    // Regenerate
    let out = h.run(&["checklist", "generate"]);
    out.assert_exit(0);

    let checklist_v2 = h.read("docs/checklist.md");

    // FT-001 should now show as complete
    // Find the line containing FT-001 and verify it has [x] not [~]
    let ft001_line = checklist_v2
        .lines()
        .find(|l| l.contains("FT-001"))
        .expect("FT-001 should appear in checklist");
    assert!(
        ft001_line.contains("[x]"),
        "After status change, FT-001 should show [x] (complete), got: {}",
        ft001_line
    );
    assert!(
        !ft001_line.contains("[~]"),
        "After status change, FT-001 should no longer show [~] (in-progress), got: {}",
        ft001_line
    );

    // No residue: the old in-progress marker for FT-001 should not appear
    // (count occurrences of FT-001 — should appear exactly once as a heading)
    let ft001_headings: Vec<&str> = checklist_v2
        .lines()
        .filter(|l| l.contains("FT-001") && l.starts_with("###"))
        .collect();
    assert_eq!(
        ft001_headings.len(),
        1,
        "FT-001 should appear exactly once as a heading (no residue).\nHeadings: {:?}\nChecklist:\n{}",
        ft001_headings, checklist_v2
    );
}

#[test]
fn tc_159_checklist_generation_idempotent() {
    let h = fixture_checklist_three_features();

    // Generate twice
    let out1 = h.run(&["checklist", "generate"]);
    out1.assert_exit(0);
    let checklist_first = h.read("docs/checklist.md");

    let out2 = h.run(&["checklist", "generate"]);
    out2.assert_exit(0);
    let checklist_second = h.read("docs/checklist.md");

    // Both generations should produce identical output (ignoring timestamp which uses the same day)
    assert_eq!(
        checklist_first, checklist_second,
        "Two consecutive checklist generations should produce identical output.\nFirst:\n{}\nSecond:\n{}",
        checklist_first, checklist_second
    );
}

#[test]
fn tc_209_checklist_gitignore_default() {
    let h = Harness::new();
    // Remove existing product.toml to simulate a new repository
    let _ = std::fs::remove_file(h.dir.path().join("product.toml"));

    let out = h.run(&["init", "--yes"]);
    out.assert_exit(0);

    // product.toml should exist
    assert!(
        h.exists("product.toml"),
        "product.toml should be created by init"
    );

    // .gitignore should exist and contain checklist.md
    assert!(
        h.exists(".gitignore"),
        ".gitignore should be created by init"
    );
    let gitignore = h.read(".gitignore");
    assert!(
        gitignore.contains("checklist.md"),
        "checklist.md should appear in .gitignore by default.\nGot:\n{}",
        gitignore
    );
}

#[test]
fn tc_210_checklist_gitignore_opt_out() {
    let h = Harness::new();
    // Pre-create product.toml with checklist-in-gitignore = false
    h.write(
        "product.toml",
        r#"name = "test"
schema-version = "1"
checklist-in-gitignore = false

[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"

[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
"#,
    );

    let out = h.run(&["init", "--force", "--yes"]);
    out.assert_exit(0);

    // .gitignore should exist (for docs/graph/ at least)
    assert!(
        h.exists(".gitignore"),
        ".gitignore should be created by init"
    );
    let gitignore = h.read(".gitignore");

    // checklist.md should NOT appear in .gitignore
    assert!(
        !gitignore.contains("checklist.md"),
        "checklist.md should NOT appear in .gitignore when checklist-in-gitignore = false.\nGot:\n{}",
        gitignore
    );

    // docs/graph/ should still be present (always gitignored)
    assert!(
        gitignore.contains("docs/graph/"),
        "docs/graph/ should still appear in .gitignore.\nGot:\n{}",
        gitignore
    );
}

