---
id: TC-612
title: bundle_type_ordering_exit_criteria_first
type: scenario
status: unimplemented
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
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
