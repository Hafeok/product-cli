---
id: TC-643
title: due_date_never_blocks_verification_or_phase_gate
type: scenario
status: unimplemented
validates:
  features:
  - FT-053
  adrs:
  - ADR-045
phase: 5
---

## Session — due-date-is-advisory

### Given

A fixture repo where `FT-009` is `status: in-progress`,
`due-date: 2026-04-01` (long overdue), with all TCs
passing but the feature status not yet advanced by verify.

### When

The user runs `product verify FT-009`.

### Then

- Verify runs all TCs, they pass, and the feature transitions
  to `status: complete` — the same behaviour as a feature with
  no `due-date`.
- The overall `product verify` exit code is 0 for the feature
  completion; the separate W028 finding is reported at exit 2
  if run against `product verify` in graph-check stage.
- The phase-gate evaluation for `FT-009`'s phase is unaffected
   by the missed date.

### And

No code path makes `product verify` exit 1 solely because a
`due-date` has passed.
