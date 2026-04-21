---
id: TC-645
title: cycle_times_lists_complete_features
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_645_cycle_times_lists_complete_features
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.7s
---

## Session — cycle-times-lists-complete-features

### Given

A repository fixture with 3 complete features:
- `FT-101` with tags `product/FT-101/started` (at 2026-04-08T13:00Z)
  and `product/FT-101/complete` (at 2026-04-11T09:14Z).
- `FT-102` with `started` at 2026-04-12T10:30Z and `complete` at
  2026-04-17T15:42Z.
- `FT-103` with `started` at 2026-04-15T08:00Z and `complete` at
  2026-04-18T18:00Z.

### When

The user runs `product cycle-times` in the fixture root.

### Then

- Exit code is 0.
- Output contains one row per feature ordered by `started`
  timestamp ascending: `FT-101`, `FT-102`, `FT-103`.
- Each row renders the started date (YYYY-MM-DD), the completed
  date (YYYY-MM-DD), and the cycle time in days with one
  decimal (computed as `(completed - started).num_seconds() /
  86400.0`, rounded to one decimal).
- The summary footer reports `count: 3`, an `All` row with
  `median / min / max`, and no trend line (count < 6 per
  ADR-046 §4).