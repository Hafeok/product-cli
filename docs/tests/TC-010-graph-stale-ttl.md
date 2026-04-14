---
id: TC-010
title: graph_stale_ttl
type: scenario
status: passing
validates:
  features:
  - FT-006
  - FT-016
  - FT-024
  - FT-014
  adrs:
  - ADR-003
phase: 1
runner: cargo-test
runner-args: "tc_010_graph_stale_ttl"
last-run: 2026-04-14T14:04:19.495078770+00:00
---

generate `index.ttl`, then add a new feature file. Invoke `product feature list`. Assert the new feature appears in the list (graph was rebuilt from files, not from stale TTL).