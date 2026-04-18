---
id: TC-587
title: absence_tc_fails_when_thing_present
type: scenario
status: unimplemented
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
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
