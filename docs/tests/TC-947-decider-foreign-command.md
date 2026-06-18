---
id: TC-947
title: decider foreign command nonconformant
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
- stderr
runner: cargo-test
runner-args: tc_947_decider_foreign_command_nonconformant
---

## Scenario — a Decider handling a foreign command fails validation

**Given** a captured What graph where only `PlaceOrder` targets the `Order`
aggregate, and an authored decider whose `handles` includes `ForeignCmd` (which
does not target `Order`),
**When** the user runs `product decider validate order-decider`,
**Then** the process exits 1 and stderr reports the `ForeignCmd` no-foreign-commands
violation (§3.3).

## Validates

- FT-121 — product decider — derive an aggregate's executable signature and validate drift
- ADR-061 — A Decider's signature is derived from and validated against the event model
