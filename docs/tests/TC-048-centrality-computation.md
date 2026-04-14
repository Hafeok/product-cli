---
id: TC-048
title: centrality_computation
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
runner-args: "tc_048_centrality_computation"
last-run: 2026-04-14T13:57:28.405167723+00:00
---

load a graph with known topology. Assert betweenness centrality values match hand-computed expected values within ±0.001.