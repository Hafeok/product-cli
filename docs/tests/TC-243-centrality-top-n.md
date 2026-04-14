---
id: TC-243
title: centrality_top_n
type: scenario
status: unimplemented
validates:
  features: 
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  adrs:
  - ADR-012
phase: 1
---

assert `product graph central --top 3` returns exactly 3 ADRs in descending centrality order.