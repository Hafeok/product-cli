---
id: TC-973
title: dispatch requires all required slots
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
runner-args: tc_973_dispatch_requires_all_required_slots
---

## Scenario — every required slot must be bound

**Given** the task type with five required slots,
**When** the user runs `product cell dispatch --bind entity=Order` only,
**Then** the process exits 1 and stderr reports a required slot is not bound.

## Validates

- FT-117 — product cell dispatch — instantiate a task type into frozen SPMC work units
- ADR-059 — Cell dispatch instantiates a task type into frozen work units bound to real entities
