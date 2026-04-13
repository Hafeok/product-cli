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
---

parse a `⟦Λ:Scenario⟧` block with all three fields. Assert `given`, `when`, `then` are all populated.