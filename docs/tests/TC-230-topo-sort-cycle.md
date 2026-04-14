---
id: TC-230
title: topo_sort_cycle
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

FT-001 depends-on FT-002, FT-002 depends-on FT-001. Assert `product graph check` exits with code 1 and names both features in the error message.