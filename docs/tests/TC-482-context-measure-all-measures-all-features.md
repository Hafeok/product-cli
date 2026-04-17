---
id: TC-482
title: context measure-all measures all features
type: scenario
status: passing
validates:
  features:
  - FT-040
  adrs:
  - ADR-006
  - ADR-024
phase: 1
runner: cargo-test
runner-args: tc_482_context_measure_all_measures_all_features
last-run: 2026-04-17T09:56:49.097152789+00:00
last-run-duration: 0.2s
---

## Scenario

Given a repo with 3 features and no prior measurements, when `product context --measure-all` is run, all 3 feature files are updated with `bundle` blocks containing `tokens-approx` values, a metrics.jsonl file is created with entries for each feature, and the command exits with code 0.