---
id: TC-464
title: adr scope validates enum values
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_464_adr_scope_validates_enum_values"
last-run: 2026-04-18T10:42:03.345580667+00:00
last-run-duration: 0.2s
---

Run `product adr scope ADR-XXX invalid-scope`. Assert exit code 1 and error E001. Run with each valid value: `cross-cutting`, `domain`, `feature-specific`. Assert exit code 0 for each and the `scope` field in front-matter matches the set value.