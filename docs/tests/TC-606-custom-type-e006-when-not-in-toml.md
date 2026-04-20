---
id: TC-606
title: custom_type_e006_when_not_in_toml
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_606_custom_type_e006_when_not_in_toml"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.1s
---

## Session: ST-185 — custom-type-e006-when-not-in-toml

### Given
A repository with `[tc-types].custom = ["contract"]` and a TC declaring
`type: smoke`.

### When
`product graph check` runs.

### Then
- E006 is emitted naming the TC and the unknown type `smoke`.
- The error message lists the built-in types AND the configured custom types
  (`["contract"]`).
- The error message includes a `product request change` snippet that would
  add `smoke` to `[tc-types].custom`.
- Exit code is 1.