---
id: TC-557
title: verify_phase_scope_flag
type: scenario
status: passing
validates:
  features:
  - FT-044
  adrs:
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_557_verify_phase_scope_flag
last-run: 2026-04-18T10:54:35.903075401+00:00
last-run-duration: 0.2s
---

## Session: ST-115 — verify-phase-scope-flag

**Validates:** FT-044, ADR-040 (`--phase N` scopes stage 5 to one phase)

### Given

A temp repository with features in phase 1 and phase 2, all complete and passing.

### When

`product verify --phase 1` is run.

### Then

- Stages 1, 2, 3, 4, 6 all run normally.
- Stage 5 reports only phase-1 features; phase-2 features are not enumerated (not even as `skipped`).
- Exit code reflects phase-1 results only.
- `--ci` JSON: stages[4].findings references only phase-1 features.