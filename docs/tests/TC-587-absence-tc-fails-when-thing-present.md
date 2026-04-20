---
id: TC-587
title: absence_tc_fails_when_thing_present
type: scenario
status: passing
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_587_absence_tc_fails_when_thing_present
last-run: 2026-04-20T08:01:08.284116371+00:00
last-run-duration: 0.2s
---

## Session: ST-141 — absence-tc-fails-when-thing-present

### Given
A repository with one absence TC whose runner is `bash -c 'exit 1'` (always
fails), validating an ADR with `removes: [foo]`.

### When
`product verify --platform` is invoked.

### Then
- The absence TC's runner is executed.
- The runner exits non-zero.
- The TC's status in front-matter is set to `failing`.
- The platform verify exits 1.