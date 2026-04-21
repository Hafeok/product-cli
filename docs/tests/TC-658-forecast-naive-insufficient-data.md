---
id: TC-658
title: forecast_naive_insufficient_data
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_658_forecast_naive_insufficient_data
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.4s
---

## Session — forecast-naive-insufficient-data

### Given

A fixture with only 2 complete features (both have `started`
and `complete` tags), `FT-015` in-progress with a `started`
tag, and default `[cycle-times].min-features = 3`.

### When

The user runs `product forecast FT-015 --naive`.

### Then

- Exit code is 2 (user asked for a projection we cannot
  responsibly give; ADR-046 §8).
- stderr contains `Insufficient data for naive projection.`
- The message reports both the current count (`Only 2
  features ...`) and the required minimum (`requires at
  least 3`).
- The message suggests the corrective path `View current
  cycle times:  product cycle-times`.

### And

Running `product cycle-times` on the same fixture exits 0 with
an empty table and a footer explaining that at least 3 complete
features are required for summary statistics (no projection is
attempted there).