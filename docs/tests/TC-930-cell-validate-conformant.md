---
id: TC-930
title: cell validate passes on conformant example
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
- stdout
runner: cargo-test
runner-args: tc_930_cell_validate_conformant_example
---

## Scenario — validate a conformant task type

**Given** the bundled example task type at `.product/cell.yaml`,
**When** the user runs `product cell validate`,
**Then** the process exits 0 and stdout reports `conformant` with the slot,
cell, and audit counts.

## Validates

- FT-113 — product cell — validate task-types against the What graph and How contract
- ADR-055 — Task-types (cells) are cross-validated against the captured What graph and How contract
