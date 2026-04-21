---
id: TC-661
title: status_omits_cycle_time_column_when_below_min
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-046
phase: 5
runner: cargo-test
runner-args: tc_661_status_omits_cycle_time_column_when_below_min
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.4s
---

## Session — status-omits-cycle-time-column-when-below-min

### Given

A fixture with `[cycle-times].min-features = 3` and only 2
complete features with both `started` and `complete` tags.
Several features are in-progress or planned.

### When

The user runs `product status`.

### Then

- The cycle-time column is entirely omitted from the output —
  not rendered as empty, not rendered as `—`, simply absent.
- No warning is emitted about insufficient data; this is a
  rendering-only decision and the user has not asked for cycle
  times.
- Raising `min-features` to 1 on the same fixture causes the
  column to re-appear (the threshold is the only knob).

### And

Running `product status --format json` on the same fixture
omits the `cycle_time_days` field from every feature entry
when the column is suppressed (JSON parity with the text
rendering).