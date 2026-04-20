---
id: TC-612
title: bundle_type_ordering_exit_criteria_first
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_612_bundle_type_ordering_exit_criteria_first"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.2s
---

## Session: ST-191 — bundle-type-ordering-exit-criteria-first

### Given
A feature with one TC of each built-in type, declared in random order in the
feature's `tests:` list.

### When
`product context FT-XXX` is invoked.

### Then
- The first TC rendered is the `exit-criteria` TC.
- The second TC is the `invariant` TC.
- The third is the `chaos` TC.
- The fourth is the `absence` TC.
- The fifth is the `scenario` TC.
- The sixth is the `benchmark` TC.