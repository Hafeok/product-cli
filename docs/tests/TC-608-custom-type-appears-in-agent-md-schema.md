---
id: TC-608
title: custom_type_appears_in_agent_md_schema
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_608_custom_type_appears_in_agent_md_schema"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.1s
---

## Session: ST-187 — custom-type-appears-in-agent-md-schema

### Given
A repository with `[tc-types].custom = ["contract", "migration", "smoke"]`.

### When
`product agent-init` (or `product agent-context`) renders the schema.

### Then
- The TC type schema lists the four structural types annotated `(structural)`.
- It lists the two built-in descriptive types annotated
  `(built-in descriptive)`.
- It lists `contract`, `migration`, `smoke` annotated
  `(custom — this project)`.
- The custom list is taken from the loaded `product.toml` and reflects any
  change without re-installing Product.