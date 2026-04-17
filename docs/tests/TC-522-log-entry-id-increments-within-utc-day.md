---
id: TC-522
title: log entry id increments within utc day
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_522_log_entry_id_increments_within_utc_day
---

## Description

Entry IDs follow `req-{YYYYMMDD}-{NNN}` and the sequence `NNN` increments monotonically within a UTC day, resetting at UTC midnight.

## Setup

1. Fixture with time mocking (e.g. via a fake clock, or by freezing wall-clock at known UTC times).
2. Three successive applies at UTC times: `2026-04-14T23:59:00Z`, `2026-04-14T23:59:30Z`, `2026-04-15T00:00:10Z`.

## Steps

1. Apply request A at the first mocked time. Assert the resulting entry `id == "req-20260414-001"`.
2. Apply request B at the second mocked time. Assert `id == "req-20260414-002"`.
3. Apply request C at the third mocked time. Assert `id == "req-20260415-001"` (sequence reset at UTC midnight).

## Invariant

Entry IDs are date-sequence, per-UTC-day, and start at `001` each day. The sequence counter never skips within a day.
