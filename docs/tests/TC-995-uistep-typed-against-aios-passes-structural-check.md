---
id: TC-995
title: UiStep typed against AIOs passes structural check
type: scenario
status: passing
validates:
  features:
  - FT-134
  adrs:
  - ADR-078
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_995_uistep_typed_against_aios_passes_structural_check
last-run: 2026-06-19T16:37:43.369842347+00:00
last-run-duration: 0.5s
---

## Scenario — a UiStep whose interactions are all AIO-typed is structurally conformant

**Given** a captured What graph with a read model `OrderSummary` and a command
`ConfirmOrder`,
**And** a `UiStep` `ReviewOrder` that `surfaces` `OrderSummary` through a
`display-collection` AIO and `offers` `ConfirmOrder` through a `trigger-action`
AIO, each interaction `typed_as` exactly one `Aio` node,
**When** the user runs the framework's What-side conformance check (the
`pf::rules_ui` AIO-only rule, via `product graph check`),
**Then** the process exits 0 and the structural AIO-only rule reports no
violation for `ReviewOrder` — every interaction it references resolves to an
`Aio`-typed node in the graph.

## Validates

- FT-134 — Abstract Interaction Object vocabulary and the typed UiStep
- ADR-078 — UI steps are typed against Abstract Interaction Objects; AIOs and CIOs are graph nodes