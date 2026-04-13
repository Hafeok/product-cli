---
id: TC-037
title: formal_block_parse_scenario
type: scenario
status: passing
validates:
  features:
  - FT-015
  adrs:
  - ADR-011
phase: 1
---

parse a `⟦Λ:Scenario⟧` block with `given/when/then` fields. Assert all three fields are present and non-empty.