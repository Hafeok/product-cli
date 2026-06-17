---
id: TC-983
title: recording conformance flips done
type: scenario
status: passing
validates:
  features:
  - FT-130
  adrs:
  - ADR-071
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_983_recording_conformance_flips_done
---

## Scenario — done reflects realised behavioural conformance

**Given** a deliverable whose slice covers an aggregate with a sound + complete
Decider, with acceptance recorded passing,
**When** the user runs `product deliverable done`,
**Then** it exits 1 — behavioural conformance is `behavioural-conform` pending.

**And when** the user records a passing verdict via `product decider conform
Order-decider --runner "<matching>"` and re-runs `done`, **then** it exits 0 and
reports `DONE`.

## Validates

- FT-130 — product build — the SPMC build orchestrator that records conformance into done
- ADR-071 — build assembles the SPMC context; conformance is recorded into done
