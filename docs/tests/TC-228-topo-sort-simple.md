---
id: TC-228
title: topo_sort_simple
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

three features: FT-001, FT-002 depends-on FT-001, FT-003 depends-on FT-002. Assert topological order is [FT-001, FT-002, FT-003].