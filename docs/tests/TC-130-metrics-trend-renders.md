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
last-run: 2026-04-13T14:27:30.366814571+00:00
---

`metrics.jsonl` with 10 records. Run `product metrics trend`. Assert stdout contains sparkline output (non-empty, no errors).