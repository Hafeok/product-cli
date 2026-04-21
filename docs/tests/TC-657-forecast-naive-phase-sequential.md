---
id: TC-657
title: forecast_naive_phase_sequential
type: scenario
status: unimplemented
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_657_forecast_naive_phase_sequential
---

## Session — forecast-naive-phase-sequential

### Given

A fixture at `now = 2026-05-22T12:00Z` with:
- Phase 2 has 5 remaining features (`FT-009, FT-010, FT-011,
  FT-012, FT-013`), none complete, none in-progress.
- 5 recent completions provide `median 4.01d, min 2.44d, max
  7.22d`.

### When

The user runs `product forecast --phase 2 --naive`.

### Then

- The header reads `Phase 2 — <name>` and
  `Features remaining: 5 (FT-009, FT-010, FT-011, FT-012,
  FT-013)`.
- The projection block reports:
  - `Likely completion:` = today + 5 × 4.01 ≈ today + 20.05d →
    `2026-06-11` (± 1 day for rounding).
  - `Optimistic:` = today + 5 × 2.44 ≈ today + 12.2d →
    `2026-06-03` (± 1 day).
  - `Pessimistic:` = today + 5 × 7.22 ≈ today + 36.1d →
    `2026-06-27` (± 1 day).
- The footer contains the strings
  `Assumes no parallelism and no dependency blocking.` and
  `For a more precise forecast, export cycle times:
   product cycle-times --format csv > cycle-times.csv`.
