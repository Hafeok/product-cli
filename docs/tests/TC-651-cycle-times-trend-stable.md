---
id: TC-651
title: cycle_times_trend_stable
type: scenario
status: unimplemented
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_651_cycle_times_trend_stable
---

## Session — cycle-times-trend-stable

### Given

A fixture with ≥ 6 complete features where the recent-5 median
is within ±25% of the all-time median. The sample-data table
in `docs/product-cycle-times-spec (2).md` (14 features, recent-5
median 4.01d vs all-time median 4.02d, ratio ≈ 0.0025).

### When

The user runs `product cycle-times` with default
`trend-threshold = 0.25`.

### Then

- The summary footer shows `Trend: stable` with the sub-label
  `(recent ≈ historical)`.
- `product cycle-times --format json` sets
  `summary.trend = "stable"`.
