---
id: TC-948
title: decider simulate sound and complete
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
- stdout
runner: cargo-test
runner-args: tc_948_decider_simulate_sound_and_complete
---

## Scenario — a sound, complete Decider simulates clean

**Given** an `order-decider` with a guarded state machine (Place → Pay) and three
scenarios covering both commands (place fresh, cannot-pay-unplaced, place-then-pay),
**When** the user runs `product decider simulate order-decider`,
**Then** the process exits 0 and stdout reports `sound + complete` over
`3 scenario(s)`.

## Validates

- FT-122 — product decider simulate — prove a Decider sound and complete before realisation
- ADR-062 — Decider logic is a declarative guarded state machine simulated as pure functions
