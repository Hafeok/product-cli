---
id: TC-1015
title: design system manifest validates internally
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
runner-args: tc_1015_design_system_manifest_validates_internally
---

## Scenario — a whole design-system manifest validates; a dangling reification cio fails

**Given** a design-system manifest (the §11.3 schema) in which every `cio` named
in `reification` exists in `components`, every token a component references is
declared in `tokens`, and every `criterion` a component claims under `satisfies`
is a real WCAG 2.2 entity,
**When** the user validates the manifest,
**Then** the process exits 0 and the validator reports the manifest internally
whole.

**And given** a second manifest whose `reification` names a `cio` absent from
`components`, **when** the user validates it, **then** the process exits non-zero
and the validator emits a finding naming the offending reification rule and the
missing component — internal wholeness is enforced, not assumed.

## Validates

- FT-141 — Design System Conformance Profile (preview)
- ADR-085 — Preview conformance profiles for the design system and the content store
