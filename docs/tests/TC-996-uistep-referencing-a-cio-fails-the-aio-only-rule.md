---
id: TC-996
title: UiStep referencing a CIO fails the AIO only rule
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
runner-args: tc_996_uistep_referencing_a_cio_fails_the_aio_only_rule
last-run: 2026-06-19T16:37:43.369842347+00:00
last-run-duration: 0.5s
---

## Scenario — a UiStep that names a concrete control is a structural violation

**Given** a captured What graph with a `UiStep` `ReviewOrder` whose interaction
references a concrete control (a CIO such as `primary-button`) rather than an
`Aio`-typed node,
**When** the user runs the framework's What-side conformance check (the
`pf::rules_ui` AIO-only rule, via `product graph check`),
**Then** the process exits non-zero and the AIO-only rule emits a
graph-conformance finding that names the offending step (`ReviewOrder`) and the
non-AIO reference — the type boundary is enforced structurally, not by review,
exactly as §3.2.1 requires ("a UI step naming a dropdown" is rejected like a
misplaced file in the layout allowlist).

## Validates

- FT-134 — Abstract Interaction Object vocabulary and the typed UiStep
- ADR-078 — UI steps are typed against Abstract Interaction Objects; AIOs and CIOs are graph nodes