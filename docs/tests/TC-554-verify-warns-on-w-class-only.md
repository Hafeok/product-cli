---
id: TC-554
title: verify_warns_on_w_class_only
type: scenario
status: unimplemented
validates:
  features:
  - FT-044
  adrs:
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_554_verify_warns_on_w_class_only
---

## Session: ST-112 — verify-warns-on-w-class-only

**Validates:** FT-044, ADR-040 (Stage 2 W-class only yields exit 2)

### Given

A temp repository with at least one W-class warning (e.g. a completed feature with an unimplemented TC triggering W016) and no E-class errors.

### When

`product verify` is run with no arguments.

### Then

- Stage 2 (graph-structure) emits `warning`, not `fail`.
- Exit code is `2`.
- `--ci` JSON mode: stages are mostly `pass` except stage 2 which is `warning`; top-level `passed: false`, `exit: 2`.
- Pretty output shows `Result: PASS (with warnings)` or equivalent and lists the W-code per stage.
