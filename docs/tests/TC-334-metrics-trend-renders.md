---
id: TC-334
title: metrics_trend_renders
type: scenario
status: unimplemented
validates:
  features: 
  - FT-028
  adrs:
  - ADR-024
phase: 1
---

`metrics.jsonl` with 10 records. Run `product metrics trend`. Assert stdout contains sparkline output (non-empty, no errors).