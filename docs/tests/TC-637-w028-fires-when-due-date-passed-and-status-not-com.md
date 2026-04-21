---
id: TC-637
title: w028_fires_when_due_date_passed_and_status_not_complete
type: scenario
status: unimplemented
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
---

## Session — w028-fires-when-overdue

### Given

A fixture repo containing:
- `FT-009` with `status: in-progress` and
  `due-date: 2026-04-01` (in the past relative to the test
  clock fixed at 2026-04-21).
- `FT-010` with `status: complete` and
  `due-date: 2026-04-01` (also in the past).

### When

The user runs `product verify` stage 2 (graph structure).

### Then

- `FT-009` emits W028 with a message naming the feature id,
  title, and how many days ago the date passed.
- `FT-010` does NOT emit W028 (status is `complete`).
- The overall exit code is 2 (W-class), not 1.
- `product status` output includes a visible overdue flag next
  to `FT-009`.
