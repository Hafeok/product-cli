---
id: TC-482
title: context measure-all measures all features
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Scenario

Given a repo with 3 features and no prior measurements, when `product context --measure-all` is run, all 3 feature files are updated with `bundle` blocks containing `tokens-approx` values, a metrics.jsonl file is created with entries for each feature, and the command exits with code 0.