---
id: TC-650
title: cycle_times_trend_accelerating
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_650_cycle_times_trend_accelerating
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.7s
---

## Session — cycle-times-trend-accelerating

### Given

A fixture with ≥ 6 complete features where the recent-5 median
is more than 25% below the all-time median. For example:
- All-time cycle times: `8.0, 7.5, 9.0, 8.5, 7.8, 8.2` (older)
  and `3.0, 3.5, 2.8, 3.2, 4.0` (most recent 5).
- All-time median ≈ 5.75d; recent-5 median ≈ 3.2d; ratio ≈
  -0.44 (below -0.25 threshold).

### When

The user runs `product cycle-times` with default
`trend-threshold = 0.25`.

### Then

- The summary footer includes `Trend: accelerating` with a
  sub-label such as `(recent < historical)`.
- `product cycle-times --format json` sets
  `summary.trend = "accelerating"`.