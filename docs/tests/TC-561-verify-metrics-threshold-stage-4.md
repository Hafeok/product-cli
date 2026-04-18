---
id: TC-561
title: verify_metrics_threshold_stage_4
type: scenario
status: passing
validates:
  features:
  - FT-044
  adrs:
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_561_verify_metrics_threshold_stage_4
last-run: 2026-04-18T10:54:35.903075401+00:00
last-run-duration: 0.2s
---

## Session: ST-119 — verify-metrics-threshold-stage-4

**Validates:** FT-044, ADR-040 (Stage 4 applies metric thresholds)

### Given

A temp repository with `[metrics.thresholds.bundle_tokens_p95]` set to `max = 5000, severity = "warning"` and an actual p95 above that.

### When

`product verify` is run.

### Then

- Stage 4 (metrics) emits `warning` with a finding naming the breached threshold.
- Exit code is `2` (warning-level).
- If the same threshold is `severity = "error"` instead, stage 4 emits `fail` and exit code is `1`.
- `--ci` JSON: `stages[3].findings` contains the threshold name.