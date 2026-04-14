---
id: TC-191
title: graph_stale_ttl
type: scenario
status: unimplemented
validates:
  features: 
  - FT-016
  adrs:
  - ADR-003
phase: 1
---

generate `index.ttl`, then add a new feature file. Invoke `product feature list`. Assert the new feature appears in the list (graph was rebuilt from files, not from stale TTL).