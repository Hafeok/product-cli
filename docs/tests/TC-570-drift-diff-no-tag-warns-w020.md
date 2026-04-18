---
id: TC-570
title: drift_diff_no_tag_warns_w020
type: scenario
status: unimplemented
validates:
  features:
  - FT-045
  adrs:
  - ADR-023
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_570_drift_diff_no_tag_warns_w020
---

## Session: ST-127 — drift-diff-no-tag-warns-w020

**Validates:** FT-045, ADR-023 (amended), ADR-040

### Given

A temp repository with FT-001 in status `in-progress` — no completion tag exists.

### When

`product drift diff FT-001` is run.

### Then

- stderr contains a W020 warning stating no completion tag exists for FT-001.
- stdout still emits a well-formed bundle with the Instructions and Governing ADRs sections; the Implementation Anchor section lists `completion-tag: (none)`; the Changes Since Completion section is empty with a note explaining why.
- Exit code is `2` (warning).
