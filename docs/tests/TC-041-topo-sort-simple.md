---
id: TC-041
title: topo_sort_simple
type: scenario
status: passing
validates:
  features:
  - FT-011
  - FT-016
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_041_topo_sort_simple"
---

three features: FT-001, FT-002 depends-on FT-001, FT-003 depends-on FT-002. Assert topological order is [FT-001, FT-002, FT-003].