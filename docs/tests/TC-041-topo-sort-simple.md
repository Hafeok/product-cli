---
id: TC-041
title: topo_sort_simple
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
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_041_topo_sort_simple"
last-run: 2026-04-14T14:04:19.495078770+00:00
---

three features: FT-001, FT-002 depends-on FT-001, FT-003 depends-on FT-002. Assert topological order is [FT-001, FT-002, FT-003].
  - FT-006