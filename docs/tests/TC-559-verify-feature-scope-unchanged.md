---
id: TC-559
title: verify_feature_scope_unchanged
type: scenario
status: passing
validates:
  features:
  - FT-044
  adrs:
  - ADR-021
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_559_verify_feature_scope_unchanged
last-run: 2026-04-18T10:54:35.903075401+00:00
last-run-duration: 0.2s
---

## Session: ST-117 — verify-feature-scope-unchanged

**Validates:** FT-044, ADR-040, ADR-021 (`product verify FT-XXX` retains existing behaviour)

### Given

A temp repository with a feature whose TCs are configured with cargo-test runners.

### When

`product verify FT-001` (per-feature form, positional argument) is run.

### Then

- Only that feature's TCs are executed — no pipeline stages are run.
- Feature status is updated per the existing per-feature rules (complete on all pass, in-progress on any fail).
- Completion tag `product/FT-001/complete` is created on transition to complete (unchanged ADR-036 behaviour).
- Output format is the existing per-feature output, not the pipeline report.
- Exit code follows the existing per-feature semantics.