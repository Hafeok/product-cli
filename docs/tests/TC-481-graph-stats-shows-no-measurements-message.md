---
id: TC-481
title: graph stats shows no measurements message
type: scenario
status: passing
validates:
  features:
  - FT-040
  adrs:
  - ADR-024
phase: 1
runner: cargo-test
runner-args: tc_481_graph_stats_shows_no_measurements_message
last-run: 2026-04-17T09:56:49.097152789+00:00
last-run-duration: 0.2s
---

## Scenario

Given a repo with features that have NO `bundle` blocks, when `product graph stats` is run, the output includes the line "No bundle measurements" suggesting the user run `product context --measure-all`.