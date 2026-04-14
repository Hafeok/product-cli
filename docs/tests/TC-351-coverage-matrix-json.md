---
id: TC-351
title: coverage_matrix_json
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

run `product graph coverage --format json`. Assert valid JSON with `features` array, each containing `domains` map with coverage status.