---
id: TC-1026
title: divergence rate trend is surfaced across runs
type: scenario
status: passing
validates:
  features:
  - FT-145
  adrs:
  - ADR-089
phase: 7
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_1026_divergence_rate_trend_is_surfaced_across_runs
---

## Scenario — spec staleness becomes measurable as the rate moves

**Given** an authored `OrderShape` and `OrdersLive` dataset,
**When** the user runs `domain data OrdersLive` on clean data,
**Then** the verdict reports a 0.0% divergence rate marked **first run** and
records it to the per-product history,
**And when** the data then drifts (an undeclared shipping value) and the user
runs `domain data OrdersLive` again,
**Then** the verdict reports the rate **rising** against the previous run — the
§3.1 data-divergence signal made visible as it happens,
**And** `--no-record` runs the check without appending to the history, so the
standing signal is only written when intended.

## Validates

- FT-145 — Domain model structure/data split and data conformance
- ADR-089 — The data side is first-class: reference data is What, production data is the oracle
