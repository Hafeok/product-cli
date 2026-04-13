---
id: TC-049
title: centrality_top_n
type: scenario
status: passing
validates:
  features:
  - FT-011
  - FT-016
  adrs:
  - ADR-012
phase: 1
---

assert `product graph central --top 3` returns exactly 3 ADRs in descending centrality order.