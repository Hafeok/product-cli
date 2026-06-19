---
id: TC-1016
title: design system coupling covers every AIO context
type: scenario
status: unimplemented
validates:
  features:
  - FT-141
  adrs:
  - ADR-085
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1016_design_system_coupling_covers_every_aio_context
---

## Scenario — reification coverage over (AIO, context) is the design-system command-coverage analogue

**Given** a design-system manifest whose `reification` provides a
`reify(AIO, context) → CIO` rule for every core AIO across every context in
`contexts_supported`, and a captured What graph declaring those contexts of use,
**When** the user runs the coupling check,
**Then** the process exits 0 and the check reports reification coverage complete
— no core AIO is left unrealised in any claimed context.

**And given** a manifest missing a rule for one (AIO, context) pair (e.g.
`single-select` on `phone`), **when** the user runs the coupling check, **then**
the process exits non-zero and the check declares the design system
**non-conforming for that context**, naming the missing (AIO, context) pair —
the design-system analogue of a Decider's command coverage.

## Validates

- FT-141 — Design System Conformance Profile (preview)
- ADR-085 — Preview conformance profiles for the design system and the content store
