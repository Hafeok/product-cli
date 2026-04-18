---
id: TC-555
title: verify_fails_on_failing_tc
type: scenario
status: unimplemented
validates:
  features:
  - FT-044
  adrs:
  - ADR-021
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_555_verify_fails_on_failing_tc
---

## Session: ST-113 — verify-fails-on-failing-tc

**Validates:** FT-044, ADR-040, ADR-021 (Stage 5 TC failure fails the pipeline)

### Given

A temp repository with a feature whose linked TC has `status: failing` in front-matter and whose runner reports a non-zero exit.

### When

`product verify` is run with no arguments.

### Then

- Stage 5 (feature-tcs) emits `fail` with at least one finding object naming the failing TC and its feature.
- Exit code is `1`.
- Pretty output shows the failing TC on its own indented line under stage 5 with `FAIL` marker.
- `--ci` JSON mode: `stages[4].status == "fail"`, `findings[0]` has `{ tc, feature, status: "failing" }`.
