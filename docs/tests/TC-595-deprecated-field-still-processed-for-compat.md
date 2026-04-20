---
id: TC-595
title: deprecated_field_still_processed_for_compat
type: scenario
status: unimplemented
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_595_deprecated_field_still_processed_for_compat
---

## Session: ST-149 — deprecated-field-still-processed-for-compat

### Given
The ST-148 fixture (an ADR uses a deprecated `source-files:` field).

### When
The graph is built and the affected ADR is queried via `product adr show`.

### Then
- The graph builds without error.
- The `source-files:` field is present and populated in the parsed artifact.
- The W023 warning does not block any operation; only a single warning is
  emitted to stderr.
