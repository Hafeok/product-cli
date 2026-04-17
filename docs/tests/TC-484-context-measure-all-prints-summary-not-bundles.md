---
id: TC-484
title: context measure-all prints summary not bundles
type: scenario
status: passing
validates:
  features:
  - FT-040
  adrs:
  - ADR-024
phase: 1
runner: cargo-test
runner-args: tc_484_context_measure_all_prints_summary_not_bundles
last-run: 2026-04-17T09:56:49.097152789+00:00
last-run-duration: 0.2s
---

## Scenario

Given a repo with 3 features, when `product context --measure-all` is run, stdout contains an aggregate summary table (with "measured:", "mean:", "median:" lines) but does NOT contain the full bundle content (no "# Context Bundle:" headers). Individual bundle content is suppressed to avoid flooding stdout.