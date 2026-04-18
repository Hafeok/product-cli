---
id: TC-615
title: request_create_unknown_type_emits_e006
type: scenario
status: unimplemented
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
---

## Session: ST-194 — request-create-unknown-type-emits-e006

### Given
A repository with `[tc-types].custom = ["contract"]`. A request YAML
containing one new TC with `tc-type: regression` (not configured).

### When
`product request validate` is invoked.

### Then
- One E006 finding is reported.
- The location is the JSONPath of the offending artifact.
- The message lists built-in types AND the configured custom list
  (`["contract"]`).
- `product request apply` on the same input fails with exit 1 and writes no
  files.
