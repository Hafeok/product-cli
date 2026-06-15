---
id: TC-935
title: cell validate without file is a clear error
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
runner: cargo-test
runner-args: tc_935_cell_validate_without_file_is_a_clear_error
---

## Scenario — a missing task-type file is a clear error

**Given** a repo with no task-type file,
**When** the user runs `product cell validate`,
**Then** the process exits 1 and stderr explains there is no task type,
pointing at `product cell init`.

## Validates

- FT-113 — product cell — validate task-types against the What graph and How contract
- ADR-055 — Task-types (cells) are cross-validated against the captured What graph and How contract
