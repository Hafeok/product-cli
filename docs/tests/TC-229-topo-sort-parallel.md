---
id: TC-229
title: topo_sort_parallel
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

FT-002 and FT-003 both depend-on FT-001, no dependency between FT-002 and FT-003. Assert FT-001 appears before both; FT-002 and FT-003 order is unspecified.