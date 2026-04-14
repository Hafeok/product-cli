---
id: TC-309
title: verify_platform_runs_cross_cutting
type: scenario
status: unimplemented
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
---

run `product verify --platform`. Assert TCs linked to cross-cutting ADRs run. Assert feature-specific TCs not run.