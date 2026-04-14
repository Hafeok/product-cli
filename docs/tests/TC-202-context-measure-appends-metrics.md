---
id: TC-202
title: context_measure_appends_metrics
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_202_context_measure_appends_metrics"
validates:
  features: 
  - FT-011
  adrs:
  - ADR-006
phase: 1
last-run: 2026-04-14T13:57:28.405167723+00:00
---

run `product context FT-001 --measure`. Assert an entry is appended to `metrics.jsonl` containing the feature ID and bundle dimensions.