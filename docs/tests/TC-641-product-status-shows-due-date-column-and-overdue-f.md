---
id: TC-641
title: product_status_shows_due_date_column_and_overdue_flag
type: scenario
status: passing
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
runner: cargo-test
runner-args: tc_641_product_status_shows_due_date_column_and_overdue_flag
last-run: 2026-04-21T13:09:33.430942991+00:00
last-run-duration: 0.2s
---

## Session — status-shows-due-date-column

### Given

A fixture repo with:
- `FT-003` `status: in-progress`, `due-date: 2026-04-30`
  (future).
- `FT-009` `status: planned`, `due-date: 2026-04-15`
  (overdue at clock fixed 2026-04-21).
- `FT-012` `status: planned`, no `due-date`.

### When

The user runs `product status`.

### Then

- The rendered output shows a `due …` cell for `FT-003` and
  `FT-009`, and no date cell for `FT-012`.
- `FT-009`'s row carries a visible overdue indicator (the
  W028 glyph or an equivalent text marker).
- `FT-003`'s row has no overdue marker.
- Status output parses cleanly by the existing status formatter
  tests (columns align, no trailing whitespace).