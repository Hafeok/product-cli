---
id: TC-987
title: build parallel plan lists work units
type: scenario
status: passing
validates:
  features:
  - FT-132
  adrs:
  - ADR-073
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_987_build_parallel_plan_lists_work_units
---

## Scenario — build fans work units across workers (the parallel plan)

**Given** a deliverable over a captured What graph, a scaffolded worker catalog,
and two work units under `.product/work-units/`,
**When** the user runs `product build place-order --jobs 4 --dry-run`,
**Then** it exits 0 and stdout shows a `Parallel run plan` of `4 job(s) over 2
work unit(s)`, listing each work unit (`wu-a`, `wu-b`) → its resolved capability.

## Validates

- FT-132 — parallel work-unit execution — build fans units across workers
- ADR-073 — Work units are the parallel unit, fanned out bounded with a coherence gate
