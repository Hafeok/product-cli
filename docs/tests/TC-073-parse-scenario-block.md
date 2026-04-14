---
id: TC-073
title: parse_scenario_block
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-004
  - FT-015
  adrs:
  - ADR-016
phase: 1
runner: cargo-test
runner-args: "tc_073_parse_scenario_block"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

parse a `⟦Λ:Scenario⟧` block with all three fields. Assert `given`, `when`, `then` are all populated.