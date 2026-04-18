---
id: TC-571
title: drift_diff_no_changes_empty_diff_section
type: scenario
status: passing
validates:
  features:
  - FT-045
  adrs:
  - ADR-023
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_571_drift_diff_no_changes_empty_diff_section
last-run: 2026-04-18T14:07:23.114790413+00:00
last-run-duration: 0.2s
---

## Session: ST-128 — drift-diff-no-changes-empty-diff-section

**Validates:** FT-045, ADR-023 (amended), ADR-040

### Given

A temp repository with FT-001 marked complete (`product/FT-001/complete` tag exists) and **no** implementation-file changes since the tag.

### When

`product drift diff FT-001` is run.

### Then

- stdout is a well-formed bundle.
- The Changes Since Completion section is present but empty (or contains a one-line note: `(no changes since completion)`).
- All other sections (Instructions, Implementation Anchor, Governing ADRs) are populated normally.
- Exit code is `0`.