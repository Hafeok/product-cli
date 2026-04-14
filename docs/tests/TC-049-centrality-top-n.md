---
id: TC-049
title: centrality_top_n
type: scenario
status: passing
validates:
  features:
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  - FT-014
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_049_centrality_top_n"
last-run: 2026-04-14T14:04:19.495078770+00:00
---

assert `product graph central --top 3` returns exactly 3 ADRs in descending centrality order.