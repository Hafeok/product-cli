---
id: TC-130
title: metrics_trend_renders
type: scenario
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-024
phase: 1
runner: cargo-test
runner-args: "tc_130_metrics_trend_renders"
last-run: 2026-04-18T10:41:56.996985101+00:00
last-run-duration: 0.2s
---

`metrics.jsonl` with 10 records. Run `product metrics trend`. Assert stdout contains sparkline output (non-empty, no errors).