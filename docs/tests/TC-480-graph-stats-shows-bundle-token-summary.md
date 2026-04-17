---
id: TC-480
title: graph stats shows bundle token summary
type: scenario
status: passing
validates:
  features:
  - FT-040
  adrs:
  - ADR-024
phase: 1
runner: cargo-test
runner-args: tc_480_graph_stats_shows_bundle_token_summary
last-run: 2026-04-17T09:56:49.097152789+00:00
last-run-duration: 0.2s
---

## Scenario

Given a repo with 3 features, 2 of which have `bundle` blocks in front-matter (from prior `--measure` runs), when `product graph stats` is run, the output includes a "Bundle size" section showing measured count, mean, median, p95, max (with feature ID), and min (with feature ID) token values, plus threshold breach lines.