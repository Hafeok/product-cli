---
id: TC-946
title: decider derive and validate
type: scenario
status: passing
validates:
  features:
  - FT-121
  adrs:
  - ADR-061
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_946_decider_derive_and_validate
---

## Scenario — derive a Decider's signature and validate it against the model

**Given** a captured What graph with an aggregate `Order` and a command
`PlaceOrder` that targets it and emits `OrderPlaced`,
**When** the user runs `product decider derive Order`,
**Then** the process exits 0, reports `Derived decider 'Order-decider'`, and
writes `.product/deciders/Order-decider.yaml`.

**And when** the user runs `product decider validate Order-decider`, **then** the
process exits 0 and stdout reports `conformant` — the derived signature matches
the event model by construction. `show` and `list` surface the decider.

## Validates

- FT-121 — product decider — derive an aggregate's executable signature and validate drift
- ADR-061 — A Decider's signature is derived from and validated against the event model
