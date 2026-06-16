---
id: TC-944
title: archetype cells cross-check the domain
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
- stderr
runner: cargo-test
runner-args: tc_944_archetype_cells_cross_check_the_domain
---

## Scenario — a cell's domain input is checked against the What graph

**Given** an assembled archetype whose cell is `derived_from` a concrete
`domain:Ghost` pointer, and a captured What graph without a Ghost entity,
**When** the user runs `product archetype validate`,
**Then** the process exits 0 (a dangling domain pointer is a warning), stdout
reports `domain: cross-checked`, and stderr warns about `domain:Ghost`.

## Validates

- FT-114 — product archetype — assemble and validate How, layout, and cells as one
- ADR-056 — An archetype assembles How, layout, and cells from a directory and validates the whole
