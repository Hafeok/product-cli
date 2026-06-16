---
id: TC-956
title: decider conform matching runner passes
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
- stdout
runner: cargo-test
runner-args: tc_956_decider_conform_matching_runner_passes
---

## Scenario — realised code matching the oracle is behaviourally conformant

**Given** the `order-decider` (with logic + three scenarios) and a runner whose
JSON outcomes match the Decider's simulated outcomes
(`[emit OrderPlaced, reject pay-only-placed, emit OrderPaid]`),
**When** the user runs `product decider conform order-decider --runner "<cmd>"`,
**Then** the process exits 0 and stdout reports `behaviourally conformant`.

## Validates

- FT-123 — product decider conform — check realised code against the Decider oracle
- ADR-064 — Behavioural conformance replays the Decider scenarios against a pluggable runner
