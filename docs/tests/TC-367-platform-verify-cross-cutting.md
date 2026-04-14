---
id: TC-367
title: platform_verify_cross_cutting
type: scenario
status: unimplemented
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
---

run `product verify --platform`. Assert TCs linked to cross-cutting ADRs are run. Assert their status is updated. Assert feature-specific TCs are not run.