---
id: TC-572
title: drift_check_structural_reports_file_changes
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
runner-args: tc_572_drift_check_structural_reports_file_changes
last-run: 2026-04-18T14:07:23.114790413+00:00
last-run-duration: 0.2s
---

## Session: ST-129 — drift-check-structural-reports-file-changes

**Validates:** FT-045, ADR-023 (amended), ADR-040

### Given

A temp repository with FT-001 complete (`product/FT-001/complete` tag exists) and two source files under `[drift].source-roots` modified after the tag.

### When

`product drift check FT-001` is run.

### Then

- stdout lists the two changed files with their per-file insertion/deletion counts.
- The output includes the completion tag name and timestamp.
- Exit code is `2` (warning — changes detected).
- A hint at the bottom of the output reads `Run: product drift diff FT-001 | your-llm "check for drift"` or equivalent.
- No LLM call was made.