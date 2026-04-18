---
id: TC-553
title: verify_fails_on_e_class_graph_error
type: scenario
status: passing
validates:
  features:
  - FT-044
  adrs:
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_553_verify_fails_on_e_class_graph_error
last-run: 2026-04-18T10:54:35.903075401+00:00
last-run-duration: 0.2s
---

## Session: ST-111 — verify-fails-on-e-class-graph-error

**Validates:** FT-044, ADR-040 (Stage 2 E-class failure fails the pipeline)

### Given

A temp repository that introduces a broken reference — a feature whose `adrs` array contains `ADR-999` which does not exist, triggering E002.

### When

`product verify` is run with no arguments.

### Then

- Stage 2 (graph-structure) emits `fail` with finding code `E002`.
- All six stages still run to completion — stages 1, 3, 4, 5, 6 are reported even though stage 2 failed.
- Exit code is `1`.
- `--ci` JSON mode: `stages[1].status == "fail"`, `findings` contains `"E002"`, top-level `passed: false`, `exit: 1`.