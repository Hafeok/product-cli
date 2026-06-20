---
id: TC-1000
title: UI step covers every projection state
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
runner-args: tc_1000_ui_step_covers_every_projection_state
last-run: 2026-06-20T17:54:52.495698456+00:00
last-run-duration: 22.5s
---

## Scenario — a UI step that means every state of its projection is conformant

**Given** a captured What graph with a read model `OrderSummary` whose declared
state space is {`present`, `empty`, `failed`},
**And** a `UiStep` `ReviewOrder` that `surfaces` `OrderSummary` and annotates a
meaning for each of `present`, `empty`, and `failed`,
**When** the user runs the framework's What-side conformance check (the
`pf::rules_ui` state-coverage rule, via `product graph check`),
**Then** the process exits 0 and the coverage rule reports no violation for
`ReviewOrder` — every state in the projection's declared alphabet is meant.

## Validates

- FT-136 — Read-model state space and UI-step state coverage
- ADR-080 — Read models declare a state space; UI steps cover it constrained and complete