---
id: TC-593
title: g009_clear_when_absence_tc_linked
type: scenario
status: passing
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_593_g009_clear_when_absence_tc_linked
last-run: 2026-04-20T08:01:08.284116371+00:00
last-run-duration: 0.2s
---

## Session: ST-147 — g009-clear-when-absence-tc-linked

### Given
The ST-145 fixture, then a request that creates an absence TC linked to the
offending ADR via `validates.adrs` is applied.

### When
`product gap check` and `product graph check` are both re-run.

### Then
- No G009 finding is reported.
- No W022 warning is reported.
- Both commands exit 0.