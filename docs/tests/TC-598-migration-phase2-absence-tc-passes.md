---
id: TC-598
title: migration_phase2_absence_tc_passes
type: scenario
status: unimplemented
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_598_migration_phase2_absence_tc_passes
---

## Session: ST-152 — migration-phase2-absence-tc-passes

### Given
A repository post-migration: the deprecated thing is removed. The phase-2
absence TC's runner asserts "the thing does not exist anywhere in the
codebase".

### When
`product verify --platform` runs.

### Then
- The phase-2 TC's runner exits 0 (thing absent).
- The TC's status is `passing`.
