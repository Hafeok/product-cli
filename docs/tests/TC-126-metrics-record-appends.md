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
runner: cargo-test
runner-args: "tc_126_metrics_record_appends"
last-run: 2026-04-18T10:41:56.996985101+00:00
last-run-duration: 0.2s
---

run `product metrics record` twice. Assert `metrics.jsonl` has two lines and both are valid JSON with all required fields.