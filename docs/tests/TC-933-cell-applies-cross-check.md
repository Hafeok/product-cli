---
id: TC-933
title: cell validate cross-checks applies against how
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
runner-args: tc_933_cell_validate_cross_checks_applies_against_how
---

## Scenario — a cell's applied pattern is checked against the How contract

**Given** the example task type (whose handler cell applies `vertical-slice`)
and the example How contract (which does not define that pattern),
**When** the user runs `product cell validate`,
**Then** the process exits 0 and stderr warns that `vertical-slice` is not a
pattern/principle in the How contract while stdout reports `how: cross-checked`.

## Validates

- FT-113 — product cell — validate task-types against the What graph and How contract
- ADR-055 — Task-types (cells) are cross-validated against the captured What graph and How contract
