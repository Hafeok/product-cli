---
id: TC-026
title: sparql_phase_filter
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
  - ADR-008
phase: 1
runner: cargo-test
runner-args: "tc_026_sparql_phase_filter"
---

execute a query filtering features by `pm:phase 1`. Assert only phase-1 features appear in the result.