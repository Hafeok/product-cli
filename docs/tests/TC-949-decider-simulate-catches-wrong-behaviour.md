---
id: TC-949
title: decider simulate catches wrong behaviour
type: scenario
status: passing
validates:
  features:
  - FT-122
  adrs:
  - ADR-062
phase: 6
observes:
- exit-code
- stderr
runner: cargo-test
runner-args: tc_949_decider_simulate_catches_wrong_behaviour
---

## Scenario — a scenario that contradicts the logic fails simulation

**Given** the `order-decider`, but with the `cannot pay unplaced` scenario edited
to expect `emit: [OrderPaid]` (it must reject, since the pay guard requires a
placed order),
**When** the user runs `product decider simulate order-decider`,
**Then** the process exits 1 and stderr reports the failing scenario
`cannot pay unplaced`.

## Validates

- FT-122 — product decider simulate — prove a Decider sound and complete before realisation
- ADR-062 — Decider logic is a declarative guarded state machine simulated as pure functions
