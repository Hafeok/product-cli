---
id: TC-656
title: forecast_naive_single_feature
type: scenario
status: unimplemented
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_656_forecast_naive_single_feature
---

## Session — forecast-naive-single-feature

### Given

A fixture at simulated `now = 2026-05-22T12:00Z` with:
- `FT-015` in-progress; `product/FT-015/started` @
  2026-05-20T07:00Z (elapsed 2.2d).
- 5 complete features with cycle times: `2.44, 6.78, 4.01,
  3.55, 7.22` (recent_5: median 4.01d, min 2.44d, max 7.22d).

### When

The user runs `product forecast FT-015 --naive`.

### Then

- Output header reads
  `FT-015 — <title>  [in-progress, started 2026-05-20]`.
- `Elapsed:` row reads `2.2d` (one decimal).
- `Recent 5 complete features:` row reads
  `median 4.01d  ·  range 2.44 – 7.22d`.
- `Naive projection` block shows:
  - `Likely completion:` = today + max(0, 4.01 - 2.2) ≈ today +
    1.81d → `2026-05-24`.
  - `Optimistic:` = today + max(0, 2.44 - 2.2) ≈ today + 0.24d →
    `2026-05-22`.
  - `Pessimistic:` = today + max(0, 7.22 - 2.2) ≈ today + 5.02d →
    `2026-05-27` or `2026-05-28` (depending on rounding of the
    today-offset).
- A trailing disclaimer reads
  `This is a rough estimate based on 5 recent features. It is
   not a probability forecast.`
