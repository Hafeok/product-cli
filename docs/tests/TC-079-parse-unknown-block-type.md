---
id: TC-079
title: parse_unknown_block_type
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

parse `⟦X:Unknown⟧{ ... }`. Assert E001 with "unrecognised block type".