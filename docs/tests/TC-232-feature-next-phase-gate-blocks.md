---
id: TC-232
title: feature_next_phase_gate_blocks
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_232_feature_next_phase_gate_blocks"
validates:
  features: 
  - FT-006
  - FT-011
  - FT-014
  - FT-016
  - FT-024
  adrs:
  - ADR-012
phase: 1
last-run: 2026-04-14T15:03:33.506444091+00:00
---

Phase 1 has TC-007 (exit-criteria, failing). FT-005 is phase 2. Assert `product feature next` skips FT-005 and reports the phase gate with TC-007 named. Assert it returns a remaining phase-1 feature instead.