---
id: TC-636
title: due_date_field_parses_iso_8601_date
type: scenario
status: passing
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
runner: cargo-test
runner-args: tc_636_due_date_field_parses_iso_8601_date
last-run: 2026-04-21T13:09:33.430942991+00:00
last-run-duration: 0.3s
---

## Session — due-date-parses-iso-8601

### Given

A feature file `FT-009` with front-matter including
`due-date: 2026-05-01`.

### When

The graph is rebuilt via `product graph check` and the feature
is loaded back.

### Then

- The parsed feature's `due_date` equals `2026-05-01`
  (`NaiveDate::from_ymd_opt(2026, 5, 1)`).
- Round-trip: re-serialising the feature produces front-matter
  with `due-date: 2026-05-01` exactly (no timezone suffix, no
  time component).

### And

A feature with `due-date: "not-a-date"` fails to parse with
E006 and an `expected YYYY-MM-DD` hint. The graph rebuild
surfaces the parse error at the offending file path without
masking other artifacts.