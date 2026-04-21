---
id: TC-649
title: cycle_times_recent_5_computed_correctly
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_649_cycle_times_recent_5_computed_correctly
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 1.0s
---

## Session — cycle-times-recent-5-computed-correctly

### Given

A fixture with 14 complete features whose cycle times in days
are (in order of completion timestamp ascending):
`2.84, 5.12, 3.21, 8.44, 2.10, 4.88, 1.95, 11.32, 3.67, 2.44,
6.78, 4.01, 3.55, 7.22`.

The `[cycle-times]` config has `recent-window = 5` (default).

### When

The user runs `product cycle-times --format json`.

### Then

- `summary.count` equals 14.
- `summary.recent_5.median` equals 4.01 (median of the last five
  completions: 2.44, 6.78, 4.01, 3.55, 7.22 → sorted 2.44, 3.55,
  **4.01**, 6.78, 7.22).
- `summary.recent_5.min` equals 2.44.
- `summary.recent_5.max` equals 7.22.
- `summary.all.median` equals 4.02 (median of all 14 values).
- `summary.all.min` equals 1.95, `summary.all.max` equals
  11.32.

### And

All values are rendered with one decimal precision; the JSON
numbers are parseable as floats whose string representation
matches what the text table would print.