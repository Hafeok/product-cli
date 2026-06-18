---
id: TC-940
title: archetype validate full assembly
type: scenario
status: passing
validates:
  features:
  - FT-114
  adrs:
  - ADR-056
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_940_archetype_validate_full_assembly
---

## Scenario — validate a fully assembled archetype

**Given** `.product/archetypes/example-rest-api/` containing a How contract, a
layout model, and one task-type cell,
**When** the user runs `product archetype validate example-rest-api`,
**Then** the process exits 0 and stdout reports `conformant` with
`how present, layout present, 1 cell(s)`.

## Validates

- FT-114 — product archetype — assemble and validate How, layout, and cells as one
- ADR-056 — An archetype assembles How, layout, and cells from a directory and validates the whole
