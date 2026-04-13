---
id: TC-073
title: parse_scenario_block
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-015
  adrs:
  - ADR-016
phase: 1
---

parse a `⟦Λ:Scenario⟧` block with all three fields. Assert `given`, `when`, `then` are all populated.