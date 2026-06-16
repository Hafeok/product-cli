---
id: TC-955
title: decider simulate cel guard and payloads
type: scenario
status: passing
validates:
  features:
  - FT-122
  adrs:
  - ADR-063
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_955_decider_simulate_cel_guard_and_payloads
---

## Scenario — a CEL guard over a payload, with a computed emitted payload

**Given** an `account-decider` whose `Charge` command is guarded by the CEL
expression `command.amount <= state.limit` (state derived from an `Opened`
event's payload) and emits `Charged` with `amount: "=command.amount"`, plus two
scenarios (charge within limit → emit; charge over limit → reject),
**When** the user runs `product decider simulate account-decider`,
**Then** the process exits 0 and stdout reports `sound + complete` — the CEL
guard, the payload-derived state, and the computed emitted payload all evaluate
as expected.

## Validates

- FT-122 — product decider simulate — prove a Decider sound and complete before realisation
- ADR-063 — CEL is the Decider expression language for guards and assignments
