//! Integration tests — link.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_356_link_tests_basic() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Test Feature
phase: 1
status: planned
adrs:
- ADR-002
tests: []
---

Feature body.
");
    h.write("docs/adrs/ADR-002-domain.md", "\
---
id: ADR-002
title: Domain ADR
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-002-test.md", "\
---
id: TC-002
title: Test Criterion
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-002
phase: 1
---

TC body.
");
    let out = h.run(&["migrate", "link-tests"]);
    out.assert_exit(0);

    // TC-002 gains validates.features: [FT-001]
    let tc = h.read("docs/tests/TC-002-test.md");
    assert!(tc.contains("FT-001"), "TC-002 should gain FT-001 in validates.features. Got:\n{}", tc);

    // FT-001 gains tests: [TC-002]
    let ft = h.read("docs/features/FT-001-test.md");
    assert!(ft.contains("TC-002"), "FT-001 should gain TC-002 in tests. Got:\n{}", ft);
}

#[test]
fn tc_357_link_tests_multi_feature() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Feature One
phase: 1
status: planned
adrs:
- ADR-002
tests: []
---

Feature one.
");
    h.write("docs/features/FT-005-test.md", "\
---
id: FT-005
title: Feature Five
phase: 1
status: planned
adrs:
- ADR-002
tests: []
---

Feature five.
");
    h.write("docs/adrs/ADR-002-domain.md", "\
---
id: ADR-002
title: Domain ADR
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-002-test.md", "\
---
id: TC-002
title: Test Criterion
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-002
phase: 1
---

TC body.
");
    let out = h.run(&["migrate", "link-tests"]);
    out.assert_exit(0);

    // TC-002 gains both FT-001 and FT-005
    let tc = h.read("docs/tests/TC-002-test.md");
    assert!(tc.contains("FT-001"), "TC-002 should contain FT-001. Got:\n{}", tc);
    assert!(tc.contains("FT-005"), "TC-002 should contain FT-005. Got:\n{}", tc);
}

#[test]
fn tc_358_link_tests_cross_cutting_excluded() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Feature One
phase: 1
status: planned
adrs:
- ADR-001
tests: []
---

Feature.
");
    h.write("docs/features/FT-002-test.md", "\
---
id: FT-002
title: Feature Two
phase: 1
status: planned
adrs:
- ADR-001
tests: []
---

Feature.
");
    h.write("docs/adrs/ADR-001-cross.md", "\
---
id: ADR-001
title: Cross Cutting ADR
status: accepted
scope: cross-cutting
---

Cross-cutting ADR.
");
    h.write("docs/tests/TC-001-test.md", "\
---
id: TC-001
title: Cross Cutting Test
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-001
phase: 1
---

TC body.
");
    let out = h.run(&["migrate", "link-tests"]);
    out.assert_exit(0);

    // TC-001.validates.features remains empty
    let tc = h.read("docs/tests/TC-001-test.md");
    assert!(!tc.contains("FT-001"), "TC-001 should NOT gain FT-001 (cross-cutting excluded). Got:\n{}", tc);
    assert!(!tc.contains("FT-002"), "TC-001 should NOT gain FT-002 (cross-cutting excluded). Got:\n{}", tc);

    // Features should not gain TC-001
    let ft1 = h.read("docs/features/FT-001-test.md");
    assert!(!ft1.contains("TC-001"), "FT-001 should NOT gain TC-001. Got:\n{}", ft1);

    // Output should mention skipping
    assert!(out.stdout.contains("skipped") || out.stdout.contains("cross-cutting") || out.stdout.contains("0 new links"),
        "Output should mention skipping cross-cutting. Got:\n{}", out.stdout);
}

#[test]
fn tc_359_link_tests_idempotent() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Test Feature
phase: 1
status: planned
adrs:
- ADR-002
tests: []
---

Feature body.
");
    h.write("docs/adrs/ADR-002-domain.md", "\
---
id: ADR-002
title: Domain ADR
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-002-test.md", "\
---
id: TC-002
title: Test Criterion
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-002
phase: 1
---

TC body.
");
    // First run
    let out1 = h.run(&["migrate", "link-tests"]);
    out1.assert_exit(0);

    let tc_after_first = h.read("docs/tests/TC-002-test.md");
    let ft_after_first = h.read("docs/features/FT-001-test.md");

    // Second run
    let out2 = h.run(&["migrate", "link-tests"]);
    out2.assert_exit(0);

    let tc_after_second = h.read("docs/tests/TC-002-test.md");
    let ft_after_second = h.read("docs/features/FT-001-test.md");

    // File content identical after both runs
    assert_eq!(tc_after_first, tc_after_second, "TC file should be identical after second run");
    assert_eq!(ft_after_first, ft_after_second, "Feature file should be identical after second run");

    // Second run reports "0 new links"
    assert!(out2.stdout.contains("0 new links"), "Second run should report 0 new links. Got:\n{}", out2.stdout);
}

#[test]
fn tc_360_link_tests_dry_run_no_write() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Test Feature
phase: 1
status: planned
adrs:
- ADR-002
tests: []
---

Feature body.
");
    h.write("docs/adrs/ADR-002-domain.md", "\
---
id: ADR-002
title: Domain ADR
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-002-test.md", "\
---
id: TC-002
title: Test Criterion
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-002
phase: 1
---

TC body.
");
    let tc_before = h.read("docs/tests/TC-002-test.md");
    let ft_before = h.read("docs/features/FT-001-test.md");

    let out = h.run(&["migrate", "link-tests", "--dry-run"]);
    out.assert_exit(0);

    // No files modified
    let tc_after = h.read("docs/tests/TC-002-test.md");
    let ft_after = h.read("docs/features/FT-001-test.md");
    assert_eq!(tc_before, tc_after, "TC file should be unchanged after dry-run");
    assert_eq!(ft_before, ft_after, "Feature file should be unchanged after dry-run");

    // Stdout contains inference plan
    assert!(out.stdout.contains("dry run"), "Output should mention dry run. Got:\n{}", out.stdout);
    assert!(out.stdout.contains("TC-002") || out.stdout.contains("FT-001"),
        "Output should mention affected artifacts. Got:\n{}", out.stdout);
}

#[test]
fn tc_361_link_tests_adr_scope() {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Test Feature
phase: 1
status: planned
adrs:
- ADR-002
- ADR-006
tests: []
---

Feature body.
");
    h.write("docs/adrs/ADR-002-domain.md", "\
---
id: ADR-002
title: Domain ADR Two
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/adrs/ADR-006-domain.md", "\
---
id: ADR-006
title: Domain ADR Six
status: accepted
scope: domain
---

ADR body.
");
    h.write("docs/tests/TC-002-test.md", "\
---
id: TC-002
title: TC for ADR-002
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-002
phase: 1
---

TC body.
");
    h.write("docs/tests/TC-006-test.md", "\
---
id: TC-006
title: TC for ADR-006
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-006
phase: 1
---

TC body.
");

    // Run with --adr ADR-002 filter
    let out = h.run(&["migrate", "link-tests", "--adr", "ADR-002"]);
    out.assert_exit(0);

    // TC-002 should be updated (linked to ADR-002)
    let tc2 = h.read("docs/tests/TC-002-test.md");
    assert!(tc2.contains("FT-001"), "TC-002 should gain FT-001. Got:\n{}", tc2);

    // TC-006 should NOT be updated (linked to ADR-006, not in scope)
    let tc6 = h.read("docs/tests/TC-006-test.md");
    assert!(!tc6.contains("FT-001"), "TC-006 should NOT gain FT-001 (not in --adr scope). Got:\n{}", tc6);
}

