---
id: TC-660
title: status_shows_cycle_time_column_when_data_present
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_660_status_shows_cycle_time_column_when_data_present
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.4s
---

## Session — status-shows-cycle-time-column-when-data-present

### Given

A fixture at `now = 2026-05-22T12:00Z` with default
`[cycle-times].min-features = 3` and:
- 3 complete features with known cycle times (e.g. `FT-001
  2.84d`, `FT-002 5.12d`, `FT-003 3.21d`). Recent-N median ≈
  3.21d.
- `FT-004` in-progress, `started` @ 2026-05-19T12:00Z
  (elapsed 3.0d).
- `FT-005` planned, no tags.

### When

The user runs `product status`.

### Then

- Output renders a cycle-time column next to status.
- Complete features show their cycle time (e.g. `2.84d`,
  `5.12d`, `3.21d`).
- `FT-004` renders `elapsed 3.0d  (recent median: 3.2d)` (per
  ADR-046 §12).
- `FT-005` renders an empty cycle-time cell.
- The column header reads `Cycle time` (or equivalent
  constant).