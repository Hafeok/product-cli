---
id: TC-970
title: dispatch instantiates work units
type: scenario
status: passing
validates:
  features:
  - FT-117
  adrs:
  - ADR-059
phase: 6
observes:
- exit-code
- file
runner: cargo-test
runner-args: tc_970_dispatch_instantiates_work_units
---

## Scenario — dispatch a task type into work units

**Given** the add-crud-resource task type and a captured What graph with an
Order entity,
**When** the user runs `product cell dispatch --bind entity=Order …` (binding
every required slot),
**Then** the process exits 0 and writes the work-unit files on disk (e.g.
`.product/work-units/contract-order.yaml`); the contract unit's
`derived_from` shows `domain:Order` (the slot resolved), with
`frozen: true` and a `sha256:` context hash.

## Validates

- FT-117 — product cell dispatch — instantiate a task type into frozen SPMC work units
- ADR-059 — Cell dispatch instantiates a task type into frozen work units bound to real entities
