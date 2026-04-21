---
id: TC-638
title: w029_fires_within_configurable_warning_window_and_can_be_disabled
type: scenario
status: passing
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
runner: cargo-test
runner-args: tc_638_w029_fires_within_configurable_warning_window_and_can_be_disabled
last-run: 2026-04-21T13:09:33.430942991+00:00
last-run-duration: 0.2s
---

## Session — w029-fires-within-warning-window

### Given

A fixture repo with the test clock fixed at 2026-04-21 and:
- `FT-009` `status: in-progress`, `due-date: 2026-04-23`
  (2 days out).
- `FT-010` `status: in-progress`, `due-date: 2026-05-10`
  (19 days out).
- `product.toml` `[planning].due-date-warning-days = 3`.

### When

The user runs `product verify`.

### Then

- `FT-009` emits W029 (2 days ≤ 3-day window).
- `FT-010` does NOT emit W029 (19 > 3).
- Exit code is 2.

### And

Setting `[planning].due-date-warning-days = 0` and re-running
verify suppresses W029 for `FT-009` (0 disables the warning
entirely). W028 is unaffected by the `due-date-warning-days`
knob.