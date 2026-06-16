---
id: TC-971
title: dispatched work units validate
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
- stdout
runner: cargo-test
runner-args: tc_971_dispatched_work_units_validate
---

## Scenario — the dispatched units are conformant SPMC manifests

**Given** the work units produced by a dispatch,
**When** the user runs `product work-unit validate --file <a produced unit>`,
**Then** the process exits 0 and stdout reports `domain: cross-checked` — a
dispatched unit is itself a conformant work unit against the What graph.

## Validates

- FT-117 — product cell dispatch — instantiate a task type into frozen SPMC work units
- ADR-059 — Cell dispatch instantiates a task type into frozen work units bound to real entities
