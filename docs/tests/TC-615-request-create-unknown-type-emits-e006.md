---
id: TC-615
title: request_create_unknown_type_emits_e006
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_615_request_create_unknown_type_emits_e006"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.1s
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