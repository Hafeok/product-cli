---
id: TC-126
title: metrics_record_appends
type: scenario
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-024
phase: 1
---

run `product metrics record` twice. Assert `metrics.jsonl` has two lines and both are valid JSON with all required fields.