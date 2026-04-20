---
id: TC-599
title: migration_phase2_phase1_tc_unrunnable_no_block
type: migration
status: passing
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_599_migration_phase2_phase1_tc_unrunnable_no_block
last-run: 2026-04-20T08:01:08.284116371+00:00
last-run-duration: 0.2s
---

## Session: ST-153 — migration-phase2-phase1-tc-unrunnable-no-block

### Given
The ST-152 state, plus the phase-1 TC marked `unrunnable` with a documented
reason ("superseded by phase-2 absence TC").

### When
`product verify --platform` runs.

### Then
- The phase-1 TC is skipped (status `unrunnable`).
- The phase-2 TC runs and passes.
- Platform verify exits 0 — `unrunnable` does not contribute to failure.
- Graph check emits no error or warning for the unrunnable status.