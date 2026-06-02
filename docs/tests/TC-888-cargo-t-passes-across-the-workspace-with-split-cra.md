---
id: TC-888
title: cargo t passes across the workspace with split crates
type: exit-criteria
status: passing
validates:
  features:
  - FT-107
  adrs:
  - ADR-018
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_888_cargo_t_passes_across_the_workspace_with_split_crates
last-run: 2026-06-02T19:16:30.693377638+00:00
last-run-duration: 0.2s
---

## Description

Verifies the headline contract of FT-107: every test that passes
today still passes after the split. `cargo t` (alias for
`cargo test --no-fail-fast`) runs at the workspace root and
exercises every test binary across every member.

## Procedure

1. From the repo root, run `cargo t --workspace` and capture
   **exit-code** and **stdout**.
2. Parse the per-binary summary lines from **stdout**
   (`test result: ok. N passed; ...`).
3. Assert the sum across all binaries equals or exceeds the
   pre-split baseline (820 tests as of commit 4bfd6db). The exact
   number may grow as new tests land alongside this feature; it
   may never shrink.
4. Assert **exit-code** is `0`.

## Expected

- Step 1 exits `0`.
- Step 3 finds every pre-existing test binary
  (`code_quality_tests`, `integration_tests`, `property_tests`,
  `sessions`, plus per-crate `--lib` and `--doc` runs) present in
  **stdout** with `0 failed`.

This TC asserts via the test runner's own report, which is the
only proof that the split did not silently drop a binary. A
missing binary would show as a lower test count or an absent
summary line — both caught by step 3.