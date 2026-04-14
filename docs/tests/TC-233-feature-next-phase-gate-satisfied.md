---
id: TC-233
title: feature_next_phase_gate_satisfied
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_233_feature_next_phase_gate_satisfied"
validates:
  features: 
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  adrs:
  - ADR-012
phase: 1
last-run: 2026-04-14T13:57:28.405167723+00:00
---

all phase-1 exit-criteria TCs are passing. Assert `product feature next` returns the first eligible phase-2 feature.