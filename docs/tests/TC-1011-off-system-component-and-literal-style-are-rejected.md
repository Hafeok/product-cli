---
id: TC-1011
title: off-system component and literal style are rejected
type: scenario
status: unimplemented
validates:
  features:
  - FT-139
  adrs:
  - ADR-083
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_1011_off_system_component_and_literal_style_are_rejected
---

## Scenario — reification stays on-system and styling stays in tokens

**Given** a design system whose closed CIO catalog does not contain
`fancy-carousel`,
**And** a reification rule that `reifies` `fancy-carousel`,
**When** the user runs the closed-vocabulary check,
**Then** the process exits non-zero and the check names the offending rule and
the off-system component — reification may choose among the system's components
but may not invent one.

**And given** a screen that carries a literal style value (e.g. `#3366ff`)
instead of a `Token` reference, **when** the tokens-not-literals check runs,
**then** the process exits non-zero and the screen is reported non-conformant.

## Validates

- FT-139 — Design system and reification rules
- ADR-083 — Screens bind to a design system; AIOs reify to CIOs by context of use
