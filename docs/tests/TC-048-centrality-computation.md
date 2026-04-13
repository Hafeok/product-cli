---
id: TC-048
title: centrality_computation
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
runner-args: "tc_048_centrality_computation"
---

load a graph with known topology. Assert betweenness centrality values match hand-computed expected values within ±0.001.