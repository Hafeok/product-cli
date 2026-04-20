---
id: TC-586
title: absence_tc_passes_when_thing_gone
type: scenario
status: passing
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_586_absence_tc_passes_when_thing_gone
last-run: 2026-04-20T08:01:08.284116371+00:00
last-run-duration: 0.2s
---

## Session: ST-140 — absence-tc-passes-when-thing-gone

### Given
A repository with one absence TC whose runner is `bash -c 'true'` (always
exits 0), validating an ADR with `removes: [foo]`.

### When
`product verify --platform` is invoked.

### Then
- The absence TC's runner is executed.
- The runner exits 0.
- The TC's status in front-matter is set to `passing`.
- The platform verify exits 0 (no failing TCs).