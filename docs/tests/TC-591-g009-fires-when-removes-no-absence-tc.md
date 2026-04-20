---
id: TC-591
title: g009_fires_when_removes_no_absence_tc
type: scenario
status: passing
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_591_g009_fires_when_removes_no_absence_tc
last-run: 2026-04-20T08:01:08.284116371+00:00
last-run-duration: 0.2s
---

## Session: ST-145 — g009-fires-when-removes-no-absence-tc

### Given
A repository with an accepted ADR whose `removes:` is non-empty and whose
linked TCs are all of `tc-type: scenario` (no absence TC).

### When
`product gap check` runs.

### Then
- One G009 finding is reported, naming the offending ADR.
- Severity is `high`.
- The exit code is 1 (new gap finding).
- Same shape applies for an ADR with non-empty `deprecates:` and no absence
  TC (parameterised case in the same TC).