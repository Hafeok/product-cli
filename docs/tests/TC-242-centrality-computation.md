---
id: TC-242
title: centrality_computation
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

load a graph with known topology. Assert betweenness centrality values match hand-computed expected values within ±0.001.