---
id: TC-957
title: decider conform divergent runner fails
type: scenario
status: passing
validates:
  features:
  - FT-123
  adrs:
  - ADR-064
phase: 6
observes:
- exit-code
- stderr
runner: cargo-test
runner-args: tc_957_decider_conform_divergent_runner_fails
---

## Scenario — realised code that diverges from the oracle fails conformance

**Given** the `order-decider` and a runner that wrongly *accepts* paying an
unplaced order (emitting `OrderPaid` where the Decider rejects with
`pay-only-placed`),
**When** the user runs `product decider conform order-decider --runner "<cmd>"`,
**Then** the process exits 1 and stderr reports the divergent scenario
`cannot pay unplaced`.

## Validates

- FT-123 — product decider conform — check realised code against the Decider oracle
- ADR-064 — Behavioural conformance replays the Decider scenarios against a pluggable runner
