---
id: TC-573
title: drift_check_no_changes_exits_0
type: scenario
status: failing
validates:
  features:
  - FT-045
  adrs:
  - ADR-023
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_573_drift_check_no_changes_exits_0
last-run: 2026-04-18T10:42:24.877521871+00:00
last-run-duration: 0.1s
failure-message: "No matching test function found (0 tests ran)"
---

## Session: ST-130 — drift-check-no-changes-exits-0

**Validates:** FT-045, ADR-023 (amended), ADR-040

### Given

A temp repository with FT-001 complete (`product/FT-001/complete` tag exists) and **no** source-file changes since the tag.

### When

`product drift check FT-001` is run.

### Then

- stdout reports the completion tag and the message `No changes since completion.` (or equivalent).
- Exit code is `0`.
- No LLM call was made.