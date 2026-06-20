---
id: TC-1001
title: forgotten failed state fails coverage
type: scenario
status: passing
validates:
  features:
  - FT-136
  adrs:
  - ADR-080
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1001_forgotten_failed_state_fails_coverage
last-run: 2026-06-20T17:54:52.495698456+00:00
last-run-duration: 0.5s
---

## Scenario — a projection that can fail whose screen never says what failure means

**Given** a captured What graph with a read model `OrderSummary` whose declared
state space is {`present`, `empty`, `failed`},
**And** a `UiStep` `ReviewOrder` that `surfaces` `OrderSummary` but annotates
meanings only for `present` and `empty` — the `failed` state is neither meant
nor waived,
**When** the user runs the framework's What-side conformance check (the
`pf::rules_ui` state-coverage rule, via `product graph check`),
**Then** the process exits non-zero and the covering half of the rule emits a
finding naming the forgotten state (`failed` on `ReviewOrder`) — the dangerous
case §3.2.1 exists to catch is rejected by construction, not left to review.

## Validates

- FT-136 — Read-model state space and UI-step state coverage
- ADR-080 — Read models declare a state space; UI steps cover it constrained and complete