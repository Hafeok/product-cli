---
id: TC-127
title: metrics_threshold_error_exits_1
type: exit-criteria
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-024
phase: 1
runner: cargo-test
runner-args: "tc_127_metrics_threshold_error_exits_1"
last-run: 2026-04-13T14:27:30.366814571+00:00
---

set `spec_coverage` threshold, configure a repo below it. Run `product metrics threshold`. Assert exit code 1.