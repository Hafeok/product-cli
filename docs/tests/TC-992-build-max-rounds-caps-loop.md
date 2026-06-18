---
id: TC-992
title: build max-rounds caps the fix loop
type: scenario
status: passing
validates:
  features:
  - FT-131
  adrs:
  - ADR-071
phase: 6
observes:
- stdout
runner: cargo-test
runner-args: tc_992_build_max_rounds_caps_the_fix_loop
---

## Scenario — --max-rounds bounds escalation

**Given** a deliverable whose acceptance runner never passes (the scripted
worker only ever writes a failing output),
**When** the user runs `product build conv --role coder --max-rounds 0`,
**Then** the verify gate records the failing verdict but **never re-dispatches a
fix** — proving the operational round cap bounds cost/runtime rather than
escalating indefinitely.

## Validates

- FT-131 — product build gates — operational controls (--max-rounds, budget)
- ADR-071 — verification is recorded, not judged
