---
id: TC-604
title: tc_type_absence_drives_g009
type: scenario
status: unimplemented
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
---

## Session: ST-183 — tc-type-absence-drives-g009

### Given
An accepted ADR with non-empty `removes:` and no linked TC.

### When
`product gap check` runs, then a TC with `type: absence` is linked to the
ADR (via `validates.adrs`), then the command is re-run.

### Then
- First run: G009 is reported.
- Second run: G009 is cleared.
- Linking a TC of any other type (scenario, invariant, chaos, benchmark, or
  a custom type) does NOT clear G009.
