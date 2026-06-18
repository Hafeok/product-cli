---
id: TC-932
title: cell validate cross-checks domain pointers
type: scenario
status: passing
validates:
  features:
  - FT-113
  adrs:
  - ADR-055
phase: 6
observes:
- exit-code
- stderr
- stdout
runner: cargo-test
runner-args: tc_932_cell_validate_cross_checks_domain_pointers
---

## Scenario — a cell's domain input is checked against the What graph

**Given** a task type whose cell is `derived_from` a concrete `domain:Ghost`
pointer, and a captured What graph that has no Ghost entity,
**When** the user runs `product cell validate`,
**Then** the process exits 0 (a dangling domain pointer is a warning) and
stderr warns about `domain:Ghost` while stdout reports `domain: cross-checked`.

## Validates

- FT-113 — product cell — validate task-types against the What graph and How contract
- ADR-055 — Task-types (cells) are cross-validated against the captured What graph and How contract
