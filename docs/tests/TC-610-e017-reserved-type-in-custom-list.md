---
id: TC-610
title: e017_reserved_type_in_custom_list
type: scenario
status: unimplemented
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
---

## Session: ST-189 — e017-reserved-type-in-custom-list

### Given
A repository with `[tc-types].custom = ["contract", "exit-criteria"]`.

### When
Any `product` command is invoked.

### Then
- Product exits 1 with E017.
- The error message names `exit-criteria` as the offending entry.
- The error message lists all four reserved names.
- No subcommand mechanics run (no graph build, no validation, no context
  assembly).
- Same behaviour for each of the other three reserved names individually.
