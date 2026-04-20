---
id: TC-614
title: request_create_with_custom_type_validates_against_toml
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_614_request_create_with_custom_type_validates_against_toml"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.2s
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