---
id: TC-597
title: migration_phase1_deprecation_tc_passes
type: scenario
status: passing
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_597_migration_phase1_deprecation_tc_passes
last-run: 2026-04-20T08:01:08.284116371+00:00
last-run-duration: 0.2s
---

## Session: ST-151 — migration-phase1-deprecation-tc-passes

### Given
A repository in mid-migration: the deprecated thing is still present and
decorated with a deprecation marker. The phase-1 absence TC's runner asserts
"the deprecation warning is emitted on use".

### When
`product verify --platform` runs.

### Then
- The phase-1 TC's runner exits 0 (warning observed).
- The TC's status is `passing`.
- Platform verify exits 0.