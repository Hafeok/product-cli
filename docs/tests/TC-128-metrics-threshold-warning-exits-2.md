---
id: TC-128
title: metrics_threshold_warning_exits_2
type: exit-criteria
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-024
phase: 1
runner: cargo-test
runner-args: "tc_128_metrics_threshold_warning_exits_2"
last-run: 2026-04-18T10:41:56.996985101+00:00
last-run-duration: 0.2s
---

breach a warning-severity threshold only. Assert exit code 2.