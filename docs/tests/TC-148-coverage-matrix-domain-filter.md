---
id: TC-148
title: coverage_matrix_domain_filter
type: scenario
status: passing
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
runner: cargo-test
runner-args: "tc_148_coverage_matrix_domain_filter"
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

run `product graph coverage --domain security`. Assert output contains only the security column.