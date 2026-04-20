---
id: TC-601
title: tc_type_exit_criteria_drives_phase_gate
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_601_tc_type_exit_criteria_drives_phase_gate"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 2.3s
---

## Session: ST-180 — tc-type-exit-criteria-drives-phase-gate

### Given
A repository with a phase-1 feature linked to one `exit-criteria` TC. The
TC's status is `failing`. A phase-2 feature exists with status `planned`.

### When
`product feature next` is invoked.

### Then
- The phase-2 feature is NOT returned (gate closed).
- When the `exit-criteria` TC's status is changed to `passing` and the
  command is re-run, the phase-2 feature IS returned.
- No other TC type closes or opens the gate (verified by adding a `failing`
  scenario TC and observing no effect).