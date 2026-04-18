---
id: TC-614
title: request_create_with_custom_type_validates_against_toml
type: scenario
status: unimplemented
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
---

## Session: ST-193 — request-create-with-custom-type-validates-against-toml

### Given
A repository with `[tc-types].custom = ["contract"]`. A request YAML
containing one new TC with `tc-type: contract`.

### When
`product request validate` then `product request apply` is invoked.

### Then
- Validate: no findings.
- Apply: the TC is created, the file is written with `type: contract` in
  front-matter, and `graph_check_clean` is true.
