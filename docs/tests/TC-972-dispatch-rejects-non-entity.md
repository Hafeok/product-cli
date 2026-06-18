---
id: TC-972
title: dispatch rejects binding to non-entity
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
- stderr
runner: cargo-test
runner-args: tc_972_dispatch_rejects_binding_to_non_entity
---

## Scenario — a binding must name a real entity

**Given** the task type and a What graph without a Ghost entity,
**When** the user runs `product cell dispatch --bind entity=Ghost …`,
**Then** the process exits 1, stderr reports that the value is not an entity in
the What graph, and no work units are written.

## Validates

- FT-117 — product cell dispatch — instantiate a task type into frozen SPMC work units
- ADR-059 — Cell dispatch instantiates a task type into frozen work units bound to real entities
