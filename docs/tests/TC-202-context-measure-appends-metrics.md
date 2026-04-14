---
id: TC-202
title: context_measure_appends_metrics
type: scenario
status: unimplemented
validates:
  features: 
  - FT-011
  adrs:
  - ADR-006
phase: 1
---

run `product context FT-001 --measure`. Assert an entry is appended to `metrics.jsonl` containing the feature ID and bundle dimensions.