---
id: TC-652
title: cycle_times_trend_slowing
type: scenario
status: unimplemented
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_652_cycle_times_trend_slowing
---

## Session — cycle-times-trend-slowing

### Given

A fixture with ≥ 6 complete features where the recent-5 median
is more than 25% above the all-time median. For example:
- Historical cycle times: `3.0, 3.5, 2.8, 3.2, 3.1, 3.3`
  (older) and `6.0, 5.5, 7.0, 6.8, 5.9` (most recent 5).
- All-time median ≈ 4.4d; recent-5 median 6.0d; ratio ≈ 0.36
  (above +0.25 threshold).

### When

The user runs `product cycle-times` with default
`trend-threshold = 0.25`.

### Then

- The summary footer shows `Trend: slowing` with the sub-label
  `(recent > historical)`.
- `product cycle-times --format json` sets
  `summary.trend = "slowing"`.

### And

Setting `[cycle-times].trend-threshold = 0.50` on the same
fixture switches the classifier output to `stable` — the
threshold is the only knob controlling the boundary.
