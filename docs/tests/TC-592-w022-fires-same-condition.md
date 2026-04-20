---
id: TC-592
title: w022_fires_same_condition
type: scenario
status: passing
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_592_w022_fires_same_condition
last-run: 2026-04-20T08:01:08.284116371+00:00
last-run-duration: 0.2s
---

## Session: ST-146 — w022-fires-same-condition

### Given
A repository with an accepted ADR whose `removes:` is non-empty and no
linked absence TC (same fixture as ST-145).

### When
`product graph check` runs.

### Then
- One W022 warning is reported, naming the offending ADR.
- Severity is `warning`.
- W022 message text matches the G009 message text shape (same underlying
  rule).
- Exit code is 2 (warnings only, no errors).