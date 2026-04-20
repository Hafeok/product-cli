---
id: TC-607
title: custom_type_treated_as_scenario_in_mechanics
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_607_custom_type_treated_as_scenario_in_mechanics"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.2s
---

## Session: ST-186 — custom-type-treated-as-scenario-in-mechanics

### Given
A repository with two TCs validating the same feature: one of `type:
scenario`, one of a configured custom type (`type: contract`). Both have
identical runner config and identical bodies.

### When
`product verify FT-XXX` is invoked, then `product context FT-XXX` is
assembled.

### Then
- Both TCs are executed.
- Both TCs have their status updated identically based on runner exit code.
- Both TCs appear in the context bundle.
- Neither TC triggers W004, G002, G009 (custom types carry no mechanics).