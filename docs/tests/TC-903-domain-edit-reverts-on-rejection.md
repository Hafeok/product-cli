---
id: TC-903
title: domain edit reverts on rejection
type: scenario
status: passing
validates:
  features:
  - FT-110
  adrs:
  - ADR-053
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_903_domain_edit_reverts_on_rejection
---

## Scenario — a rejected edit is reverted, a valid edit persists

**Given** a conformant entity `Order` in context `Sales`,
**When** the user runs `product domain edit Order --context Ghost` (a missing
context),
**Then** the process exits 1 and a subsequent `domain show Order` stdout
still reports context `Sales`; a later valid `edit --definition` exits 0 and
`show` stdout reflects the new definition.

## Validates

- FT-110 — product domain — CLI list, show, and CRUD over the captured What graph
- ADR-053 — Domain authoring is a separate What graph with native in-loop conformance
