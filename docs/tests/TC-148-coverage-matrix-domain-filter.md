---
id: TC-148
title: coverage_matrix_domain_filter
type: scenario
status: unimplemented
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
---

run `product graph coverage --domain security`. Assert output contains only the security column.