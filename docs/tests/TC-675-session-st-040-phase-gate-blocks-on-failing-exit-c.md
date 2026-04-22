---
id: TC-675
title: session ST-040 phase-gate-blocks-on-failing-exit-criteria
type: session
status: passing
validates:
  features:
  - FT-043
  adrs:
  - ADR-018
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_675_session_st_040_phase_gate_blocks_on_failing_exit_criteria
last-run: 2026-04-22T12:59:08.455929045+00:00
last-run-duration: 0.2s
---

Session ST-040 — feature next refuses to advance to phase 2 when phase 1 has a failing exit-criteria TC.