---
id: TC-594
title: w023_fires_on_deprecated_field
type: scenario
status: unimplemented
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_594_w023_fires_on_deprecated_field
---

## Session: ST-148 — w023-fires-on-deprecated-field

### Given
A repository with an accepted ADR whose `deprecates: [source-files]`, and a
separate ADR file whose front-matter contains a `source-files:` field.

### When
`product graph check` runs.

### Then
- One W023 warning is reported.
- The warning message names the field (`source-files`) and the deprecating
  ADR by ID.
- Exit code is 2 (warnings only).
