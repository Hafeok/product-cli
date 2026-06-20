---
id: TC-1002
title: waiving an ignorable state passes with reason
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
runner-args: tc_1002_waiving_an_ignorable_state_passes_with_reason
last-run: 2026-06-20T17:54:52.495698456+00:00
last-run-duration: 0.4s
---

## Scenario — an ignorable state is dismissable on the record; an impossible one is not

**Given** a captured What graph with a read model `OrderSummary` whose declared
state space is {`present`, `loading`, `empty`, `failed`},
**And** a `UiStep` `ReviewOrder` that means `present`, `empty`, and `failed` and
**waives** `loading` with a written reason ("the fold is too fast to perceive a
load state"),
**When** the user runs the `pf::rules_ui` state-coverage rule (via `product
graph check`),
**Then** the process exits 0 — the waiver-with-reason satisfies the covering
half, the escape hatch the Decider's command coverage does not get.

**And given** a second `UiStep` that annotates a meaning for a state the
projection does *not* declare (e.g. `empty` on a projection whose space is only
{`present`, `failed`}), **when** the same rule runs, **then** it exits non-zero
and the constrained half emits a finding naming the impossible (step, state)
pair.

## Validates

- FT-136 — Read-model state space and UI-step state coverage
- ADR-080 — Read models declare a state space; UI steps cover it constrained and complete