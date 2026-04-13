---
id: TC-077
title: parse_empty_block_warning
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

parse `⟦Γ:Invariants⟧{}`. Assert W004. Assert no error.