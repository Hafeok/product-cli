---
id: TC-969
title: deliverable done lifecycle
type: scenario
status: passing
validates:
  features:
  - FT-127
  adrs:
  - ADR-068
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_969_deliverable_done_lifecycle
---

## Scenario — done is computed; acceptance is recorded, not judged

**Given** a slice over a conformant What subgraph and a deliverable on it with a
pending acceptance criterion,
**When** the user runs `product deliverable done place-order`,
**Then** the process exits 1 and reports `not done` (the in-scope domain checks
pass, acceptance is pending).

**And when** the user runs `product deliverable accept place-order a1 --pass` and
re-runs `done`, **then** it exits 0 and reports `DONE`.

## Validates

- FT-127 — product deliverable/release done — the §7.2 computed delivery predicates
- ADR-068 — Done is computed from existing verifications plus recorded acceptance
