---
id: TC-483
title: context measure-all with depth flag
type: scenario
status: passing
validates:
  features:
  - FT-040
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: tc_483_context_measure_all_with_depth_flag
last-run: 2026-04-17T09:56:49.097152789+00:00
last-run-duration: 0.3s
---

## Scenario

Given a repo with features, when `product context --measure-all --depth 2` is run, the bundle assembly uses depth 2 for BFS traversal, and the resulting `bundle` blocks reflect the wider context.