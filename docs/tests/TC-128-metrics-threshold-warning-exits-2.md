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
last-run: 2026-04-13T14:27:30.366814571+00:00
---

breach a warning-severity threshold only. Assert exit code 2.