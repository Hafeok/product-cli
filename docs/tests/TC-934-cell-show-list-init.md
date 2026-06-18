---
id: TC-934
title: cell show list and init
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
runner-args: tc_934_cell_show_list_and_init
---

## Scenario — scaffold, then inspect

**Given** a repo with no task type,
**When** the user runs `product cell init add-crud-resource --archetype rest-api`,
**Then** the process exits 0 and writes `.product/cell.yaml`; a second `init`
without `--force` exits 1; `cell validate` on the scaffold exits 0; and
`cell show` / `cell list slots` stdout report the task-type id and slots.

## Validates

- FT-113 — product cell — validate task-types against the What graph and How contract
- ADR-055 — Task-types (cells) are cross-validated against the captured What graph and How contract
